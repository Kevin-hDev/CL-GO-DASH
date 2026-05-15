import math

MAX_POINTS = 10000
MAX_HORIZON = 1000
MAX_COVARIATES = 64


def validate_values(values):
    if not isinstance(values, list) or not values or len(values) > MAX_POINTS:
        raise ValueError("invalid_values")
    series = []
    for value in values:
        if not isinstance(value, (int, float)):
            raise ValueError("invalid_values")
        numeric = float(value)
        if not math.isfinite(numeric):
            raise ValueError("invalid_values")
        series.append(numeric)
    return series


def validate_horizon(raw_horizon):
    horizon = int(raw_horizon or 0)
    if horizon <= 0 or horizon > MAX_HORIZON:
        raise ValueError("invalid_horizon")
    return horizon


def validate_quantiles(raw_quantiles):
    if not isinstance(raw_quantiles, list) or not raw_quantiles:
        raise ValueError("invalid_quantiles")
    normalized = []
    for quantile in raw_quantiles:
        if not isinstance(quantile, (int, float)):
            raise ValueError("invalid_quantiles")
        value = float(quantile)
        if value <= 0 or value >= 1:
            raise ValueError("invalid_quantiles")
        normalized.append(value)
    return normalized


def validate_row_dicts(rows, error_code):
    if rows is None:
        return []
    if not isinstance(rows, list) or len(rows) > MAX_POINTS:
        raise ValueError(error_code)
    normalized = []
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError(error_code)
        normalized.append(row)
    return normalized


def validate_column_name(value, error_code):
    if not isinstance(value, str) or not value.strip() or len(value) > 128:
        raise ValueError(error_code)
    return value


def validate_column_names(values):
    if not isinstance(values, list):
        return []
    result = []
    for value in values[:MAX_COVARIATES]:
        if isinstance(value, str) and value.strip() and len(value) <= 128:
            result.append(value)
    return result


def validate_optional_column_name(value):
    if value is None:
        return None
    return validate_column_name(value, "invalid_series_column")


def read_string_value(value, error_code):
    if not isinstance(value, str) or not value.strip():
        raise ValueError(error_code)
    return value


def read_series_value(row, series_column):
    if not series_column:
        return "series-1"
    value = row.get(series_column)
    if isinstance(value, str):
        trimmed = value.strip()
        if trimmed:
            return trimmed
    elif isinstance(value, (int, float, bool)):
        return str(value)
    raise ValueError("invalid_series_value")


def read_numeric_value(value, error_code):
    if not isinstance(value, (int, float)) or not math.isfinite(float(value)):
        raise ValueError(error_code)
    return float(value)


def quantile_key(level):
    return f"q{int(round(level * 100)):02d}"
