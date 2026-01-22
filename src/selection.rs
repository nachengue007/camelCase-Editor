use crate::CursorPos;
use crate::char_to_byte_idx;

pub fn start_selection_if_needed(selection: &mut Option<CursorPos>, cursor: CursorPos) { 
  if selection.is_none() {
    *selection = Some(cursor);
  }
}

pub fn is_selected(pos: CursorPos, start: CursorPos, end: CursorPos) -> bool {
  if start.y > end.y || (start.y == end.y && start.x > end.x) {
    return is_selected(pos, end, start);
  }

  if pos.y < start.y || pos.y > end.y {
    return false;
  }

  if pos.y == start.y && pos.x < start.x {
    return false;
  }

  if pos.y == end.y && pos.x >= end.x {
    return false;
  }

  true
}

pub fn has_selection(selection: &Option<CursorPos>, cursor: &CursorPos) -> bool {
  if let Some(start) = selection {
    start.x != cursor.x || start.y != cursor.y
  }
  else{
    false
  }
}

pub fn delete_selection(lines: &mut Vec<String>, cursor: &mut CursorPos, selection: &mut Option<CursorPos>) {
  let Some(start) = *selection else { return };
  let mut start = start;
  let mut end = *cursor;

  if start.y > end.y || (start.y == end.y && start.x > end.x) {
    std::mem::swap(&mut start, &mut end);
  }

  // mismo renglón
  if start.y == end.y {
    let line = &lines[start.y];
    let a = char_to_byte_idx(line, start.x);
    let b = char_to_byte_idx(line, end.x);
    lines[start.y].replace_range(a..b, "");
  } else {
    let line = &lines[start.y];
    let a = char_to_byte_idx(line, start.x);
    lines[start.y].replace_range(a.., "");
  
    for _ in (start.y + 1)..end.y {
      lines.remove(start.y + 1);
    }
  
    let tail = lines.remove(start.y + 1);
  
    let tail_byte = char_to_byte_idx(&tail, end.x);
    lines[start.y].push_str(&tail[tail_byte..]);
  }

  cursor.x = start.x;
  cursor.y = start.y;
  *selection = None;
}

pub fn get_selected_text(lines: &Vec<String>, cursor: &CursorPos, selection: &Option<CursorPos>) -> Option<String> {
  let Some(start) = selection else { return None };
  let mut start = *start;
  let mut end = *cursor;
  
  if start.y > end.y || (start.y == end.y && start.x > end.x) {
    std::mem::swap(&mut start, &mut end);
  }

  let mut result = String::new();

  if start.y == end.y {
    let line = &lines[start.y];
    let a = char_to_byte_idx(line, start.x);
    let b = char_to_byte_idx(line, end.x);
    result.push_str(&line[a..b]);
  } else {
    // primera línea
    {
      let line = &lines[start.y];
      let a = char_to_byte_idx(line, start.x);
      result.push_str(&line[a..]);
      result.push('\n');
    }

    // líneas intermedias
    for y in (start.y + 1)..end.y {
      result.push_str(&lines[y]);
      result.push('\n');
    }

    // última línea
    {
      let line = &lines[end.y];
      let b = char_to_byte_idx(line, end.x);
      result.push_str(&line[..b]);
    }
  }

  Some(result)
}

pub fn paste_text(lines: &mut Vec<String>, cursor: &mut CursorPos, selection: &mut Option<CursorPos>, text: &str) {
  if selection.is_some() {
    delete_selection(lines, cursor, selection);
  }

  let parts: Vec<&str> = text.split('\n').collect();

  if parts.len() == 1 {
    let byte_idx = char_to_byte_idx(&lines[cursor.y], cursor.x);
    lines[cursor.y].insert_str(byte_idx, parts[0]);
    cursor.x += parts[0].len();
  }
  else {
    let tail = lines[cursor.y].split_off(cursor.x);
    lines[cursor.y].push_str(parts[0]);

    for i in 1..parts.len() {
      lines.insert(cursor.y + i, parts[i].to_string());
    }

    let last = cursor.y + parts.len() - 1;
    lines[last].push_str(&tail);

    cursor.y = last;
    cursor.x = parts.last().unwrap().len();
  }

  *selection = None;
}