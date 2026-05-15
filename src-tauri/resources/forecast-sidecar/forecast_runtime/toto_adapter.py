from .adapter_utils import (
    forecast_payload_result,
    forecast_quantile_index,
    values_tensor,
)


class TotoAdapter:
    def __init__(self, _family_id, _model_name, model_dir):
        self.model_dir = str(model_dir)
        self.model = None

    def predict(self, payload, horizon, quantile_levels):
        return forecast_payload_result(
            payload, horizon, quantile_levels, self._forecast_one
        )

    def _forecast_one(self, values, horizon, quantile_levels):
        import torch

        target = values_tensor(values).view(1, 1, -1)
        mask = torch.ones_like(target, dtype=torch.bool)
        series_ids = torch.zeros(1, 1, dtype=torch.long)
        with torch.no_grad():
            quantiles = self._load_model().to("cpu").eval().forecast(
                {"target": target, "target_mask": mask, "series_ids": series_ids},
                horizon=horizon,
                decode_block_size=768,
                has_missing_values=False,
            )
        q50 = quantiles[forecast_quantile_index(0.5), 0, 0, :horizon]
        selected = [
            quantiles[forecast_quantile_index(level), 0, 0, :horizon]
            for level in quantile_levels
        ]
        return q50, selected

    def _load_model(self):
        if self.model is None:
            from toto2 import Toto2Model

            self.model = Toto2Model.from_pretrained(self.model_dir)
        return self.model
