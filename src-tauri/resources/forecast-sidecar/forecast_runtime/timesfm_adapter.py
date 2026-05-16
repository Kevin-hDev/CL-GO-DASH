from .adapter_utils import forecast_payload_result, values_tensor
from .config_utils import config_int, standard_quantile_levels
from .device_utils import move_tensor, transformers_device_map
from .quantile_utils import select_standard_quantiles


class TimesFmAdapter:
    def __init__(self, _family_id, _model_name, model_dir, device="gpu"):
        self.model_dir = str(model_dir)
        self.device = device
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
        try:
            return self._predict_transformers(values, horizon, quantile_levels)
        except Exception:
            return self._predict_timesfm_package(
                values, horizon, quantile_levels, payload
            )

    def _predict_transformers(self, values, horizon, quantile_levels):
        import torch
        from transformers import TimesFm2_5ModelForPrediction

        model = self._load_transformers_model(TimesFm2_5ModelForPrediction)
        with torch.no_grad():
            context = move_tensor(values_tensor(values), self.device)
            outputs = model(past_values=[context], return_dict=True)
        median = outputs.mean_predictions[0][:horizon]
        quantiles = self._select_quantiles(
            getattr(outputs, "full_predictions", None),
            horizon,
            quantile_levels,
            drop_mean=True,
        )
        return median, quantiles

    def _load_transformers_model(self, model_class):
        if self.model is None:
            self.model = model_class.from_pretrained(
                self.model_dir, device_map=transformers_device_map(self.device)
            ).eval()
        return self.model

    def _predict_timesfm_package(self, values, horizon, quantile_levels, payload):
        import timesfm

        checkpoint = self._timesfm_checkpoint(timesfm)
        context_len = config_int(payload, "context_length", 0, 0, 100000)
        hparams = timesfm.TimesFmHparams(
            backend="torch",
            per_core_batch_size=1,
            horizon_len=horizon,
            context_len=context_len or min(len(values), 2048),
        )
        model = timesfm.TimesFm(hparams=hparams, checkpoint=checkpoint)
        forecast, quantile_forecast = model.forecast([values], freq=[0])
        quantiles = self._select_quantiles(
            quantile_forecast, horizon, quantile_levels, drop_mean=True
        )
        return forecast[0][:horizon], quantiles

    def _select_quantiles(self, quantiles, horizon, quantile_levels, drop_mean):
        if quantiles is None:
            return None
        return select_standard_quantiles(
            quantiles[0], horizon, quantile_levels, drop_mean=drop_mean
        )

    def _timesfm_checkpoint(self, timesfm):
        try:
            return timesfm.TimesFmCheckpoint(path=self.model_dir)
        except TypeError:
            return timesfm.TimesFmCheckpoint(huggingface_repo_id=self.model_dir)
