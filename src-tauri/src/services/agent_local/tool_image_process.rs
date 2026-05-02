use crate::services::agent_local::security::{validate_read_path, validate_write_path};
use crate::services::agent_local::types_tools::ToolResult;
use image::imageops::FilterType;
use image::DynamicImage;
use serde_json::Value;
use std::path::Path;

pub async fn process_image(
    input_path: &str,
    output_path: &str,
    operations: &Value,
    working_dir: &Path,
) -> ToolResult {
    if input_path.is_empty() {
        return ToolResult::err("Le paramètre 'input_path' est requis");
    }
    if output_path.is_empty() {
        return ToolResult::err("Le paramètre 'output_path' est requis");
    }

    let resolved_in = super::tool_office_utils::resolve_path(input_path, working_dir);

    let validated_in = match validate_read_path(&resolved_in, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };

    let resolved_out = super::tool_office_utils::resolve_path(output_path, working_dir);

    let validated_out = match validate_write_path(&resolved_out) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };

    let mut img = match image::open(&validated_in) {
        Ok(i) => i,
        Err(_) => return ToolResult::err("Impossible d'ouvrir l'image"),
    };

    let ops = match super::tool_spreadsheet_write::coerce_to_array(operations) {
        Some(arr) => arr,
        None => return ToolResult::err("Le paramètre 'operations' doit être un tableau"),
    };

    let mut quality: Option<u8> = None;

    for op in &ops {
        let op_type = op["type"].as_str().unwrap_or("");
        match op_type {
            "resize" => match apply_resize(img, op) {
                Ok(i) => img = i,
                Err(e) => return e,
            },
            "crop" => match apply_crop(img, op) {
                Ok(i) => img = i,
                Err(e) => return e,
            },
            "quality" => {
                let val = op["value"].as_u64().unwrap_or(85).clamp(1, 100) as u8;
                quality = Some(val);
            }
            unknown => {
                return ToolResult::err(format!("Opération inconnue: {unknown}"));
            }
        }
    }

    let (width, height) = (img.width(), img.height());

    if let Err(e) = save_image(&img, &validated_out, quality) {
        return ToolResult::err(e);
    }

    let file_size = match std::fs::metadata(&validated_out) {
        Ok(m) => m.len(),
        Err(_) => 0,
    };

    let json = serde_json::json!({
        "output_path": validated_out.to_string_lossy(),
        "width": width,
        "height": height,
        "file_size_bytes": file_size
    });
    ToolResult::ok(json.to_string())
}

fn apply_resize(img: DynamicImage, op: &Value) -> Result<DynamicImage, ToolResult> {
    let w = op["width"].as_u64().ok_or_else(|| ToolResult::err("resize: 'width' requis"))? as u32;
    let h = op["height"].as_u64().ok_or_else(|| ToolResult::err("resize: 'height' requis"))? as u32;
    let mode = op["mode"].as_str().unwrap_or("fit");

    let resized = match mode {
        "fill" => img.resize_to_fill(w, h, FilterType::Lanczos3),
        "exact" => img.resize_exact(w, h, FilterType::Lanczos3),
        _ => img.resize(w, h, FilterType::Lanczos3),
    };
    Ok(resized)
}

fn apply_crop(img: DynamicImage, op: &Value) -> Result<DynamicImage, ToolResult> {
    let x = op["x"].as_u64().ok_or_else(|| ToolResult::err("crop: 'x' requis"))? as u32;
    let y = op["y"].as_u64().ok_or_else(|| ToolResult::err("crop: 'y' requis"))? as u32;
    let w = op["width"].as_u64().ok_or_else(|| ToolResult::err("crop: 'width' requis"))? as u32;
    let h = op["height"].as_u64().ok_or_else(|| ToolResult::err("crop: 'height' requis"))? as u32;

    Ok(img.crop_imm(x, y, w, h))
}

fn save_image(img: &DynamicImage, path: &Path, quality: Option<u8>) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if let Some(q) = quality {
        match ext.as_str() {
            "jpg" | "jpeg" => {
                let file = std::fs::File::create(path)
                    .map_err(|_| "Impossible de créer le fichier de sortie")?;
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, q);
                img.write_with_encoder(encoder)
                    .map_err(|_| "Erreur encodage JPEG")?;
                return Ok(());
            }
            "webp" => {
                // image 0.25 supporte uniquement WebP lossless — quality ignorée
                let file = std::fs::File::create(path)
                    .map_err(|_| "Impossible de créer le fichier de sortie")?;
                let encoder = image::codecs::webp::WebPEncoder::new_lossless(file);
                img.write_with_encoder(encoder)
                    .map_err(|_| "Erreur encodage WebP")?;
                return Ok(());
            }
            _ => {}
        }
    }

    img.save(path).map_err(|_| "Erreur lors de la sauvegarde de l'image".to_string())
}
