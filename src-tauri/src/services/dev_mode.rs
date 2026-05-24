pub fn force_first_launch() -> bool {
    cfg!(debug_assertions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_build_forces_first_launch() {
        assert!(force_first_launch());
    }
}
