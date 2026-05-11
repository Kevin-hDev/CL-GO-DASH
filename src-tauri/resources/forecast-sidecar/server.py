#!/usr/bin/env python3
import argparse
import json
import math
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

MAX_POINTS = 10000
MAX_HORIZON = 1000


class ForecastHandler(BaseHTTPRequestHandler):
    server_version = "CLGOForecastSidecar/1.0"

    def do_GET(self):
        if self.path != "/health":
            self._send_json(404, {"error": "not_found"})
            return
        self._send_json(200, {"ok": True})

    def do_POST(self):
        if self.path != "/predict":
            self._send_json(404, {"error": "not_found"})
            return
        try:
            length = int(self.headers.get("content-length", "0"))
            if length <= 0 or length > 2 * 1024 * 1024:
                self._send_json(400, {"error": "invalid_payload"})
                return
            payload = json.loads(self.rfile.read(length))
            values = payload.get("values")
            horizon = int(payload.get("horizon", 0))
            if not isinstance(values, list) or not values or len(values) > MAX_POINTS:
                self._send_json(400, {"error": "invalid_values"})
                return
            if horizon <= 0 or horizon > MAX_HORIZON:
                self._send_json(400, {"error": "invalid_horizon"})
                return
            series = [float(v) for v in values if isinstance(v, (int, float))]
            if not series:
                self._send_json(400, {"error": "invalid_values"})
                return
            self._send_json(200, predict(series, horizon))
        except Exception:
            self._send_json(500, {"error": "prediction_failed"})

    def log_message(self, _format, *_args):
        return

    def _send_json(self, status, payload):
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def predict(values, horizon):
    window = values[-min(len(values), 12):]
    baseline = sum(window) / len(window)
    trend = safe_trend(values)
    median = [baseline + trend * (i + 1) for i in range(horizon)]
    spread = safe_spread(window)
    q10 = [v - spread for v in median]
    q90 = [v + spread for v in median]
    return {"median": median, "q10": q10, "q90": q90}


def safe_trend(values):
    if len(values) < 2:
        return 0.0
    span = min(len(values) - 1, 12)
    return (values[-1] - values[-1 - span]) / span


def safe_spread(values):
    mean = sum(values) / len(values)
    variance = sum((v - mean) ** 2 for v in values) / len(values)
    spread = math.sqrt(variance) * 1.28
    return spread if math.isfinite(spread) else 0.0


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--models-dir", required=True)
    args = parser.parse_args()
    server = ThreadingHTTPServer(("127.0.0.1", args.port), ForecastHandler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        server.server_close()


if __name__ == "__main__":
    main()
