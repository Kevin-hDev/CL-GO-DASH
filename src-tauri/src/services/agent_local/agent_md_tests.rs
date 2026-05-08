use super::*;
use tempfile::TempDir;

// ─── Tests existants (adaptés à la nouvelle signature) ───────────────────────

#[tokio::test]
async fn load_global_only() {
    let tmp = TempDir::new().unwrap();
    let global = tmp.path().join("AGENT.md");
    tokio::fs::write(&global, "# Global rules\nBe concise.").await.unwrap();

    let result = load_agent_md_from(tmp.path(), None).await;
    assert!(result.is_some());
    let content = result.unwrap();
    assert!(content.contains("Global rules"));
    assert!(content.contains("global instructions"));
}

#[tokio::test]
async fn load_global_and_project() {
    let tmp = TempDir::new().unwrap();
    let global = tmp.path().join("AGENT.md");
    tokio::fs::write(&global, "Global stuff").await.unwrap();

    let proj = TempDir::new().unwrap();
    let proj_md = proj.path().join("AGENT.md");
    tokio::fs::write(&proj_md, "Project stuff").await.unwrap();

    let result = load_agent_md_from(tmp.path(), Some(proj.path())).await;
    let content = result.unwrap();
    assert!(content.contains("Global stuff"));
    assert!(content.contains("Project stuff"));
    assert!(content.contains("project instructions"));
    let global_pos = content.find("Global stuff").unwrap();
    let project_pos = content.find("Project stuff").unwrap();
    assert!(project_pos > global_pos);
}

#[tokio::test]
async fn no_files_returns_none() {
    let tmp = TempDir::new().unwrap();
    let result = load_agent_md_from(tmp.path(), Some(tmp.path())).await;
    assert!(result.is_none());
}

#[tokio::test]
async fn project_only_no_global() {
    let tmp = TempDir::new().unwrap();
    let proj = TempDir::new().unwrap();
    let proj_md = proj.path().join("AGENT.md");
    tokio::fs::write(&proj_md, "Project only").await.unwrap();

    let result = load_agent_md_from(tmp.path(), Some(proj.path())).await;
    let content = result.unwrap();
    assert!(content.contains("Project only"));
    assert!(!content.contains("global instructions"));
}

// ─── Nouveaux tests ───────────────────────────────────────────────────────────

/// Arborescence tmp/parent/child avec AGENT.md à chaque niveau.
/// Vérifie que l'ordre est : parent avant child (plus éloigné → plus proche).
#[tokio::test]
async fn test_agent_md_hierarchy() {
    let root = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();

    // Crée parent/child
    let parent = root.path().join("parent");
    let child = parent.join("child");
    tokio::fs::create_dir_all(&child).await.unwrap();

    tokio::fs::write(parent.join("AGENT.md"), "Parent rules").await.unwrap();
    tokio::fs::write(child.join("AGENT.md"), "Child rules").await.unwrap();

    let result = load_agent_md_from(data_dir.path(), Some(&child)).await;
    let content = result.unwrap();

    assert!(content.contains("Parent rules"), "Parent rules manquant");
    assert!(content.contains("Child rules"), "Child rules manquant");

    // Parent doit apparaître AVANT child (plus éloigné = chargé avant)
    let parent_pos = content.find("Parent rules").unwrap();
    let child_pos = content.find("Child rules").unwrap();
    assert!(
        parent_pos < child_pos,
        "Parent ({parent_pos}) doit précéder Child ({child_pos})"
    );
}

/// 15 niveaux de profondeur → on s'arrête à MAX_DEPTH (10).
#[tokio::test]
async fn test_agent_md_max_depth() {
    let root = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();

    // Construit 15 niveaux : root/l1/l2/.../l15
    let mut current = root.path().to_path_buf();
    let mut dirs: Vec<PathBuf> = vec![current.clone()];
    for i in 1..=15 {
        current = current.join(format!("l{i}"));
        tokio::fs::create_dir_all(&current).await.unwrap();
        dirs.push(current.clone());
    }

    // Écrit AGENT.md à chaque niveau avec un marqueur unique
    for (i, dir) in dirs.iter().enumerate() {
        tokio::fs::write(dir.join("AGENT.md"), format!("Level {i}")).await.unwrap();
    }

    let deepest = &dirs[15]; // niveau 15
    let result = load_agent_md_from(data_dir.path(), Some(deepest)).await;
    let content = result.unwrap();

    // Niveau 15 (cwd) doit être présent
    assert!(content.contains("Level 15"), "Level 15 absent");

    // Niveau 15 - MAX_DEPTH = niveau 5 doit être présent (on remonte 10 depuis 15 → niveau 5)
    assert!(content.contains("Level 5"), "Level 5 absent");

    // Niveau 0 (racine) ne doit PAS être présent (trop loin, > 10 niveaux)
    assert!(
        !content.contains("Level 0"),
        "Level 0 ne devrait pas être présent (> MAX_DEPTH)"
    );
}

/// .cl-go/AGENT.md est chargé depuis le répertoire cwd.
#[tokio::test]
async fn test_agent_md_cl_go_dir() {
    let root = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();

    let cl_go = root.path().join(".cl-go");
    tokio::fs::create_dir_all(&cl_go).await.unwrap();
    tokio::fs::write(cl_go.join("AGENT.md"), "ClGo specific rules").await.unwrap();

    let result = load_agent_md_from(data_dir.path(), Some(root.path())).await;
    let content = result.unwrap();
    assert!(content.contains("ClGo specific rules"), ".cl-go/AGENT.md non chargé");
    assert!(content.contains("project instructions"));
}

/// .cl-go/rules/*.md sont chargés et triés alphabétiquement.
#[tokio::test]
async fn test_agent_md_rules_dir() {
    let root = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();

    let rules = root.path().join(".cl-go").join("rules");
    tokio::fs::create_dir_all(&rules).await.unwrap();

    // Écrit dans un ordre non-alpha intentionnel
    tokio::fs::write(rules.join("c-rule.md"), "Rule C").await.unwrap();
    tokio::fs::write(rules.join("a-rule.md"), "Rule A").await.unwrap();
    tokio::fs::write(rules.join("b-rule.md"), "Rule B").await.unwrap();

    let result = load_agent_md_from(data_dir.path(), Some(root.path())).await;
    let content = result.unwrap();

    assert!(content.contains("Rule A"), "Rule A absente");
    assert!(content.contains("Rule B"), "Rule B absente");
    assert!(content.contains("Rule C"), "Rule C absente");

    // Ordre alphabétique : A < B < C
    let pos_a = content.find("Rule A").unwrap();
    let pos_b = content.find("Rule B").unwrap();
    let pos_c = content.find("Rule C").unwrap();
    assert!(pos_a < pos_b, "Rule A doit précéder Rule B");
    assert!(pos_b < pos_c, "Rule B doit précéder Rule C");

    assert!(content.contains("project rules"));
}
