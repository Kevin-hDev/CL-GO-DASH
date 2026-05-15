from .adapter_utils import (
    quantile_at,
    quantiles_to_lists,
    row_series_id,
    to_float_list,
)
from .validation import read_numeric_value


def build_covariate_jobs(payload, horizon, covariates):
    rows = payload.get("history_rows") or []
    target_column = payload.get("target_column")
    if not rows or not target_column:
        raise ValueError("invalid_covariates")

    series_column = payload.get("series_column")
    grouped = {}
    for row in rows:
        series_id = row_series_id(row, series_column)
        entry = grouped.setdefault(
            series_id,
            new_entry(None if not series_column else series_id, covariates),
        )
        target = read_numeric_value(row.get(target_column), "invalid_target")
        entry["values"].append(target)
        for name in covariates:
            value = read_optional_number(row.get(name))
            if value is None:
                raise ValueError("invalid_covariates")
            entry["covariates"][name]["history"].append(value)
            entry["covariates"][name]["history_mask"].append(True)

    add_future_covariates(grouped, payload, series_column, covariates)
    return [finish_entry(entry, covariates, horizon) for entry in grouped.values()]


def new_entry(series_id, covariates):
    return {
        "series_id": series_id,
        "values": [],
        "covariates": {
            name: {
                "history": [],
                "history_mask": [],
                "future": [],
                "future_mask": [],
            }
            for name in covariates
        },
        "dates": [],
    }


def add_future_covariates(grouped, payload, series_column, covariates):
    date_column = payload.get("date_column")
    for row in payload.get("future_rows") or []:
        series_id = row_series_id(row, series_column)
        entry = grouped.get(series_id)
        if entry is None:
            continue
        if date_column and isinstance(row.get(date_column), str):
            entry["dates"].append(row[date_column])
        for name in covariates:
            value = read_optional_number(row.get(name))
            entry["covariates"][name]["future"].append(0.0 if value is None else value)
            entry["covariates"][name]["future_mask"].append(value is not None)


def finish_entry(entry, covariates, horizon):
    for name in covariates:
        covariate = entry["covariates"][name]
        missing = horizon - len(covariate["future"])
        if missing > 0:
            covariate["future"].extend([0.0] * missing)
            covariate["future_mask"].extend([False] * missing)
        covariate["future"] = covariate["future"][:horizon]
        covariate["future_mask"] = covariate["future_mask"][:horizon]
    if len(entry["dates"]) < horizon:
        start = len(entry["dates"])
        entry["dates"].extend(f"T+{index + 1}" for index in range(start, horizon))
    return {
        "series_id": entry["series_id"],
        "values": entry["values"],
        "covariates": list(entry["covariates"].values()),
        "dates": entry["dates"][:horizon],
    }


def format_covariate_predictions(forecasts, quantile_levels, horizon):
    predictions = []
    for job, median, quantiles in forecasts:
        median_values = to_float_list(median)[:horizon]
        quantile_values = quantiles_to_lists(
            quantiles,
            len(quantile_levels),
            len(median_values),
        )
        quantile_by_key = {
            f"q{int(round(level * 100)):02d}": values
            for level, values in zip(quantile_levels, quantile_values, strict=False)
        }
        predictions.extend(
            format_job_predictions(job, median_values, quantile_by_key)
        )
    return {"predictions": predictions}


def format_job_predictions(job, median_values, quantile_by_key):
    items = []
    for index, value in enumerate(median_values):
        date = job["dates"][index] if index < len(job["dates"]) else f"T+{index + 1}"
        item = {
            "date": date,
            "value": value,
            "series_id": job["series_id"],
        }
        for key in ("q10", "q50", "q90"):
            quantile_value = quantile_at(quantile_by_key, key, index)
            if quantile_value is not None:
                item[key] = quantile_value
        items.append(item)
    return items


def read_optional_number(value):
    if value is None:
        return None
    return read_numeric_value(value, "invalid_covariates")
