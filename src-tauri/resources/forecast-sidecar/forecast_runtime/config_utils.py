def config(payload):
    value = payload.get("model_config")
    return value if isinstance(value, dict) else {}


def config_bool(payload, key, default):
    value = config(payload).get(key)
    if isinstance(value, bool):
        return value
    return default


def config_int(payload, key, default, min_value=None, max_value=None):
    value = config(payload).get(key)
    if not isinstance(value, int):
        return default
    if min_value is not None and value < min_value:
        return default
    if max_value is not None and value > max_value:
        return default
    return value


def config_float(payload, key, default, min_value=None, max_value=None):
    value = config(payload).get(key)
    if not isinstance(value, (int, float)):
        return default
    value = float(value)
    if min_value is not None and value < min_value:
        return default
    if max_value is not None and value > max_value:
        return default
    return value
