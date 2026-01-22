pub fn move_word_left(lines: &Vec<String>, x: &mut usize, y: usize) {
  let line: &String = &lines[y];
  let chars: Vec<char> = line.chars().collect();

  if *x == 0 {
    return;
  }

  let mut i: usize = *x - 1;

  // retroceder sobre separadores
  while i > 0 && crate::draw::is_separator(chars[i]) {
    i -= 1;
  }

  // retroceder hasta el separador anterior
  while i > 0 && !crate::draw::is_separator(chars[i - 1]) {
    i -= 1;
  }

  *x = i;
}

pub fn move_word_right(lines: &Vec<String>, x: &mut usize, y: usize) {
  let line = &lines[y];
  let chars: Vec<char> = line.chars().collect();

  let mut i = *x;

  // avanzo hasta encontrar un separador
  while i < chars.len() && !crate::draw::is_separator(chars[i]) {
    i += 1;
  }

  // salto todos los separadores que me cruce
  while i < chars.len() && crate::draw::is_separator(chars[i]) {
    i += 1;
  }

  *x = i;
}