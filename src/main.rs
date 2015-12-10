#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;

extern crate builtins;
//TODO: better arg escaping
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
}

fn shell_loop(input: &mut BufRead, interactive: bool) {
	print_prompt(interactive);

	for line in input.lines() {
		match line {
			Ok(expr) => {
				match parse(&expr) {
					Some((command, expressions)) => {
						match command {
							"" => {},
							"cd" => builtins::cd(expressions[0]),
							"exit" => builtins::exit(expressions[0]),
							_ => invoke(command, &expressions),
						}
					},
					None => {},

				}
				print_prompt(interactive);
			},
			Err(err) => err_exit!(err),
		}
	}
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
fn print_prompt(interactive: bool) {
	use std::io::Write;

	if interactive {
		print!("> ");
		io::stdout().flush();
	}
}
