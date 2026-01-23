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

fn main() -> Result<(), Box<dyn std::error::Error>> {
  enable_raw_mode()?;

  execute!(stdout(),EnterAlternateScreen)?;

  let mut lines: Vec<String> = vec![String::new()];

  let mut cursor = CursorPos{ x: 0, y: 0 };
  let mut selection_start: Option<CursorPos> = None;

  let mut scroll_x: usize = 0;
  let mut scroll_y: usize = 0;

  let mut show_help: bool = false;

  draw(&lines, &cursor, selection_start, scroll_x, scroll_y, 2, show_help)?;

  loop {
    if let Event::Key(key) = read()? {
      if key.kind != KeyEventKind::Press {
        continue;
      }

      match key.code {
        // salir
        KeyCode::Char('q') 
          if key.modifiers.contains(KeyModifiers::CONTROL)
          && key.modifiers.contains(KeyModifiers::ALT) => {
          break;
        },

        // ayuda
        KeyCode::Char('h')
          if key.modifiers.contains(KeyModifiers::CONTROL)
          && key.modifiers.contains(KeyModifiers::ALT) => {
            show_help = !show_help;
          }
        
        // copiar
        KeyCode::Char('c') 
          if key.modifiers.contains(KeyModifiers::CONTROL)
          && key.modifiers.contains(KeyModifiers::ALT) =>  {
            if let Some(text) = get_selected_text(&lines, &cursor, &selection_start) {
              set_windows_clipboard(text);
            }
        },
      
        // cortar
        KeyCode::Char('x') 
          if key.modifiers.contains(KeyModifiers::CONTROL)
          && key.modifiers.contains(KeyModifiers::ALT) => {
            if let Some(text) = get_selected_text(&lines, &cursor, &selection_start) {
              set_windows_clipboard(text);
              delete_selection(&mut lines, &mut cursor, &mut selection_start);
            }
        },
      
        // pegar
        KeyCode::Char('v') 
          if key.modifiers.contains(KeyModifiers::CONTROL)
          && key.modifiers.contains(KeyModifiers::ALT) => {
            if let Some(text) = get_windows_clipboard() {
              paste_text(&mut lines, &mut cursor, &mut selection_start, &text);
            }
        },

        // escribir
        KeyCode::Char(c) => {
          //lines[cursor.y].insert(cursor.x, c);

          let byte_idx = char_to_byte_idx(&lines[cursor.y], cursor.x);
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
          cursor.x = lines[cursor.y].len();
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
      let visible_lines = term_height as usize - 2;

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

      draw(&lines, &cursor, selection_start, scroll_x, scroll_y, 2, show_help)?;
    }
  }

  execute!(stdout(),LeaveAlternateScreen)?;

  disable_raw_mode()?;
  Ok(())
}