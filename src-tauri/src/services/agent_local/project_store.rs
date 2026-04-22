use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub order: usize,
    pub created_at: DateTime<Utc>,
}

fn projects_path() -> PathBuf {
    crate::services::paths::data_dir().join("projects.json")
}

async fn read_all() -> Vec<Project> {
    let path = projects_path();
    let data = match tokio::fs::read_to_string(&path).await {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str(&data).unwrap_or_default()
}

async fn write_atomic(projects: &[Project]) -> Result<(), String> {
    let path = projects_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    let data =
        serde_json::to_string_pretty(projects).map_err(|e| format!("Serialize: {e}"))?;
    tokio::fs::write(&tmp, &data)
        .await
        .map_err(|e| format!("Write tmp: {e}"))?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| format!("Rename: {e}"))?;
    Ok(())
}

pub async fn list() -> Result<Vec<Project>, String> {
    let mut projects = read_all().await;
    projects.sort_by_key(|p| p.order);
    Ok(projects)
}

pub async fn add(path: &str) -> Result<Project, String> {
    let mut projects = read_all().await;
    if projects.iter().any(|p| p.path == path) {
        return Err("Ce dossier est déjà ajouté".to_string());
    }
    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Projet")
        .to_string();
    let project = Project {
        id: Uuid::new_v4().to_string(),
        name,
        path: path.to_string(),
        order: projects.len(),
        created_at: Utc::now(),
    };
    projects.push(project.clone());
    write_atomic(&projects).await?;
    Ok(project)
}

pub async fn rename(id: &str, name: &str) -> Result<(), String> {
    let mut projects = read_all().await;
    let p = projects
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or("Projet introuvable")?;
    p.name = name.to_string();
    write_atomic(&projects).await
}

pub async fn delete(id: &str) -> Result<(), String> {
    let mut projects = read_all().await;
    projects.retain(|p| p.id != id);
    for (i, p) in projects.iter_mut().enumerate() {
        p.order = i;
    }
    write_atomic(&projects).await
}

pub async fn reorder(ids: Vec<String>) -> Result<(), String> {
    let mut projects = read_all().await;
    for (i, id) in ids.iter().enumerate() {
        if let Some(p) = projects.iter_mut().find(|p| &p.id == id) {
            p.order = i;
        }
    }
    projects.sort_by_key(|p| p.order);
    write_atomic(&projects).await
}
