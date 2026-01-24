use std::io::{ Write, stdout };
use crossterm::cursor::{ MoveTo };
use crossterm::execute;
use crossterm::terminal::{ Clear, ClearType };
use crossterm::style::{ Color, ResetColor, SetBackgroundColor, SetForegroundColor };

use crate::CursorPos;
use crate::PopupMode;
use crate::popup::draw_popup;

pub fn draw(
  lines: &Vec<String>,
  cursor: &CursorPos,
  selection_start: Option<CursorPos>,
  scroll_x: usize,
  scroll_y: usize,
  ui_lines: usize,
  popup: &Option<PopupMode>,
  popup_input: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let (term_width, term_height) = crossterm::terminal::size()?;
  let term_width = term_width as usize;
  let term_height = term_height as usize;

  execute!(stdout(), Clear(ClearType::All))?;

  execute!(stdout(), MoveTo(0, 0))?;

  let text: &str = "camelCase Editor  -  ctrl + H ayuda";
  let left_padding: usize = (term_width.saturating_sub(text.len())) / 2;
  let right_padding: usize = term_width.saturating_sub(text.len()).saturating_sub(left_padding);

  execute!(stdout(),SetForegroundColor(Color::Black),SetBackgroundColor(Color::Red))?;
  print!("{}{}{}", " ".repeat(left_padding), text, " ".repeat(right_padding));
  execute!(stdout(), ResetColor)?;

  // línea vacía debajo
  execute!(stdout(), MoveTo(0, 1))?;
  print!("");

  let visible = term_height.saturating_sub(ui_lines + 1);
  let usable_width = term_width.saturating_sub(2);

  for screen_y in 0..visible {
    let line_idx = scroll_y + screen_y;
    let draw_y = ui_lines + screen_y;

    execute!(stdout(), MoveTo(0, draw_y as u16))?;

    if line_idx >= lines.len() {
      print!("{}{}", " ".repeat(1 + usable_width), " ");
      continue;
    }

    let line = &lines[line_idx];

    let chars: Vec<char> = line.chars().collect();
    let total_chars = chars.len();

    let left_hidden = scroll_x > 0;
    let right_hidden = scroll_x + usable_width < total_chars;

    // indicador izquierda
    if left_hidden {
      execute!(stdout(),SetForegroundColor(Color::Black),SetBackgroundColor(Color::DarkRed))?;
      print!("{}", '<' );
      execute!(stdout(), ResetColor)?;
    }
    else{
      print!("{}", ' ' );
    }

    // contenido visible
    for i in 0..usable_width {
      let char_idx = scroll_x + i;

      if char_idx >= total_chars {
        print!(" ");
        continue;
      }

      let c = chars[char_idx];

      let pos = CursorPos { x: char_idx, y: line_idx };

      if let Some(start) = selection_start.as_ref() {
        if crate::selection::is_selected(pos, *start, *cursor) {
          execute!(stdout(),SetForegroundColor(Color::Black),SetBackgroundColor(Color::DarkGrey))?;
          print!("{}", c);
          execute!(stdout(), ResetColor)?;
        } else {
          print!("{}", c);
        }
      } else {
        print!("{}", c);
      }
    }

    // indicador derecha
    if right_hidden {
      execute!(stdout(),SetForegroundColor(Color::Black),SetBackgroundColor(Color::DarkRed))?;
      print!("{}", '>' );
      execute!(stdout(), ResetColor)?;
    }
    else{
      print!("{}", ' ' );
    }
  }

  let status_y = term_height - 1;

  // limpiar toda la linea del status
  execute!(stdout(), MoveTo(0, status_y as u16))?;
  print!("{}", " ".repeat(term_width));
  execute!(stdout(), MoveTo(0, status_y as u16))?;
  
  execute!(stdout(), SetForegroundColor(Color::Black), SetBackgroundColor(Color::Red))?;
  
  let status = format!(" Linea {} | Columna {} | Scroll X:{} Y:{} ", cursor.y + 1, cursor.x + 1, scroll_x, scroll_y);
  
  let padded = format!("{:<width$}", status, width = term_width);
  print!("{}", padded);
  
  execute!(stdout(), ResetColor)?;

  if let Some(mode) = popup {
    let popup_data = mode.to_popup(popup_input);
    let _ = draw_popup(&popup_data, term_width, term_height);
  }

  if popup.is_none() {
    let screen_y = cursor.y.saturating_sub(scroll_y) + ui_lines;

    // Cursor visible dentro del viewport horizontal (+1 por el "<")
    let screen_x = cursor
      .x
      .saturating_sub(scroll_x)
      .saturating_add(1) // margen izquierdo
      .min(term_width.saturating_sub(1));

    execute!(stdout(), MoveTo(screen_x as u16, screen_y as u16))?;
  }

  stdout().flush()?;
  Ok(())
}


pub fn is_separator(c: char) -> bool {
  c == ' ' || c == '.' || c == '?' || c == '<' || c == '>' || c == '|' || c == '/' || c == '\\' || c == '"' || c == '\'' || c == ';' || c == '@'
}