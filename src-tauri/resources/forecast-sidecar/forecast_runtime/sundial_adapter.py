from .adapter_utils import forecast_payload_result, values_tensor


class SundialAdapter:
    def __init__(self, _family_id, _model_name, model_dir):
        self.model_dir = str(model_dir)
        self.model = None

    def predict(self, payload, horizon, quantile_levels):
        return forecast_payload_result(
            payload, horizon, quantile_levels, self._forecast_one
        )

    def _forecast_one(self, values, horizon, quantile_levels):
        import torch

        sample_count = 64
        context = values_tensor(values).view(1, -1)
        with torch.no_grad():
            samples = self._load_model().to("cpu").eval().generate(
                context,
                max_new_tokens=horizon,
                num_samples=sample_count,
            )
        sample_values = samples.float()
        if sample_values.shape[1] == sample_count:
            sample_values = sample_values[0, :, :horizon]
            sample_dim = 0
        elif len(sample_values.shape) > 2 and sample_values.shape[2] == sample_count:
            sample_values = sample_values[0, :horizon, :]
            sample_dim = 1
        else:
            raise ValueError("prediction_failed")
        median = torch.quantile(sample_values, 0.5, dim=sample_dim)[:horizon]
        quantiles = [
            torch.quantile(sample_values, level, dim=sample_dim)[:horizon]
            for level in quantile_levels
        ]
        return median, quantiles

    def _load_model(self):
        if self.model is None:
            from transformers import AutoModelForCausalLM

            self.model = AutoModelForCausalLM.from_pretrained(
                self.model_dir, trust_remote_code=True
            )
        return self.model
