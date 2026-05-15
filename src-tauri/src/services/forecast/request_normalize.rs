use super::types::ForecastRequest;

pub fn normalize_request(request: &mut ForecastRequest) {
    request.target_column = request.target_column.trim().to_string();
    request.date_column = request.date_column.trim().to_string();
    request.frequency = request.frequency.trim().to_string();
    request.data = normalize_optional_payload(request.data.take());
    request.file_path = normalize_optional_string(request.file_path.take());
    request.series_column = normalize_optional_string(request.series_column.take());
    request.model = normalize_optional_string(request.model.take());
    request.covariate_columns = request
        .covariate_columns
        .drain(..)
        .filter_map(|column| normalize_optional_string(Some(column)))
        .collect();
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_optional_payload(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
#[path = "request_normalize_tests.rs"]
mod tests;
