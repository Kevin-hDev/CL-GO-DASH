from .adapter_utils import (
    forecast_payload_result,
    forecast_quantile_index,
    simple_result,
    values_tensor,
)
from .config_utils import config_bool, config_int
from .config_utils import standard_quantile_levels
from .toto_covariates import build_covariate_jobs, format_covariate_predictions
from .validation import validate_column_names


class TotoAdapter:
    def __init__(self, _family_id, _model_name, model_dir):
        self.model_dir = str(model_dir)
        self.model = None

    def predict(self, payload, horizon, quantile_levels):
        quantile_levels = standard_quantile_levels(quantile_levels)
        covariates = validate_column_names(payload.get("covariate_columns"))
        if covariates:
            return self._predict_with_covariates(
                payload, horizon, quantile_levels, covariates
            )
        return forecast_payload_result(
            payload,
            horizon,
            quantile_levels,
            lambda values, length, levels: self._forecast_one(
                values, length, levels, payload
            ),
        )

    def _predict_with_covariates(self, payload, horizon, quantile_levels, covariates):
        jobs = build_covariate_jobs(payload, horizon, covariates)
        if len(jobs) == 1 and jobs[0]["series_id"] is None:
            median, quantiles = self._forecast_with_covariates(
                jobs[0]["values"],
                jobs[0]["covariates"],
                horizon,
                quantile_levels,
                payload,
            )
            return simple_result(median, quantile_levels, quantiles, horizon)

        forecasts = []
        for job in jobs:
            median, quantiles = self._forecast_with_covariates(
                job["values"],
                job["covariates"],
                horizon,
                quantile_levels,
                payload,
            )
            forecasts.append((job, median, quantiles))
        return format_covariate_predictions(forecasts, quantile_levels, horizon)

    def _forecast_one(self, values, horizon, quantile_levels, payload):
        import torch

        model = self._load_model().to("cpu").eval()
        patch_size = max(1, int(getattr(model.config, "patch_size", 32)))
        pad_count = (-len(values)) % patch_size
        padded = ([values[0]] * pad_count) + values if pad_count else values

        target = values_tensor(padded).view(1, 1, -1)
        mask = torch.ones_like(target, dtype=torch.bool)
        if pad_count:
            mask[..., :pad_count] = False
        series_ids = torch.zeros(1, 1, dtype=torch.long)
        decode_block_size = config_int(payload, "decode_block_size", 768, 1, 4096)
        has_missing_values = config_bool(payload, "has_missing_values", bool(pad_count))
        with torch.no_grad():
            quantiles = model.forecast(
                {"target": target, "target_mask": mask, "series_ids": series_ids},
                horizon=horizon,
                decode_block_size=decode_block_size,
                has_missing_values=has_missing_values or bool(pad_count),
            )
        q50 = quantiles[forecast_quantile_index(0.5), 0, 0, :horizon]
        selected = [
            quantiles[forecast_quantile_index(level), 0, 0, :horizon]
            for level in quantile_levels
        ]
        return q50, selected

    def _forecast_with_covariates(
        self, values, covariates, horizon, quantile_levels, payload
    ):
        import torch

        model = self._load_model().to("cpu").eval()
        patch_size = max(1, int(getattr(model.config, "patch_size", 32)))
        pad_count = (-len(values)) % patch_size
        padded = ([values[0]] * pad_count) + values if pad_count else values

        target = values_tensor(padded).view(1, 1, -1)
        mask = torch.ones_like(target, dtype=torch.bool)
        if pad_count:
            mask[..., :pad_count] = False
        series_ids = torch.zeros(1, 1, dtype=torch.long)

        dynamic_values = []
        dynamic_masks = []
        for series in covariates:
            history = series["history"]
            future = series["future"]
            padded_values = ([history[0]] * pad_count) + history + future
            padded_mask = (
                ([False] * pad_count)
                + series["history_mask"]
                + series["future_mask"]
            )
            dynamic_values.append(padded_values)
            dynamic_masks.append(padded_mask)

        known_dynamic = values_tensor(dynamic_values).view(1, len(dynamic_values), -1)
        known_dynamic_mask = torch.tensor(dynamic_masks, dtype=torch.bool).view(
            1, len(dynamic_masks), -1
        )
        known_dynamic_ids = torch.zeros(1, len(dynamic_values), dtype=torch.long)

        decode_block_size = config_int(payload, "decode_block_size", 768, 1, 4096)
        has_missing_values = config_bool(payload, "has_missing_values", False)
        with torch.no_grad():
            quantiles = model.forecast(
                {
                    "target": target,
                    "target_mask": mask,
                    "series_ids": series_ids,
                    "known_dynamic": known_dynamic,
                    "known_dynamic_mask": known_dynamic_mask,
                    "known_dynamic_series_ids": known_dynamic_ids,
                },
                horizon=horizon,
                decode_block_size=decode_block_size,
                has_missing_values=has_missing_values
                or bool(pad_count)
                or not bool(known_dynamic_mask.all()),
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
