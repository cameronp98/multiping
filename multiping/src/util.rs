pub fn reverse(s: String) -> String  {
    if s.len() <= 1 {
        return s;
    }

    s.chars().rev().collect()
}
