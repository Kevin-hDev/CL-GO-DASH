from .adapter_utils import forecast_payload_result


class TabPfnTsAdapter:
    def __init__(self, _family_id, _model_name, _model_dir):
        self.pipeline = None

    def predict(self, payload, horizon, quantile_levels):
        return forecast_payload_result(
            payload, horizon, quantile_levels, self._forecast_one
        )

    def _forecast_one(self, values, horizon, quantile_levels):
        context_df, future_df = self._frames(values, horizon)
        pipeline = self._load_pipeline()
        predictions = pipeline.predict_df(
            context_df=context_df,
            future_df=future_df,
        )
        median = self._prediction_values(predictions, horizon)
        quantiles = self._prediction_quantiles(
            predictions, horizon, quantile_levels
        )
        return median, quantiles

    def _load_pipeline(self):
        if self.pipeline is None:
            from tabpfn_time_series import TabPFNMode, TabPFNTSPipeline

            self.pipeline = TabPFNTSPipeline(tabpfn_mode=TabPFNMode.LOCAL)
        return self.pipeline

    def _frames(self, values, horizon):
        import pandas as pd

        context = pd.DataFrame(
            {
                "item_id": ["series-1"] * len(values),
                "timestamp": pd.date_range("2000-01-01", periods=len(values), freq="D"),
                "target": values,
            }
        )
        future = pd.DataFrame(
            {
                "item_id": ["series-1"] * horizon,
                "timestamp": pd.date_range("2100-01-01", periods=horizon, freq="D"),
            }
        )
        return context, future

    def _prediction_values(self, predictions, horizon):
        for column in ("median", "mean", "prediction", "target"):
            if column in predictions and predictions[column].dtype.kind in "fiu":
                return predictions[column].tolist()[:horizon]
        candidates = [
            column
            for column in predictions.columns
            if column not in {"item_id", "timestamp"}
            and predictions[column].dtype.kind in "fiu"
        ]
        if not candidates:
            raise ValueError("prediction_failed")
        return predictions[candidates[0]].tolist()[:horizon]

    def _prediction_quantiles(self, predictions, horizon, quantile_levels):
        selected = []
        for level in quantile_levels:
            column = self._quantile_column(predictions, level)
            if column is None:
                return None
            selected.append(predictions[column].tolist()[:horizon])
        return selected

    def _quantile_column(self, predictions, level):
        pct = int(round(level * 100))
        candidates = {
            f"q{pct}",
            f"q{pct:02d}",
            f"p{pct}",
            f"{level}",
            f"{level:.1f}",
            f"quantile_{level}",
            f"quantile_{pct}",
        }
        for column in predictions.columns:
            if str(column).lower() in candidates:
                return column
        return None
