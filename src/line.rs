use history;

struct AnsiEsc {
	csi: String,
	n: String,
}
pub struct Line {
	pub line: Vec<char>,
	cursor_pos: usize,
	esc_seq: AnsiEsc,
	pub history: history::History,
}
impl Line {
	pub fn new() -> Line {
		Line {
			line: Vec::new(),
			cursor_pos: 0,
			esc_seq: AnsiEsc { csi: String::new(), n: String::new() },
			history: history::History::new(),
		}
	}
	fn push(&mut self, chara: char) {
		self.line.insert(self.cursor_pos, chara);
		self.cursor_pos += 1;

		let mut reprint = String::new();

		reprint.push(chara);
		for charac in self.line[self.cursor_pos..self.line.len()].iter() {
			reprint.push(*charac);
		}
		for _ in self.cursor_pos..self.line.len() {
			reprint.push('\u{8}');
		}

		print_flush!("{}", reprint);
	}
	fn del_prev_char(&mut self) {//TODO: check wrapping to erase previous line
		if self.cursor_pos != 0 {
			self.cursor_pos -= 1;

			if self.cursor_pos == self.line.len() - 1 {
				print_flush!("\u{8} \u{8}");
				self.line.pop();
			} else {
				self.line.remove(self.cursor_pos);

				let mut eraser = '\u{8}'.to_string();

				for chara in self.line[self.cursor_pos..self.line.len()].iter() {
					eraser.push(*chara);
				}
				eraser.push(' ');
				for _ in self.cursor_pos..self.line.len()+1 {
					eraser.push('\u{8}');
				}

				print_flush!("{}", eraser);
			}
		}
	}
	pub fn clear(&mut self) {
		self.line.clear();
		self.cursor_pos = 0;
	}
	fn move_cursor_left(&mut self, seq: &str) {
		if self.cursor_pos != 0 {
			print_flush!("{}", seq);
			self.cursor_pos -= 1;
		}
	}
	fn move_cursor_right(&mut self, seq: &str) {
		if self.cursor_pos != self.line.len() {
			print_flush!("{}", seq);
			self.cursor_pos += 1;
		}
	}
	fn redraw(&self) {
		use std::io;
		use std::io::Write;

		print!("\u{1b}[2K\u{D}");//clear line and carriage return
		print_prompt();
		for chara in &self.line {
			print!("{}", chara);
			io::stdout().flush().ok();
		}
	}
	pub fn append(&mut self, chara: char) {
		if !self.esc_seq.csi.is_empty() {
			if self.esc_seq.csi == "\u{1b}" && chara == '[' {
				self.esc_seq.csi.push(chara);
			} else if self.esc_seq.csi == "\u{1b}[" || self.esc_seq.csi == "\u{9b}" {
				if chara >= '0' && chara <= '9' {
					self.esc_seq.n.push(chara);
				} else {
					match chara {
						'A' => {
							self.history.previous();
							if let Some(hist_line) = self.history.get_line() {//get_line returns None if history.history.len() == 0
								self.line = hist_line.chars().collect();
								self.cursor_pos = self.line.len();
								self.redraw();
							}
						},
						'B' => {
							self.history.next();
							if let Some(hist_line) = self.history.get_line() {
								self.line = hist_line.chars().collect();
								self.cursor_pos = self.line.len();
							} else {
								self.line.clear();
								self.cursor_pos = 0;
								//TODO: make redrawing more efficient in this case
							}
							self.redraw();
						},
						'C' => {
							let seq = format!("{}{}C", self.esc_seq.csi, self.esc_seq.n);
							self.move_cursor_right(&seq);
						},
						'D' => {
							let seq = format!("{}{}D", self.esc_seq.csi, self.esc_seq.n);
							self.move_cursor_left(&seq);
						},
						_ => {},
					}
					self.esc_seq.csi.clear();
					self.esc_seq.n.clear();
				}

			}
		} else if chara.is_control() {
			if chara == '\u{7f}' {
				self.del_prev_char();
			} else if chara == '\u{1b}' || chara == '\u{9b}' {
				self.esc_seq.csi = chara.to_string();
			}
		} else {
			self.push(chara);
		}
	}
}

pub fn print_prompt() {//TODO: let the user configure it
	print_flush!("> ");
}
