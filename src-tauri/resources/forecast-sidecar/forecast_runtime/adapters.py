from pathlib import Path

from .chronos_adapter import ChronosAdapter
from .external_adapter import ExternalAdapter

ADAPTERS = {
    "chronos-bolt": ChronosAdapter,
    "chronos-2": ChronosAdapter,
    "timesfm-2-5": ExternalAdapter,
    "toto-2": ExternalAdapter,
    "moirai-2": ExternalAdapter,
    "flowstate": ExternalAdapter,
    "tabpfn-ts": ExternalAdapter,
    "tirex": ExternalAdapter,
    "kairos": ExternalAdapter,
    "sundial": ExternalAdapter,
}


def load_adapter(family_id, model_name, models_dir):
    adapter_cls = ADAPTERS.get(family_id)
    if adapter_cls is None:
        raise SystemExit("missing_adapter")

    model_dir = Path(models_dir).joinpath(model_name)
    if not model_dir.exists():
        raise SystemExit("missing_model")
    return adapter_cls(family_id, model_name, model_dir)
