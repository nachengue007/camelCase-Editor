use std::io::stdout;

use crossterm::{cursor::MoveTo, execute, style::{Color, ResetColor, SetForegroundColor, SetBackgroundColor}};

#[derive(Clone)]
pub struct Popup {
  pub title: String,
  pub lines: Vec<String>,
  pub footer: String,
  pub width: usize,
  pub height: usize,
  pub selected_line: Option<usize>,
  pub scroll: usize,
}

#[derive(Clone)]
pub enum PopupMode {
  Save { selected: usize, entries: Vec<String>, scroll_y: usize },
  Open { selected: usize, entries: Vec<String>, scroll_y: usize },
  Help,
}

impl Popup {
  pub fn help() -> Self {
    let help_lines = vec![
      "Ctrl + Q -> Salir",
      "Ctrl + C -> Copiar",
      "Ctrl + X -> Cortar",
      "Ctrl + V -> Pegar",
      "Ctrl + S -> Guardar archivo",
      "Ctrl + O -> Abrir archivo",
      "Shift + Flechas -> Seleccionar",
      "Ctrl + Flechas -> Mover por palabra",
      "Inicio/Home -> Mover al principio de la linea",
      "Fin/End -> Mover al final de la linea",
      "Ctrl + Inicio/Home -> Mover al principio del documento",
      "Ctrl + Fin/End -> Mover al final del documento",
      "Esc -> Cerrar ayuda",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<_>>();

    let box_width: usize = 64;
    let box_height: usize = help_lines.len() + 6;

    Popup {
      title: "Atajos del teclado".to_string(),
      lines: help_lines,
      footer: "Creado por Ignacio Fonseca".to_string(),
      width: box_width,
      height: box_height,
      selected_line: None,
      scroll: 0,
    }
  }
}

impl PopupMode {
  pub fn to_popup(&self, input: &str) -> Popup {
    match self {
      PopupMode::Save { selected, entries, scroll_y } => {
          let mut lines = vec![format!("Guardar como: {}", input)];
          lines.push(format!("Directorio: .")); // simplified for now or use actual path
          
          let max_visible = 5; 
          let visible_entries = entries.iter()
              .skip(*scroll_y)
              .take(max_visible)
              .cloned();

          lines.extend(visible_entries);

          Popup {
              title: "Guardar archivo".to_string(),
              lines,
              footer: format!("Enter = Guardar   Esc = Cancelar"),
              width: 50,
              height: 12,
              selected_line: Some(selected.saturating_sub(*scroll_y) + 2), // +2 because of title and dir header
              scroll: *scroll_y,
          }
      }

      PopupMode::Open { selected, entries, scroll_y } => {
          let mut lines = vec![format!("Directorio: {}", input)];
          
          let max_visible = 6;
          let visible_entries = entries.iter()
              .skip(*scroll_y)
              .take(max_visible)
              .cloned();

          lines.extend(visible_entries);

          Popup {
              title: "Abrir archivo".to_string(),
              lines,
              footer: format!("{} de {} - Esc = Salir", *selected + 1, entries.len()),
              width: 50,
              height: 12,
              selected_line: Some(selected.saturating_sub(*scroll_y) + 1),
              scroll: *scroll_y,
          }
      }

      PopupMode::Help => {
          let mut p = Popup::help();
          p.selected_line = None;
          p
      }
    }
  }
}


pub fn draw_popup(
  popup: &Popup,
  term_width: usize,
  term_height: usize,
) -> std::io::Result<()> {
  let start_x = (term_width.saturating_sub(popup.width)) / 2;
  let start_y = (term_height.saturating_sub(popup.height)) / 2;

  let inner_width = popup.width - 2;

  // fondo + laterales
  for y in 0..popup.height {
    execute!(
      stdout(),
      MoveTo(start_x as u16, (start_y + y) as u16),
      SetForegroundColor(Color::DarkRed)
    )?;
    print!("|");
    execute!(stdout(), ResetColor)?;
    print!("{}", " ".repeat(inner_width));
    execute!(stdout(), SetForegroundColor(Color::DarkRed))?;
    print!("|");
    execute!(stdout(), ResetColor)?;
  }

  // borde superior
  execute!(
    stdout(),
    MoveTo(start_x as u16, start_y as u16),
    SetForegroundColor(Color::DarkRed)
  )?;
  print!("┌{}┐", "─".repeat(inner_width));

  // borde inferior
  execute!(
    stdout(),
    MoveTo(start_x as u16, (start_y + popup.height - 1) as u16)
  )?;
  print!("└{}┘", "─".repeat(inner_width));
  execute!(stdout(), ResetColor)?;

  // === TÍTULO (centrado) ===
  let title_x = start_x + 1 + (inner_width.saturating_sub(popup.title.len())) / 2;
  execute!(
    stdout(),
    MoveTo(title_x as u16, (start_y + 1) as u16)
  )?;
  print!("{}", popup.title);

  // === LÍNEAS ===
  let content_start_y = start_y + 3; // título + espacio
  for (i, line) in popup.lines.iter().enumerate() {
    execute!(
      stdout(),
      MoveTo((start_x + 2) as u16, (content_start_y + i) as u16)
    )?;
    
    if Some(i) == popup.selected_line {
        execute!(stdout(), SetForegroundColor(Color::Black), SetBackgroundColor(Color::White))?;
        print!("{:<width$}", line, width = inner_width - 2);
        execute!(stdout(), ResetColor)?;
    } else {
        print!("{}", line);
    }
  }

  // === FOOTER (centrado) ===
  let footer_y = start_y + popup.height - 2;
  let footer_x = start_x + 1 + (inner_width.saturating_sub(popup.footer.len())) / 2;
  execute!(
    stdout(),
    MoveTo(footer_x as u16, footer_y as u16)
  )?;
  print!("{}", popup.footer);

  Ok(())
}
