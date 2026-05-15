from .validation import validate_values


FAMILY_IMPORTS = {
    "timesfm-2-5": ("timesfm",),
    "toto-2": ("toto",),
    "moirai-2": ("uni2ts",),
    "flowstate": ("tsfm_public",),
    "tabpfn-ts": ("tabpfn_time_series",),
    "tirex": ("tirex",),
    "kairos": ("kairos",),
    "sundial": ("sundial",),
}


class ExternalAdapter:
    def __init__(self, family_id, _model_name, model_dir):
        self.family_id = family_id
        self.model_dir = model_dir
        self._assert_dependency_available()

    def predict(self, payload, horizon, quantile_levels):
        values = validate_values(payload.get("values"))
        if self.family_id == "timesfm-2-5":
            return self._predict_timesfm(values, horizon, quantile_levels)
        raise ValueError("adapter_not_ready")

    def _assert_dependency_available(self):
        imports = FAMILY_IMPORTS.get(self.family_id)
        if not imports:
            raise ValueError("adapter_not_ready")
        for module_name in imports:
            try:
                __import__(module_name)
            except Exception as exc:
                raise ValueError("adapter_not_ready") from exc

    def _predict_timesfm(self, values, horizon, quantile_levels):
        try:
            import timesfm
        except Exception as exc:
            raise ValueError("adapter_not_ready") from exc

        checkpoint = self._timesfm_checkpoint(timesfm)
        hparams = timesfm.TimesFmHparams(
            backend="torch",
            per_core_batch_size=1,
            horizon_len=horizon,
            context_len=min(len(values), 2048),
        )
        model = timesfm.TimesFm(hparams=hparams, checkpoint=checkpoint)
        forecast, quantile_forecast = model.forecast([values], freq=[0])
        median = list(map(float, forecast[0][:horizon]))
        result = {"median": median}

        if quantile_forecast is not None:
            for index, level in enumerate(quantile_levels):
                result[f"q{int(round(level * 100)):02d}"] = [
                    float(row[index]) for row in quantile_forecast[0][:horizon]
                ]
        return result

    def _timesfm_checkpoint(self, timesfm):
        path = str(self.model_dir)
        try:
            return timesfm.TimesFmCheckpoint(path=path)
        except TypeError:
            return timesfm.TimesFmCheckpoint(huggingface_repo_id=path)
