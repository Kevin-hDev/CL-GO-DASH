use crate::services::agent_local::security;
use crate::services::agent_local::types_tools::ToolResult;
use grep_regex::RegexMatcher;
use grep_searcher::{Searcher, Sink, SinkMatch};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::path::{Path, PathBuf};

const MAX_RESULTS: usize = 250;

pub async fn grep(
    pattern: &str,
    path: Option<&str>,
    glob_filter: Option<&str>,
    working_dir: &Path,
) -> ToolResult {
    let pattern = pattern.to_string();
    let glob_filter = glob_filter.map(|s| s.to_string());
    let root = resolve_root(path, working_dir);

    if let Err(e) = security::validate_read_path(&root, working_dir) {
        return ToolResult { content: e, is_error: true };
    }

    let result = tokio::task::spawn_blocking(move || grep_blocking(&pattern, &root, glob_filter.as_deref()))
        .await;

    match result {
        Ok(r) => r,
        Err(e) => ToolResult { content: format!("Erreur interne: {e}"), is_error: true },
    }
}

fn resolve_root(path: Option<&str>, working_dir: &Path) -> PathBuf {
    match path {
        Some(p) => {
            let pb = Path::new(p);
            if pb.is_absolute() { pb.to_path_buf() } else { working_dir.join(pb) }
        }
        None => working_dir.to_path_buf(),
    }
}

fn grep_blocking(pattern: &str, root: &Path, glob_filter: Option<&str>) -> ToolResult {
    let matcher = match RegexMatcher::new(pattern) {
        Ok(m) => m,
        Err(e) => return ToolResult {
            content: format!("Pattern regex invalide : {e}"),
            is_error: true,
        },
    };

    let mut walk_builder = WalkBuilder::new(root);
    walk_builder.hidden(false).git_ignore(true);

    if let Some(g) = glob_filter {
        let mut ov = OverrideBuilder::new(root);
        if let Err(e) = ov.add(g) {
            return ToolResult {
                content: format!("Glob invalide : {e}"),
                is_error: true,
            };
        }
        match ov.build() {
            Ok(overrides) => { walk_builder.overrides(overrides); }
            Err(e) => return ToolResult {
                content: format!("Glob invalide : {e}"),
                is_error: true,
            },
        }
    }

    let mut searcher = Searcher::new();
    let mut results: Vec<String> = Vec::new();
    let mut truncated = false;

    for dent in walk_builder.build() {
        if results.len() >= MAX_RESULTS {
            truncated = true;
            break;
        }
        let entry = match dent {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        let mut sink = MatchSink { path, results: &mut results, max: MAX_RESULTS };
        let _ = searcher.search_path(&matcher, path, &mut sink);
        if results.len() >= MAX_RESULTS {
            truncated = true;
            break;
        }
    }

    let mut output = results.join("\n");
    if truncated {
        output.push_str(&format!("\n... [tronqué à {MAX_RESULTS} lignes]"));
    }
    if output.is_empty() {
        output = "(aucun résultat)".into();
    }
    ToolResult { content: output, is_error: false }
}

struct MatchSink<'a> {
    path: &'a Path,
    results: &'a mut Vec<String>,
    max: usize,
}

impl<'a> Sink for MatchSink<'a> {
    type Error = std::io::Error;

    fn matched(
        &mut self,
        _searcher: &Searcher,
        mat: &SinkMatch<'_>,
    ) -> Result<bool, std::io::Error> {
        if self.results.len() >= self.max {
            return Ok(false);
        }
        let line = mat.line_number().unwrap_or(0);
        let text = String::from_utf8_lossy(mat.bytes());
        let trimmed = text.trim_end_matches(['\n', '\r']);
        self.results.push(format!("{}:{}:{}", self.path.display(), line, trimmed));
        Ok(self.results.len() < self.max)
    }
}
