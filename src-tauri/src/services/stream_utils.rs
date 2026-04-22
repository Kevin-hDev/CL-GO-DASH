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

pub fn clean_think_tags(content: &str) -> String {
    content
        .replace("<think>", "")
        .replace("</think>", "")
        .replace("/think", "")
        .replace("/no_think", "")
}
