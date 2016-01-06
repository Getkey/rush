#![feature(libc)]
#![feature(io)]//unstable io
#![feature(type_ascription)]//see history::History.push()

#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;

extern crate libc;
extern crate termios;

#[macro_use]
mod print_macros;
mod interpret;
mod line;
mod history;

use std::io;
use std::io::BufRead;
use std::fs::File;

macro_rules! err_exit {
	($err:expr) => ({
		println_stderr!("{}", $err);
		std::process::exit(1);
	});
}

fn main() {
	let mut args = std::env::args();

	//TODO: call shell loop over a rushrc file

	if let Some(filepath) = args.nth(1) {
		let file = match File::open(&filepath) {
			Ok(file) => file,
			Err(err) => err_exit!(err),
		};

		for line in io::BufReader::new(file).lines() {
			match line {
				Ok(line) => interpret::read(&line),
				Err(err) => err_exit!(err),
			}
		}
	} else {//interactive mode
		let old_termios = termios::Termios::from_fd(libc::STDIN_FILENO).unwrap();
		let mut new_termios = old_termios;
		new_termios.c_lflag &= !(termios::ICANON | termios::ISIG | termios::ECHO);
		termios::tcsetattr(libc::STDIN_FILENO, termios::TCSANOW, &new_termios).unwrap();

		let stdin = io::stdin();
		let mut stdinlock = stdin.lock();

		shell_loop_interactive(&mut stdinlock as &mut BufRead);

		termios::tcsetattr(libc::STDIN_FILENO, termios::TCSANOW, &old_termios).unwrap();
		println!("");//break line before exiting
	};
}

fn shell_loop_interactive(input: &mut BufRead) {
	use std::io::Read;

	line::print_prompt();

	let mut line = line::Line::new();
	for chara in input.chars() {
		let chara = chara.unwrap();

		if chara == '\n' {
			print_flush!("{}", chara);

			let s_line: String = line.line.iter().cloned().collect();
			line.clear();

			interpret::read(&s_line);
			line.history.push(s_line);

			line::print_prompt();
		} else if chara == '\u{4}' {// ^D
			print_flush!("{}", get_caret_notation(&chara));
			break;
		} else {
			line.append(chara);
		}
	}
}

fn get_caret_notation(ctrl_char: &char) -> String {
	//found by inverting the 7th bit of the ASCII code
	let printable = (*ctrl_char as u8 ^ 0b0100_0000) as char;
	format!("^{}", printable)
}
