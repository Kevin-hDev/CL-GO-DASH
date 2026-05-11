#!/usr/bin/env python3
import argparse
import json
import math
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import pandas as pd
import torch
from chronos import BaseChronosPipeline, Chronos2Pipeline

MAX_POINTS = 10000
MAX_HORIZON = 1000

PIPELINE = None
MODEL_NAME = ""
MODEL_KIND = ""


class ForecastHandler(BaseHTTPRequestHandler):
    server_version = "CLGOForecastSidecar/2.0"

    def do_GET(self):
        if self.path != "/health":
            self._send_json(404, {"error": "not_found"})
            return
        self._send_json(200, {"ok": True, "model": MODEL_NAME})

    def do_POST(self):
        if self.path != "/predict":
            self._send_json(404, {"error": "not_found"})
            return
        try:
            payload = self._read_payload()
            horizon = validate_horizon(payload.get("horizon"))
            quantiles = validate_quantiles(payload.get("quantiles"))
            result = predict(payload, horizon, quantiles)
            self._send_json(200, result)
        except ValueError as exc:
            self._send_json(400, {"error": str(exc)})
        except Exception:
            self._send_json(500, {"error": "prediction_failed"})

    def log_message(self, _format, *_args):
        return

    def _read_payload(self):
        length = int(self.headers.get("content-length", "0"))
        if length <= 0 or length > 2 * 1024 * 1024:
            raise ValueError("invalid_payload")
        return json.loads(self.rfile.read(length))

    def _send_json(self, status, payload):
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def validate_values(values):
    if not isinstance(values, list) or not values or len(values) > MAX_POINTS:
        raise ValueError("invalid_values")
    series = []
    for value in values:
        if not isinstance(value, (int, float)):
            raise ValueError("invalid_values")
        numeric = float(value)
        if not math.isfinite(numeric):
            raise ValueError("invalid_values")
        series.append(numeric)
    return series


def validate_horizon(raw_horizon):
    horizon = int(raw_horizon or 0)
    if horizon <= 0 or horizon > MAX_HORIZON:
        raise ValueError("invalid_horizon")
    return horizon


def validate_quantiles(raw_quantiles):
    if not isinstance(raw_quantiles, list) or not raw_quantiles:
        raise ValueError("invalid_quantiles")
    normalized = []
    for quantile in raw_quantiles:
        if not isinstance(quantile, (int, float)):
            raise ValueError("invalid_quantiles")
        value = float(quantile)
        if value <= 0 or value >= 1:
            raise ValueError("invalid_quantiles")
        normalized.append(value)
    return normalized


def predict(payload, horizon, quantile_levels):
    if MODEL_KIND == "chronos2" and "history_rows" in payload:
        return predict_chronos2_df(payload, horizon, quantile_levels)
    values = validate_values(payload.get("values"))
    if MODEL_KIND == "chronos2":
        return predict_chronos2(values, horizon, quantile_levels)
    return predict_chronos_bolt(values, horizon, quantile_levels)


def predict_chronos_bolt(values, horizon, quantile_levels):
    context = torch.tensor(values, dtype=torch.float32)
    quantiles, median = PIPELINE.predict_quantiles(
        context, prediction_length=horizon, quantile_levels=quantile_levels
    )
    quantiles = quantiles.squeeze(0)
    median = median.squeeze(0)
    result = {"median": median.tolist()}

    for index, level in enumerate(quantile_levels):
        result[quantile_key(level)] = quantiles[:, index].tolist()
    return result


def predict_chronos2(values, horizon, quantile_levels):
    context = torch.tensor(values, dtype=torch.float32).view(1, 1, -1)
    quantile_list, mean_list = PIPELINE.predict_quantiles(
        context, prediction_length=horizon, quantile_levels=quantile_levels
    )
    quantiles = quantile_list[0].squeeze(0)
    median = mean_list[0].squeeze(0)
    result = {"median": median.tolist()}

    for index, level in enumerate(quantile_levels):
        result[quantile_key(level)] = quantiles[:, index].tolist()
    return result


def predict_chronos2_df(payload, horizon, quantile_levels):
    history_rows = validate_row_dicts(
        payload.get("history_rows"), "invalid_history_rows"
    )
    future_rows = validate_row_dicts(
        payload.get("future_rows"), "invalid_future_rows"
    )
    date_column = validate_column_name(
        payload.get("date_column"), "invalid_date_column"
    )
    target_column = validate_column_name(
        payload.get("target_column"), "invalid_target_column"
    )
    covariate_columns = validate_column_names(payload.get("covariate_columns"))

    if not history_rows:
        raise ValueError("invalid_history_rows")
    if future_rows and len(future_rows) != horizon:
        raise ValueError("invalid_future_rows")

    history_df = pd.DataFrame(
        [
            build_history_record(
                row, date_column, target_column, covariate_columns
            )
            for row in history_rows
        ]
    )
    future_df = None
    if future_rows:
        future_df = pd.DataFrame(
            [
                build_future_record(row, date_column, covariate_columns)
                for row in future_rows
            ]
        )

    predictions = PIPELINE.predict_df(
        history_df,
        future_df=future_df,
        id_column="item_id",
        timestamp_column="timestamp",
        target="target",
        prediction_length=horizon,
        quantile_levels=quantile_levels,
    ).sort_values("timestamp")

    result = {"median": predictions["predictions"].tolist()}
    for level in quantile_levels:
        result[quantile_key(level)] = predictions[str(level)].tolist()
    return result


def quantile_key(level):
    return f"q{int(round(level * 100)):02d}"


def validate_row_dicts(rows, error_code):
    if rows is None:
        return []
    if not isinstance(rows, list):
        raise ValueError(error_code)
    normalized = []
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError(error_code)
        normalized.append(row)
    return normalized


def validate_column_name(value, error_code):
    if not isinstance(value, str) or not value.strip():
        raise ValueError(error_code)
    return value


def validate_column_names(values):
    if not isinstance(values, list):
        return []
    result = []
    for value in values:
        if isinstance(value, str) and value.strip():
            result.append(value)
    return result


def build_history_record(row, date_column, target_column, covariate_columns):
    target = read_numeric_value(row.get(target_column), "invalid_target")
    record = {
        "item_id": "series-1",
        "timestamp": read_string_value(row.get(date_column), "invalid_date"),
        "target": target,
    }
    for name in covariate_columns:
        record[name] = row.get(name)
    return record


def build_future_record(row, date_column, covariate_columns):
    record = {
        "item_id": "series-1",
        "timestamp": read_string_value(row.get(date_column), "invalid_date"),
    }
    for name in covariate_columns:
        record[name] = row.get(name)
    return record


def read_string_value(value, error_code):
    if not isinstance(value, str) or not value.strip():
        raise ValueError(error_code)
    return value


def read_numeric_value(value, error_code):
    if not isinstance(value, (int, float)) or not math.isfinite(float(value)):
        raise ValueError(error_code)
    return float(value)


def load_pipeline(model_name, models_dir):
    model_dir = Path(models_dir).joinpath(model_name)
    if not model_dir.exists():
        raise SystemExit("missing_model")
    config_path = model_dir.joinpath("config.json")
    config = json.loads(config_path.read_text(encoding="utf-8"))
    architectures = config.get("architectures") or []
    if "Chronos2Model" in architectures:
        return "chronos2", Chronos2Pipeline.from_pretrained(
            str(model_dir), device_map="cpu"
        )
    return "chronos_bolt", BaseChronosPipeline.from_pretrained(
        str(model_dir), device_map="cpu"
    )


def main():
    global MODEL_NAME
    global MODEL_KIND
    global PIPELINE

    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--models-dir", required=True)
    args = parser.parse_args()

    MODEL_NAME = args.model
    MODEL_KIND, PIPELINE = load_pipeline(args.model, args.models_dir)

    server = ThreadingHTTPServer(("127.0.0.1", args.port), ForecastHandler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        server.server_close()


if __name__ == "__main__":
    main()
