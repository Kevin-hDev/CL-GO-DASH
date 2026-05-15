from .adapter_utils import forecast_payload_result


class MoiraiAdapter:
    def __init__(self, _family_id, _model_name, model_dir):
        self.model_dir = str(model_dir)
        self.predictor = None
        self.horizon = None

    def predict(self, payload, horizon, quantile_levels):
        return forecast_payload_result(
            payload, horizon, quantile_levels, self._forecast_one
        )

    def _forecast_one(self, values, horizon, quantile_levels):
        predictor = self._load_predictor(horizon, len(values))
        dataset = self._dataset(values, "D")
        forecast = next(iter(predictor.predict(dataset)))
        median = forecast.quantile(0.5)[:horizon]
        quantiles = [forecast.quantile(level)[:horizon] for level in quantile_levels]
        return median, quantiles

    def _load_predictor(self, horizon, context_length):
        if self.predictor is not None and self.horizon == horizon:
            return self.predictor
        from uni2ts.model.moirai2 import Moirai2Forecast, Moirai2Module

        model = Moirai2Forecast(
            module=Moirai2Module.from_pretrained(self.model_dir),
            prediction_length=horizon,
            context_length=min(max(context_length, horizon), 1680),
            target_dim=1,
            feat_dynamic_real_dim=0,
            past_feat_dynamic_real_dim=0,
        )
        self.predictor = model.create_predictor(batch_size=1)
        self.horizon = horizon
        return self.predictor

    def _dataset(self, values, frequency):
        from gluonts.dataset.common import ListDataset

        return ListDataset([{"start": "2000-01-01", "target": values}], freq=frequency)
