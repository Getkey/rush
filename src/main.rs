#![feature(libc)]
#![feature(io)]

#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;

extern crate libc;
extern crate termios;

extern crate builtins;
use std::io;
use std::io::BufRead;
use std::fs::File;

macro_rules! err_exit {
	($e:expr) => ({
		println!("{}", $e);
		std::process::exit(1);
	});
}

fn main() {
	let mut args = std::env::args();

	let old_termios = termios::Termios::from_fd(libc::STDIN_FILENO).unwrap();
	let mut new_termios = old_termios;
	new_termios.c_lflag &= !(termios::ICANON | termios::ISIG | termios::ECHO);
	termios::tcsetattr(libc::STDIN_FILENO, termios::TCSANOW, &new_termios).unwrap();

	//TODO: call shell loop over a rushrc file

	if let Some(filepath) = args.nth(1) {
		let file = match File::open(&filepath) {
			Ok(file) => file,
			Err(err) => err_exit!(err),
		};

		shell_loop(&mut io::BufReader::new(file) as &mut BufRead, false);
	} else {//interactive mode
		let stdin = io::stdin();
		let mut stdinlock = stdin.lock();

		shell_loop(&mut stdinlock as &mut BufRead, true);
	};

	termios::tcsetattr(libc::STDIN_FILENO, termios::TCSANOW, &old_termios).unwrap();
	println!("");//break line before exiting
}

fn shell_loop(input: &mut BufRead, interactive: bool) {
	use std::io::Read;
	print_prompt(interactive);

	let mut esc_seq = AnsiEsc { csi: String::new(), n: String::new() };

	let mut line = Line::new();
	for chara in input.chars() {
		let chara = chara.unwrap();

		if !esc_seq.csi.is_empty() {
			if esc_seq.csi == "\u{1b}" && chara == '[' {
				esc_seq.csi.push(chara);
			} else if esc_seq.csi == "\u{1b}[" || esc_seq.csi == "\u{9b}" {
				if chara >= '0' && chara <= '9' {
					esc_seq.n.push(chara);
				} else {
					match chara {
						'A' => {},//TODO:
						'B' => {},//an history
						'C' => {
							line.move_cursor_right(&format!("{}{}C", esc_seq.csi, esc_seq.n));
						},
						'D' => {
							line.move_cursor_left(&format!("{}{}D", esc_seq.csi, esc_seq.n));
						},
						_ => {},
					}
					esc_seq.csi.clear();
					esc_seq.n.clear();
				}

			}
		} else if chara.is_control() {
			if chara == '\n' {
				print_char(&chara);

				let s_line: String = line.line.iter().cloned().collect();
				line.clear();
				if let Some((command, expressions)) = parse(&s_line) {
					match command {
						"" => {},
						"cd" => builtins::cd(expressions[0]),
						"exit" => builtins::exit(expressions[0]),
						_ => invoke(command, &expressions),
					}
				}
				print_prompt(interactive);
			} else if chara == '\u{7f}' {
				line.del_prev_char();
			} else if chara == '\u{4}' || (!interactive && chara == '\u{3}') {// ^D or ^C if not interactive mode
				print_slice(&get_caret_notation(&chara));
				break;
			} else if chara == '\u{1b}' || chara == '\u{9b}' {
				esc_seq.csi = chara.to_string();
			}
		} else {
			line.push(chara);
		}
	}
}

struct Line {
	line: Vec<char>,
	cursor_pos: usize,
}
impl Line {
	fn new() -> Line {
		Line {
			line: Vec::new(),
			cursor_pos: 0,
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

		print_slice(&reprint);
	}
	fn del_prev_char(&mut self) {//TODO: check wrapping to erase previous line
		if self.cursor_pos != 0 {
			self.cursor_pos -= 1;

			if self.cursor_pos == self.line.len() - 1 {
				print_slice("\u{8} \u{8}");
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

				print_slice(&eraser);
			}
		}
	}
	fn clear(&mut self) {
		self.line.clear();
		self.cursor_pos = 0;
	}
	fn move_cursor_left(&mut self, seq: &str) {
		if self.cursor_pos != 0 {
			print_slice(seq);
			self.cursor_pos -= 1;
		}
	}
	fn move_cursor_right(&mut self, seq: &str) {
		if self.cursor_pos != self.line.len() {
			print_slice(seq);
			self.cursor_pos += 1;
		}
	}
}

struct AnsiEsc {
	csi: String,
	n: String,
}

fn parse(line: &str) -> Option<(&str, Vec<&str>)> {
	let re = regex!(r#"([^\s']+)|'([^']*)'"#);
	let parsed: Vec<&str> = re.captures_iter(line).filter_map(|x| if let Some(f_cap) = x.at(1) {
		Some(f_cap)
	} else if let Some(s_cap) = x.at(2) {
		Some(s_cap)
	} else {
		None
	}).collect();

	if let Some(v) = parsed.split_first() {
		let (command, args) = v;
		return Some((command, (*args).to_vec()));
	}
	None
}
fn invoke(command: &str, args: &[&str]) {
	use std::process::Command;
	match Command::new(command).args(args).spawn() {
		Ok(mut subproc) => {
			subproc.wait();
		},
		Err(err) => println!("{}", err),
	}
}

fn get_caret_notation(ctrl_char: &char) -> String {
	//found by inverting the 7th bit of the ASCII code
	let printable = (*ctrl_char as u8 ^ 0b0100_0000) as char;
	format!("^{}", printable)
}

fn print_slice(string: &str) {
	use std::io::Write;

	print!("{}", string);
	io::stdout().flush();
}
fn print_char(chara: &char) {
	use std::io::Write;

	print!("{}", chara);
	io::stdout().flush();
}
fn print_prompt(interactive: bool) {
	if interactive {
		print_slice("> ");
	}
}
