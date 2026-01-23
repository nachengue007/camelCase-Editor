use clipboard_win::{ set_clipboard_string, get_clipboard_string };

pub fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
  s.char_indices()
    .nth(char_idx)
    .map(|(i, _)| i)
    .unwrap_or_else(|| s.len())
}

pub fn line_len_chars(s: &str) -> usize {
  s.chars().count()
}

pub fn set_windows_clipboard(text: String) {
  let _ = set_clipboard_string(&text);
}

pub fn get_windows_clipboard() -> Option<String> {
  get_clipboard_string().ok()
}