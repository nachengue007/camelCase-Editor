pub fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
  s.char_indices()
    .nth(char_idx)
    .map(|(i, _)| i)
    .unwrap_or_else(|| s.len())
}

pub fn line_len_chars(s: &str) -> usize {
  s.chars().count()
}