use std::io::{ Write, stdout };
use crossterm::cursor::{ MoveTo };
use crossterm::execute;
use crossterm::terminal::{ Clear, ClearType };
use crossterm::style::{ Color, ResetColor, SetBackgroundColor, SetForegroundColor };

use crate::CursorPos;

pub fn draw(
  lines: &Vec<String>,
  cursor: &CursorPos,
  selection_start: Option<CursorPos>,
  scroll_x: usize,
  scroll_y: usize,
  ui_lines: usize,
  show_help: bool,
) -> Result<(), Box<dyn std::error::Error>> {
  let (term_width, term_height) = crossterm::terminal::size()?;
  let term_width = term_width as usize;
  let term_height = term_height as usize;

  execute!(stdout(), Clear(ClearType::All))?;

  execute!(stdout(), MoveTo(0, 0))?;

  let text: &str = "camelCase Editor  -  ctrl + alt + H ayuda";
  let padding = (term_width.saturating_sub(text.len())) / 2;

  execute!(stdout(),SetForegroundColor(Color::Black),SetBackgroundColor(Color::Red))?;
  print!("{}{}{}", " ".repeat(padding), text, " ".repeat(padding));
  execute!(stdout(), ResetColor)?;

  // línea vacía debajo
  execute!(stdout(), MoveTo(0, 1))?;
  print!("");

  let visible = term_height.saturating_sub(ui_lines);
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

  if show_help {
    let help_lines = [
      "Atajos del teclado",
      "",
      "Ctrl + Alt + Q -> Salir",
      "Ctrl + Alt + C -> Copiar",
      "Ctrl + Alt + X -> Cortar",
      "Ctrl + Alt + V -> Pegar",
      "Shift + Flechas -> Seleccionar",
      "Ctrl + Flechas -> Mover por palabra",
      "Inicio/Home -> Mover al principio de la linea",
      "Fin/End -> Mover al final de la linea",
      "Ctrl + Inicio/Home -> Mover al principio del documento",
      "Ctrl + Fin/End -> Mover al final del documento",
      "Ctrl + Alt + H -> Abrir/Cerrar ayuda",
      "",
      "Creado por Ignacio Fonseca",
    ];

    let box_width: usize = 64;
    let box_height: usize = help_lines.len() + 2;

    let start_x = (term_width.saturating_sub(box_width)) / 2;
    let start_y = (term_height.saturating_sub(box_height)) / 2;

    execute!(stdout(), MoveTo(start_x as u16, start_y as u16))?;
    print!("+{}+", "-".repeat(box_width - 2));

    for (i, line) in help_lines.iter().enumerate() {
      execute!(stdout(), MoveTo(start_x as u16, (start_y + 1 + i) as u16))?;

      let mut text = line.to_string();
      if text.len() > box_width - 4 {
        text.truncate(box_width - 4);
      }

      print!("| {:<width$} |", text, width = box_width - 4);
    }

    execute!(
      stdout(),
      MoveTo(start_x as u16, (start_y + box_height - 1) as u16)
    )?;
    print!("+{}+", "-".repeat(box_width - 2));
  }

  if !show_help {
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
  c == ' ' || c == '.' || c == '?' || c == '<' || c == '>' || c == '|' || c == '/' || c == '\\' || c == '"' || c == '\''
}