use history;

#[derive(Copy, Clone)]
enum LineState {
	Start,
	Escape,
	CsiComplete,
	CollectNum(usize),
}
pub struct Line {
	pub line: Vec<char>,
	cursor_pos: usize,
	state: LineState,
	pub history: history::History,
}
impl Line {
	pub fn new() -> Line {
		Line {
			line: Vec::new(),
			cursor_pos: 0,
			state: LineState::Start,
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
	fn move_cursor_left(&mut self, step: usize) {
		if let Some(new_pos) = self.cursor_pos.checked_sub(step) {
			print_flush!("\u{1b}[{}D", step);
			self.cursor_pos = new_pos;
		}
	}
	fn move_cursor_right(&mut self, step: usize) {
		let new_pos = self.cursor_pos + step;
		if new_pos <= self.line.len() {
			print_flush!("\u{1b}[{}C", step);
			self.cursor_pos = new_pos;
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
	fn interpret_seq(&mut self, n: usize, chara: char) {
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
			'C' => self.move_cursor_right(n),
			'D' => self.move_cursor_left(n),
			'H' => {
				let step = self.cursor_pos;
				self.move_cursor_left(step);
			},
			'F' => {
				let step = self.line.len() - self.cursor_pos;
				self.move_cursor_right(step);
			},
			_ => {},
		}

		self.state = LineState::Start;
	}
	pub fn append(&mut self, chara: char) {
		use self::LineState::*;

		match (self.state, chara) {
			(Start, '\u{1b}') => self.state = Escape,
			(Start, '\u{9b}') => self.state = CsiComplete,
			(Start, '\u{7f}') => self.del_prev_char(),
			(Start, _) => self.push(chara),

			(Escape, '[') | (Escape, 'O') => self.state = CsiComplete,
			(Escape, _) => self.state = Start, // ignore lone escape character

			(CsiComplete, '0'...'9') => self.state = CollectNum(chara.to_digit(10).unwrap() as usize),
			(CsiComplete, _) => self.interpret_seq(1, chara),

			(CollectNum(ref mut n), '0'...'9') => *n = *n*10 + chara.to_digit(10).unwrap() as usize,
			(CollectNum(n), _) => self.interpret_seq(n, chara),
		}
	}
}

pub fn print_prompt() {//TODO: let the user configure it
	print_flush!("> ");
}
