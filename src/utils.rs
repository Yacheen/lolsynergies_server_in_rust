pub fn parse_username(s: &mut String) -> String {
    s.trim_start().trim_end().to_lowercase().chars().filter(|c| !c.is_whitespace()).collect::<String>()
}