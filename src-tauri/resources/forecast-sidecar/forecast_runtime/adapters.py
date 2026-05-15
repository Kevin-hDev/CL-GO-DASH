from pathlib import Path

from .chronos_adapter import ChronosAdapter
from .flowstate_adapter import FlowStateAdapter
from .kairos_adapter import KairosAdapter
from .moirai_adapter import MoiraiAdapter
from .sundial_adapter import SundialAdapter
from .tabpfn_adapter import TabPfnTsAdapter
from .timesfm_adapter import TimesFmAdapter
from .tirex_adapter import TiRexAdapter
from .toto_adapter import TotoAdapter

ADAPTERS = {
    "chronos-bolt": ChronosAdapter,
    "chronos-2": ChronosAdapter,
    "timesfm-2-5": TimesFmAdapter,
    "toto-2": TotoAdapter,
    "moirai-2": MoiraiAdapter,
    "flowstate": FlowStateAdapter,
    "tabpfn-ts": TabPfnTsAdapter,
    "tirex": TiRexAdapter,
    "kairos": KairosAdapter,
    "sundial": SundialAdapter,
}


def load_adapter(family_id, model_name, models_dir):
    adapter_cls = ADAPTERS.get(family_id)
    if adapter_cls is None:
        raise SystemExit("missing_adapter")

    model_dir = Path(models_dir).joinpath(model_name)
    if not model_dir.exists():
        raise SystemExit("missing_model")
    return adapter_cls(family_id, model_name, model_dir)
