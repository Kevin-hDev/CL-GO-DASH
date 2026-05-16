use super::common::{rows, ExportBundle};
use std::path::Path;

pub fn write(bundle: &ExportBundle, path: &Path) -> Result<(), String> {
    let mut writer =
        csv::Writer::from_path(path).map_err(|_| "Export CSV impossible".to_string())?;
    writer
        .write_record([
            "section",
            "name",
            "date",
            "series_id",
            "value",
            "q10",
            "q50",
            "q90",
            "text",
            "source",
        ])
        .map_err(|_| "Export CSV impossible".to_string())?;
    for row in rows(bundle) {
        writer
            .write_record([
                row.section,
                row.name,
                row.date,
                row.series_id,
                row.value,
                row.q10,
                row.q50,
                row.q90,
                row.text,
                row.source,
            ])
            .map_err(|_| "Export CSV impossible".to_string())?;
    }
    writer
        .flush()
        .map_err(|_| "Export CSV impossible".to_string())
}
