use std::fs;
use std::io;

pub fn save_file(path: &str, lines: &Vec<String>) -> io::Result<()> {
  let content = lines.join("\n");
  fs::write(path, content)
}

pub fn open_file(path: &str) -> io::Result<Vec<String>> {
  let content = fs::read_to_string(path)?;
  Ok(content.lines().map(|l| l.to_string()).collect())
}