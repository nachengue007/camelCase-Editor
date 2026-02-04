use std::fs;
use std::io;

pub fn save_file(path: &str, lines: &Vec<String>) -> io::Result<()> {
  let content = lines.join("\n");
  fs::write(path, content)
}

pub fn open_file(path: &str) -> io::Result<Vec<String>> {
  let content = fs::read_to_string(path)?;
  let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
  if lines.is_empty() {
      Ok(vec![String::new()])
  } else {
      Ok(lines)
  }
}

pub fn list_directory(path: &str) -> io::Result<Vec<String>> {
    let mut entries = Vec::new();
    let read_dir = fs::read_dir(path)?;

    // Agregamos ".." para subir de nivel
    entries.push("..".to_string());

    for entry in read_dir {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type()?.is_dir();
        
        if is_dir {
            entries.push(format!("{}/", name));
        } else {
            entries.push(name);
        }
    }

    Ok(entries)
}