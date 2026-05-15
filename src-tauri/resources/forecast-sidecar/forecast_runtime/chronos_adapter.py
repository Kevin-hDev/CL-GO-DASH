import json

import pandas as pd
import torch
from chronos import BaseChronosPipeline, Chronos2Pipeline

from .validation import (
    quantile_key,
    read_numeric_value,
    read_series_value,
    read_string_value,
    validate_column_name,
    validate_column_names,
    validate_optional_column_name,
    validate_row_dicts,
    validate_values,
)


class ChronosAdapter:
    def __init__(self, _family_id, _model_name, model_dir):
        self.kind, self.pipeline = self._load_pipeline(model_dir)

    def predict(self, payload, horizon, quantile_levels):
        if self.kind == "chronos2" and "history_rows" in payload:
            return self._predict_df(payload, horizon, quantile_levels)
        values = validate_values(payload.get("values"))
        if self.kind == "chronos2":
            return self._predict_chronos2(values, horizon, quantile_levels)
        return self._predict_bolt(values, horizon, quantile_levels)

    def _predict_bolt(self, values, horizon, quantile_levels):
        context = torch.tensor(values, dtype=torch.float32)
        quantiles, median = self.pipeline.predict_quantiles(
            context, prediction_length=horizon, quantile_levels=quantile_levels
        )
        return self._simple_result(
            quantiles.squeeze(0), median.squeeze(0), quantile_levels
        )

    def _predict_chronos2(self, values, horizon, quantile_levels):
        context = torch.tensor(values, dtype=torch.float32).view(1, 1, -1)
        quantile_list, mean_list = self.pipeline.predict_quantiles(
            context, prediction_length=horizon, quantile_levels=quantile_levels
        )
        return self._simple_result(
            quantile_list[0].squeeze(0), mean_list[0].squeeze(0), quantile_levels
        )

    def _predict_df(self, payload, horizon, quantile_levels):
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
        series_column = validate_optional_column_name(payload.get("series_column"))
        covariate_columns = validate_column_names(payload.get("covariate_columns"))

        if not history_rows:
            raise ValueError("invalid_history_rows")
        if future_rows and series_column is None and len(future_rows) != horizon:
            raise ValueError("invalid_future_rows")

        history_df = pd.DataFrame(
            [
                build_history_record(
                    row,
                    date_column,
                    target_column,
                    series_column,
                    covariate_columns,
                )
                for row in history_rows
            ]
        )
        future_df = None
        if future_rows:
            future_df = pd.DataFrame(
                [
                    build_future_record(
                        row, date_column, covariate_columns, series_column
                    )
                    for row in future_rows
                ]
            )

        predictions = self.pipeline.predict_df(
            history_df,
            future_df=future_df,
            id_column="item_id",
            timestamp_column="timestamp",
            target="target",
            prediction_length=horizon,
            quantile_levels=quantile_levels,
        ).sort_values(["item_id", "timestamp"])

        return {
            "predictions": [
                {
                    "series_id": record["item_id"],
                    "date": str(record["timestamp"]),
                    "value": float(record["predictions"]),
                    "q10": float(record.get("0.1", record["predictions"])),
                    "q50": float(record.get("0.5", record["predictions"])),
                    "q90": float(record.get("0.9", record["predictions"])),
                }
                for record in predictions.to_dict("records")
            ]
        }

    def _simple_result(self, quantiles, median, quantile_levels):
        result = {"median": median.tolist()}
        for index, level in enumerate(quantile_levels):
            result[quantile_key(level)] = quantiles[:, index].tolist()
        return result

    def _load_pipeline(self, model_dir):
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


def build_history_record(
    row, date_column, target_column, series_column, covariate_columns
):
    record = {
        "item_id": read_series_value(row, series_column),
        "timestamp": read_string_value(row.get(date_column), "invalid_date"),
        "target": read_numeric_value(row.get(target_column), "invalid_target"),
    }
    for name in covariate_columns:
        record[name] = row.get(name)
    return record


def build_future_record(row, date_column, covariate_columns, series_column=None):
    record = {
        "item_id": read_series_value(row, series_column),
        "timestamp": read_string_value(row.get(date_column), "invalid_date"),
    }
    for name in covariate_columns:
        record[name] = row.get(name)
    return record
