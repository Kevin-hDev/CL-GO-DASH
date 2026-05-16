from .adapter_utils import forecast_payload_result, values_tensor
from .config_utils import config_bool, config_float
from .quantile_utils import select_standard_quantiles


class FlowStateAdapter:
    def __init__(self, _family_id, _model_name, model_dir):
        self.model_dir = str(model_dir)
        self.model = None

    def predict(self, payload, horizon, quantile_levels):
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

        context = values_tensor(values).view(1, -1, 1)
        batch_first = config_bool(payload, "batch_first", True)
        scale_factor = config_float(payload, "scale_factor", 1.0, 0.0001, 1000.0)
        with torch.no_grad():
            forecast = self._load_model().to("cpu").eval()(
                past_values=context,
                prediction_length=horizon,
                batch_first=batch_first,
                scale_factor=scale_factor,
            )
        median = forecast.prediction_outputs[0, :horizon, 0]
        quantiles = getattr(forecast, "quantile_outputs", None)
        if quantiles is not None:
            quantiles = quantiles[0]
            if len(quantiles.shape) > 2 and quantiles.shape[-1] == 1:
                quantiles = quantiles[..., 0]
            quantiles = select_standard_quantiles(
                quantiles, horizon, quantile_levels
            )
        return median, quantiles

    def _load_model(self):
        if self.model is None:
            from tsfm_public import FlowStateForPrediction

            self.model = FlowStateForPrediction.from_pretrained(self.model_dir)
        return self.model
