from .adapter_utils import forecast_payload_result, values_tensor
from .config_utils import config, standard_quantile_levels
from .quantile_utils import select_standard_quantiles


class TiRexAdapter:
    def __init__(self, _family_id, _model_name, model_dir):
        self.model_dir = str(model_dir)
        self.model = None

    def predict(self, payload, horizon, quantile_levels):
        quantile_levels = standard_quantile_levels(quantile_levels)
        return forecast_payload_result(
            payload,
            horizon,
            quantile_levels,
            lambda values, length, levels: self._forecast_one(
                values, length, levels, payload
            ),
        )

    def _forecast_one(self, values, horizon, quantile_levels, payload):
        import torch

        with torch.no_grad():
            quantiles, mean = self._load_model().forecast(
                context=values_tensor(values).view(1, -1),
                prediction_length=horizon,
            )
        if config(payload).get("output_type") == "mean":
            return mean[0][:horizon], None
        selected = select_standard_quantiles(
            quantiles[0], horizon, quantile_levels
        )
        return mean[0][:horizon], selected

    def _load_model(self):
        if self.model is None:
            from tirex import load_model

            self.model = load_model(self.model_dir)
        return self.model
