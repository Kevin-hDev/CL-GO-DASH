use std::fs::File;
use std::path::Path;

const MAX_ZIP_RATIO: u64 = 100;
const MAX_ZIP_ENTRIES: usize = 4096;
const MAX_TOTAL_UNCOMPRESSED_BYTES: u64 = 200 * 1024 * 1024;

pub const MAX_DOCX_SOURCE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_DOCX_XML_BYTES: u64 = 10 * 1024 * 1024;
pub const MAX_SPREADSHEET_SOURCE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_CSV_SOURCE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_IMAGE_SOURCE_BYTES: u64 = 50 * 1024 * 1024;

pub fn ensure_source_size(path: &Path, max_bytes: u64, label: &str) -> Result<(), String> {
    let len = std::fs::metadata(path)
        .map_err(|_| format!("{label} inaccessible"))?
        .len();
    if len > max_bytes {
        return Err(format!("{label} trop volumineux"));
    }
    Ok(())
}

pub fn validate_zip_archive(path: &Path, label: &str) -> Result<(), String> {
    let file = File::open(path).map_err(|_| format!("{label} inaccessible"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| format!("{label} invalide"))?;
    if archive.len() > MAX_ZIP_ENTRIES {
        return Err(format!("{label} contient trop d'entrées"));
    }

    let mut total_uncompressed = 0u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|_| format!("{label} invalide"))?;
        let compressed = entry.compressed_size().max(1);
        let uncompressed = entry.size();
        if uncompressed / compressed > MAX_ZIP_RATIO {
            return Err(format!("{label} compression excessive"));
        }
        total_uncompressed = total_uncompressed.saturating_add(uncompressed);
        if total_uncompressed > MAX_TOTAL_UNCOMPRESSED_BYTES {
            return Err(format!("{label} décompressé trop volumineux"));
        }
    }
    Ok(())
}

pub fn ensure_zip_entry_safe(
    entry: &zip::read::ZipFile<'_, File>,
    max_bytes: u64,
    label: &str,
) -> Result<(), String> {
    let compressed = entry.compressed_size().max(1);
    let uncompressed = entry.size();
    if uncompressed > max_bytes {
        return Err(format!("{label} trop volumineux"));
    }
    if uncompressed / compressed > MAX_ZIP_RATIO {
        return Err(format!("{label} compression excessive"));
    }
    Ok(())
}
