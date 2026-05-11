pub fn compute_tps(count: u32, first: Option<std::time::Instant>) -> f64 {
    match first {
        Some(t) => {
            let elapsed = t.elapsed().as_secs_f64();
            if elapsed > 0.1 && count > 1 {
                (count - 1) as f64 / elapsed
            } else {
                0.0
            }
        }
        None => 0.0,
    }
}

pub enum FilteredChunk {
    Content(String),
    Thinking(String),
}

pub struct ThinkTagFilter {
    in_think: bool,
    pending: String,
}

impl ThinkTagFilter {
    pub fn new() -> Self {
        Self {
            in_think: false,
            pending: String::new(),
        }
    }

    pub fn feed(&mut self, chunk: &str) -> Vec<FilteredChunk> {
        self.pending.push_str(chunk);
        let mut output = Vec::new();

        loop {
            if self.in_think {
                if let Some(pos) = self.pending.find("</think>") {
                    let thinking = self.pending[..pos].to_string();
                    if !thinking.is_empty() {
                        output.push(FilteredChunk::Thinking(thinking));
                    }
                    self.pending = self.pending[pos + 8..].to_string();
                    self.in_think = false;
                    continue;
                }
                let safe = safe_flush_len(&self.pending, "</think>");
                if safe > 0 {
                    output.push(FilteredChunk::Thinking(self.pending[..safe].to_string()));
                    self.pending = self.pending[safe..].to_string();
                }
                break;
            } else {
                if let Some(pos) = self.pending.find("<think>") {
                    let content = self.pending[..pos].to_string();
                    if !content.is_empty() {
                        output.push(FilteredChunk::Content(content));
                    }
                    self.pending = self.pending[pos + 7..].to_string();
                    self.in_think = true;
                    continue;
                }
                let safe = safe_flush_len(&self.pending, "<think>");
                if safe > 0 {
                    output.push(FilteredChunk::Content(self.pending[..safe].to_string()));
                    self.pending = self.pending[safe..].to_string();
                }
                break;
            }
        }

        output
    }

    pub fn flush(&mut self) -> Vec<FilteredChunk> {
        if self.pending.is_empty() {
            return vec![];
        }
        let text = std::mem::take(&mut self.pending);
        if self.in_think {
            vec![FilteredChunk::Thinking(text)]
        } else {
            vec![FilteredChunk::Content(text)]
        }
    }
}

fn safe_flush_len(text: &str, tag: &str) -> usize {
    let len = text.len();
    for i in (1..=tag.len().min(len)).rev() {
        if text.ends_with(&tag[..i]) {
            return len - i;
        }
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_complete_tags_single_chunk() {
        let mut f = ThinkTagFilter::new();
        let out = f.feed("<think>my thoughts</think>visible answer");
        let mut out = out.into_iter().chain(f.flush());
        match out.next().unwrap() {
            FilteredChunk::Thinking(t) => assert_eq!(t, "my thoughts"),
            _ => panic!("expected thinking"),
        }
        match out.next().unwrap() {
            FilteredChunk::Content(c) => assert_eq!(c, "visible answer"),
            _ => panic!("expected content"),
        }
    }

    #[test]
    fn filter_split_across_chunks() {
        let mut f = ThinkTagFilter::new();
        let mut all = vec![];
        all.extend(f.feed("<"));
        all.extend(f.feed("think>"));
        all.extend(f.feed("reasoning here"));
        all.extend(f.feed("</"));
        all.extend(f.feed("think>"));
        all.extend(f.feed("the answer"));
        all.extend(f.flush());

        let thinking: String = all
            .iter()
            .filter_map(|c| match c {
                FilteredChunk::Thinking(t) => Some(t.as_str()),
                _ => None,
            })
            .collect();
        let content: String = all
            .iter()
            .filter_map(|c| match c {
                FilteredChunk::Content(t) => Some(t.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(thinking, "reasoning here");
        assert_eq!(content, "the answer");
    }

    #[test]
    fn filter_no_tags() {
        let mut f = ThinkTagFilter::new();
        let out = f.feed("just normal content");
        let out: Vec<_> = out.into_iter().chain(f.flush()).collect();
        assert_eq!(out.len(), 1);
        match &out[0] {
            FilteredChunk::Content(c) => assert_eq!(c, "just normal content"),
            _ => panic!("expected content"),
        }
    }

    #[test]
    fn filter_multiple_think_blocks() {
        let mut f = ThinkTagFilter::new();
        let out = f.feed("<think>first</think>middle<think>second</think>end");
        let out: Vec<_> = out.into_iter().chain(f.flush()).collect();
        let types: Vec<&str> = out
            .iter()
            .map(|c| match c {
                FilteredChunk::Content(_) => "C",
                FilteredChunk::Thinking(_) => "T",
            })
            .collect();
        assert_eq!(types, vec!["T", "C", "T", "C"]);
    }

    #[test]
    fn filter_token_by_token() {
        let mut f = ThinkTagFilter::new();
        let tokens = [
            "<", "think", ">", "I ", "need ", "to ", "think", "</", "think", ">", "Hello",
        ];
        let mut all = vec![];
        for t in tokens {
            all.extend(f.feed(t));
        }
        all.extend(f.flush());

        let thinking: String = all
            .iter()
            .filter_map(|c| match c {
                FilteredChunk::Thinking(t) => Some(t.as_str()),
                _ => None,
            })
            .collect();
        let content: String = all
            .iter()
            .filter_map(|c| match c {
                FilteredChunk::Content(t) => Some(t.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(thinking, "I need to think");
        assert_eq!(content, "Hello");
    }
}
