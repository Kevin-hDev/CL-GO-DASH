use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GitDiffPreview {
    pub hunks: Vec<GitDiffHunk>,
    pub truncated: bool,
    pub binary: bool,
}

#[derive(Debug, Serialize)]
pub struct GitDiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<GitDiffLine>,
}

#[derive(Debug, Serialize)]
pub struct GitDiffLine {
    pub kind: &'static str,
    pub content: String,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
}
