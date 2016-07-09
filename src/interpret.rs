extern crate builtins;

use std::str::Chars;


pub fn read(command_line: &str) {
	match control_machine(&mut command_line.chars()) {
		Ok(mut tree) => tree.execute_command(),
		Err(err) => println!("{}", err),
	}

}

#[derive(PartialEq, Copy, Clone)] // this enum takes 1B therefore copying it is better than having a 4B pointer (on 32bit machines) or a 8B pointer (on 64 bit machines)
pub enum State {
	Start,
	StrArg,
	SubcommandEnd,
}

fn tokenize(char_it: &mut Chars) -> (Result<Command, &'static str>, State) { // uses a recursive automaton
	use self::State::*;
	let mut cmd = Command::new();

	let mut state = State::Start;

	loop {
		if let Some(curr_char) = char_it.next() {
			match (state, curr_char) {
				(Start, ')') => {
					state = SubcommandEnd;
					return (Ok(cmd), state);
				},
				(Start, ' ') | (Start, '\t') => {},
				(Start, '(') => {
					// do recursion
					let (res, recurState) = tokenize(char_it);
					if recurState != SubcommandEnd {
						panic!("ERROR");
					} else if let Err(_) = res {
						return (res, state);
					}
				},
				(Start, _) => {
					state = StrArg
					// create string
					// push character
				},

				(StrArg, ')') => {
					state = SubcommandEnd;
					return (Ok(cmd), state);
				},
				(StrArg, ' ') | (StrArg, '\t') => {
					state = Start;
					// append string to arg vector
				},
				(StrArg, _) => {
					//push character
				},

				(SubcommandEnd, _) => {
					unreachable!();
				},
			}
		} else {
			if state == StrArg {
				// append string to arg array
			}
			break;
		}
	}

	(Ok(cmd), state)
}

fn control_machine(char_it: &mut Chars) -> Result<Command, &'static str> {
	use self::State::*;

	let (res, state) = tokenize(char_it);

	if state != Start && state != StrArg {
		Err("There is an error")
	} else {
		res
	}
}

//TODO:
//struct Command(Vec<Param>);
//https://doc.rust-lang.org/book/structs.html

struct Command {
	cmd: String,
	params: Vec<Param>,
}
impl Command {
	fn new() -> Command {
		Command {
			cmd: String::new(),
			params: Vec::new(),
		}
	}
	fn get_final_arglist(&mut self) -> Vec<&str> {
		let mut final_arglist: Vec<&str> = Vec::with_capacity(self.params.len());

		for arg in self.params.iter_mut() { // substitue subcommands recursively
			let stdout = if let Param::Cmd(ref mut subcommand) = *arg {
				Some(subcommand.execute_subcommand())
			} else {
				None
			};

			if let Some(stdout) = stdout {
				*arg = Param::Arg(stdout); // the datastructure holds the `String`
			}

			if let Param::Arg(ref arg_str) = *arg { // at this point this always happen because `Cmd` became `Arg`s
				final_arglist.push(arg_str);
			}
		}

		final_arglist
	}
	fn execute_command(&mut self) {//returns stdout - possibly return a array of strings?
		match &self.cmd[..] {
			"" => {},
			/*"cd" => builtins::cd(&self.params),
			"exit" => builtins::exit(&self.params),*/
			_ => invoke_command(self),
		};
	}
	fn execute_subcommand(&mut self) -> String {
		match &self.cmd[..] {
			/*"" => {},
			"cd" => builtins::cd(&self.params),
			"exit" => builtins::exit(&self.params),*/
			_ => invoke_subcommand(self),
		}
	}
}
enum Param {
	Arg(String),
	Cmd(Command),

}
fn invoke_command(command: &mut Command) {
	use std::process::Command;
	match Command::new(&command.cmd)
		.args(&command.get_final_arglist())
		.spawn() {
			Ok(mut subproc) => {
				subproc.wait();
			},
			Err(err) => println!("{}", err),
	}
}
fn invoke_subcommand(command: &mut Command) -> String {
	{
		for arg in command.get_final_arglist() {
			println!("shitty debugging: {}", arg);
		}
	}
	use std::process::Command;
	let output = Command::new(&command.cmd)
		.args(&command.get_final_arglist())
		.output()
		.expect("Failed to start command");

	String::from_utf8(output.stdout).unwrap()
}
