use std::io::{ stdout };
use crossterm::execute;
use crossterm::event::{ Event, KeyCode, KeyEventKind, KeyModifiers, read };
use crossterm::terminal::{ EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size };

mod cursor;
use cursor::CursorPos;

mod draw;
use draw::draw;

mod selection;
use selection::{ start_selection_if_needed, has_selection, delete_selection, get_selected_text, paste_text };

mod moves;
use moves::{ move_word_left, move_word_right };

mod utils;
use utils::{ char_to_byte_idx, line_len_chars, set_windows_clipboard, get_windows_clipboard };

mod file;
use file::{ save_file, open_file, list_directory };
use std::path::Path;

mod popup;
use popup::PopupMode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  enable_raw_mode()?;

  execute!(stdout(),EnterAlternateScreen)?;

  let mut lines: Vec<String> = vec![String::new()];

  let mut popup: Option<PopupMode> = None;
  let mut popup_input: String = String::new();

  let mut cursor: CursorPos = CursorPos{ x: 0, y: 0 };
  let mut selection_start: Option<CursorPos> = None;

  let mut scroll_x: usize = 0;
  let mut scroll_y: usize = 0;

  let mut current_dir = if popup_input.is_empty() { ".".to_string() } else { popup_input.clone() };

  draw(&lines, &cursor, selection_start, scroll_x, scroll_y, 2, &popup, &popup_input, &current_dir)?;

  loop {
    if let Event::Key(key) = read()? {
      if key.kind != KeyEventKind::Press {
        continue;
      }

      if !popup.is_none() {
        match key.code {
          KeyCode::Esc => {
            popup = None;
            popup_input.clear();
          }
      
          KeyCode::Enter => {
            match popup {
              Some(PopupMode::Save { selected, ref entries, .. }) => {
                let entry = entries[selected].clone();
                let full_path = Path::new(&current_dir).join(&entry);
                let full_path_str = full_path.to_string_lossy().to_string();

                if full_path.is_dir() || entry.ends_with('/') || entry == ".." {
                    let next_dir = if entry == ".." {
                        let path = Path::new(&current_dir);
                        if let Ok(abs_path) = std::fs::canonicalize(path) {
                            abs_path.parent().unwrap_or(Path::new(".")).to_string_lossy().to_string()
                        } else {
                            path.parent().unwrap_or(Path::new(".")).to_string_lossy().to_string()
                        }
                    } else {
                        full_path_str
                    };

                    match list_directory(&next_dir) {
                        Ok(new_entries) => {
                            current_dir = next_dir;
                            popup = Some(PopupMode::Save { selected: 0, entries: new_entries, scroll_y: 0 });
                        }
                        Err(e) => eprintln!("Error al enlistar -> {}", e),
                    }
                } else {
                    // Si el input está vacío, intentar guardar con el nombre seleccionado
                    let save_path = if popup_input.is_empty() {
                        full_path_str
                    } else {
                        Path::new(&current_dir).join(&popup_input).to_string_lossy().to_string()
                    };

                    if let Err(e) = save_file(&save_path, &lines) {
                        eprintln!("Error al guardar -> {}", e);
                    }
                    popup = None;
                    popup_input.clear();
                }
              }
          
              Some(PopupMode::Open { selected, ref entries, .. }) => {
                let entry = entries[selected].clone();
                let full_path = Path::new(&current_dir).join(&entry);
                let full_path_str = full_path.to_string_lossy().to_string();

                if full_path.is_dir() || entry.ends_with('/') || entry == ".." {
                    let next_dir = if entry == ".." {
                        let path = Path::new(&current_dir);
                        if let Ok(abs_path) = std::fs::canonicalize(path) {
                            abs_path.parent().unwrap_or(Path::new(".")).to_string_lossy().to_string()
                        } else {
                            path.parent().unwrap_or(Path::new(".")).to_string_lossy().to_string()
                        }
                    } else {
                        full_path_str
                    };

                    match list_directory(&next_dir) {
                        Ok(new_entries) => {
                            current_dir = next_dir;
                            popup = Some(PopupMode::Open { selected: 0, entries: new_entries, scroll_y: 0 });
                        }
                        Err(e) => eprintln!("Error al enlistar -> {}", e),
                    }
                } else {
                    match open_file(&full_path_str) {
                        Ok(new_lines) => {
                          lines = new_lines;
                          cursor = CursorPos { x: 0, y: 0 };
                          scroll_x = 0;
                          scroll_y = 0;
                          selection_start = None;
                          popup = None;
                          popup_input.clear();
                          if let Some(parent) = Path::new(&full_path_str).parent() {
                              current_dir = parent.to_string_lossy().to_string();
                          }
                        }
                        Err(e) => eprintln!("Error al abrir -> {}", e),
                    }
                }
              }
          
              Some(PopupMode::Help) => {
                popup = None;
                popup_input.clear();
              }
          
              None => {}
            }
          }

          KeyCode::Up => {
            if let Some(mode) = &mut popup {
                match mode {
                    PopupMode::Open { selected, entries, scroll_y } | 
                    PopupMode::Save { selected, entries, scroll_y } => {
                        if *selected > 0 {
                            *selected -= 1;
                        } else {
                            *selected = entries.len().saturating_sub(1);
                        }
                        
                        let max_visible = 6;
                        if *selected < *scroll_y {
                            *scroll_y = *selected;
                        } else if *selected >= *scroll_y + max_visible {
                            *scroll_y = *selected - (max_visible - 1);
                        }
                    }
                    _ => {}
                }
            }
          }

          KeyCode::Down => {
            if let Some(mode) = &mut popup {
                match mode {
                    PopupMode::Open { selected, entries, scroll_y } | 
                    PopupMode::Save { selected, entries, scroll_y } => {
                        if *selected + 1 < entries.len() {
                            *selected += 1;
                        } else {
                            *selected = 0;
                        }

                        let max_visible = 6;
                        if *selected < *scroll_y {
                            *scroll_y = *selected;
                        } else if *selected >= *scroll_y + max_visible {
                            *scroll_y = *selected - (max_visible - 1);
                        }
                    }
                    _ => {}
                }
            }
          }
      
          KeyCode::Backspace => {
            popup_input.pop();
          }
      
          KeyCode::Char(c) => {
            popup_input.push(c);
          }
      
          _ => {}
        }
      
        draw(
          &lines,
          &cursor,
          selection_start,
          scroll_x,
          scroll_y,
          2,
          &popup,
          &popup_input,
          &current_dir
        )?;
        continue;
      }

      match key.code {
        // salir
        KeyCode::Char('q') 
          if key.modifiers.contains(KeyModifiers::CONTROL)
          /* && key.modifiers.contains(KeyModifiers::ALT) */ => {
          break;
        },

        // ayuda
        KeyCode::Char('h')
          if key.modifiers.contains(KeyModifiers::CONTROL)
          /* && key.modifiers.contains(KeyModifiers::ALT) */ => {
            // show_help = !show_help;
            popup = Some(PopupMode::Help);
            popup_input.clear();
          }
        
        // copiar
        KeyCode::Char('c') 
          if key.modifiers.contains(KeyModifiers::CONTROL)
          /* && key.modifiers.contains(KeyModifiers::ALT) */ =>  {
            if let Some(text) = get_selected_text(&lines, &cursor, &selection_start) {
              set_windows_clipboard(text);
            }
        },
      
        // cortar
        KeyCode::Char('x') 
          if key.modifiers.contains(KeyModifiers::CONTROL)
          /* && key.modifiers.contains(KeyModifiers::ALT) */ => {
            if let Some(text) = get_selected_text(&lines, &cursor, &selection_start) {
              set_windows_clipboard(text);
              delete_selection(&mut lines, &mut cursor, &mut selection_start);
            }
        },
      
        // pegar
        KeyCode::Char('v') 
          if key.modifiers.contains(KeyModifiers::CONTROL)
          /* && key.modifiers.contains(KeyModifiers::ALT) */ => {
            if let Some(text) = get_windows_clipboard() {
              paste_text(&mut lines, &mut cursor, &mut selection_start, &text);
            }
        },

        // mostrar guardado
        KeyCode::Char('s')
          if key.modifiers.contains(KeyModifiers::CONTROL)
          /* && key.modifiers.contains(KeyModifiers::ALT) */ =>
        {
          match list_directory(&current_dir) {
              Ok(entries) => {
                  popup = Some(PopupMode::Save { selected: 0, entries, scroll_y: 0 });
                  popup_input.clear();
              }
              Err(e) => eprintln!("Error al listar directorio -> {}", e),
          }
        }

        // abrir
        KeyCode::Char('o')
          if key.modifiers.contains(KeyModifiers::CONTROL)
          /* && key.modifiers.contains(KeyModifiers::ALT) */ => {
            match list_directory(&current_dir) {
                Ok(entries) => {
                    popup = Some(PopupMode::Open { selected: 0, entries, scroll_y: 0 });
                    popup_input.clear();
                }
                Err(e) => eprintln!("Error al listar directorio -> {}", e),
            }
        },
        
        // escribir
        KeyCode::Char(c) => {
          let byte_idx = char_to_byte_idx(&lines[cursor.y], cursor.x);

          if has_selection(&selection_start, &cursor) {
            delete_selection(&mut lines, &mut cursor, &mut selection_start);
          }

          lines[cursor.y].insert(byte_idx, c);
          cursor.x += 1;
        },

        // izquierda
        KeyCode::Left => {
          let selecting = key.modifiers.contains(KeyModifiers::SHIFT);
          
          if selecting {
            start_selection_if_needed(&mut selection_start, CursorPos { x: cursor.x, y: cursor.y });
          }
          else {
            selection_start = None;
          }

          if cursor.x > 0 {
            cursor.x -= 1;
          }
          else if cursor.y > 0 {
            cursor.y -= 1;
            cursor.x = line_len_chars(&lines[cursor.y]);
          }

          if key.modifiers.contains(KeyModifiers::CONTROL){
            move_word_left(&lines, &mut cursor.x, cursor.y);
          }
        },

        // derecha
        KeyCode::Right => {
          let selecting = key.modifiers.contains(KeyModifiers::SHIFT);

          if selecting {
            start_selection_if_needed(&mut selection_start, CursorPos { x: cursor.x, y: cursor.y });
          }
          else{ 
            selection_start = None;
          }

          if cursor.x < line_len_chars(&lines[cursor.y]) {
            cursor.x += 1;
          }
          else if cursor.y + 1 < lines.len() {
            cursor.y += 1;
            cursor.x = 0;
          }
          if key.modifiers.contains(KeyModifiers::CONTROL){
            move_word_right(&lines, &mut cursor.x, cursor.y);
          }
        },

        // subir
        KeyCode::Up => {
          let selecting = key.modifiers.contains(KeyModifiers::SHIFT);
          
          if selecting {
            start_selection_if_needed(&mut selection_start, CursorPos { x: cursor.x, y: cursor.y });
          }
          else {
            selection_start = None;
          }

          if cursor.y > 0 {
            cursor.y -= 1;
            cursor.x = cursor.x.min(line_len_chars(&lines[cursor.y]));
          }
        },

        // bajar
        KeyCode::Down => {
          let selecting = key.modifiers.contains(KeyModifiers::SHIFT);
          
          if selecting {
            start_selection_if_needed(&mut selection_start, CursorPos { x: cursor.x, y: cursor.y });
          }
          else {
            selection_start = None;
          }

          if cursor.y + 1 < lines.len() {
            cursor.y += 1;
            cursor.x = cursor.x.min(line_len_chars(&lines[cursor.y]));
          }
        },

        // inicio
        KeyCode::Home => {
          let selecting = key.modifiers.contains(KeyModifiers::SHIFT);

          if selecting {
            if key.modifiers.contains(KeyModifiers::CONTROL){
              cursor.y = 0;
              cursor.x = 0;
            }

            start_selection_if_needed(&mut selection_start, CursorPos { x: cursor.x, y: cursor.y });
          }
          else{ 
            selection_start = None;
          }

          if key.modifiers.contains(KeyModifiers::CONTROL){
            cursor.y = 0;
          }
          cursor.x = 0;
        },

        // final
        KeyCode::End => {
          let selecting = key.modifiers.contains(KeyModifiers::SHIFT);

          if selecting {
            start_selection_if_needed(&mut selection_start, CursorPos { x: cursor.x, y: cursor.y });
          }
          else{ 
            selection_start = None;
          }

          if key.modifiers.contains(KeyModifiers::CONTROL){
            cursor.y = lines.len() - 1;
          }
          cursor.x = line_len_chars(&lines[cursor.y]);
        },

        // enter
        KeyCode::Enter => {
          let byte_idx = char_to_byte_idx(&lines[cursor.y], cursor.x);
          let new_line = lines[cursor.y].split_off(byte_idx);

          lines.insert(cursor.y + 1, new_line);
          cursor.y += 1;
          cursor.x = 0;
        },

        // backspace
        KeyCode::Backspace => {
          if has_selection(&selection_start, &cursor) {
            delete_selection(&mut lines, &mut cursor, &mut selection_start);
          } else if cursor.x > 0 {
            let byte_idx = char_to_byte_idx(&lines[cursor.y], cursor.x - 1);

            lines[cursor.y].remove(byte_idx);
            cursor.x -= 1;
          } else if cursor.y > 0 {
            let current = lines.remove(cursor.y);
            
            cursor.y -= 1;
            cursor.x = line_len_chars(&lines[cursor.y]);
            lines[cursor.y].push_str(&current);
          }
        },
        _ => {}
      }

      let (term_width, term_height) = size()?;
      let visible_lines = term_height as usize - (2 + 1);

      let usable_width = (term_width as usize).saturating_sub(2);
      
      if cursor.x < scroll_x {
        scroll_x = cursor.x;
      }
      
      if cursor.x >= scroll_x + usable_width {
        scroll_x = cursor.x + 1 - usable_width;
      }

      if cursor.y < scroll_y {
        scroll_y = cursor.y;
      }

      if cursor.y >= scroll_y + visible_lines {
        scroll_y = cursor.y + 1 - visible_lines;
      }

      draw(&lines, &cursor, selection_start, scroll_x, scroll_y, 2, &popup, &popup_input, &current_dir)?;
    }
  }

  execute!(stdout(),LeaveAlternateScreen)?;

  disable_raw_mode()?;
  Ok(())
}