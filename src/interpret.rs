extern crate builtins;


pub fn read(command_line: &str) {
	/*match control_machine(command_line) {
		Ok(mut tree) => tree.execute_command(),
		Err(err) => println!("{}", err),
	}*/
	let token_list = tokenize(command_line);
	let token_iter = token_list.iter();
}

#[derive(PartialEq, Copy, Clone)] // this enum takes 8b therefore copying it is better than having a 32b or 64b pointer
pub enum State {
	Start,
	StrCmd(usize),
	SeparateWs,
	StrArg(usize),
}

pub enum Token<'a, 'b> {
	LeftParen,
	RightParen,
	Cmd(&'a str),
	Arg(&'b str),
}

fn tokenize(command_line: &str) -> Vec<Token> {
	use self::State::*;
	use self::Token::*;

	let mut state = Start;
	let mut token_list = Vec::new();

	let mut char_it = command_line.char_indices();

	loop {
		if let Some((i, curr_char)) = char_it.next() {
			match (state, curr_char) {
				(Start, ' ') | (Start, '\t') => {},
				(Start, '(') => token_list.push(LeftParen),
				(Start, _) => state = StrCmd(i),

				(StrCmd(slice_start), ')') => {
					token_list.push(Cmd(&command_line[slice_start..i]));
					state = Start;
				},
				(StrCmd(slice_start), ' ') | (StrCmd(slice_start), '\t') => {
					token_list.push(Cmd(&command_line[slice_start..i]));
					state = SeparateWs;
				},
				(StrCmd(_), _) => {},

				(SeparateWs, ' ') | (SeparateWs, '\t') => {},
				(SeparateWs, ')') => {
					token_list.push(RightParen);
					state = Start;
				},
				(SeparateWs, '(') => {
					token_list.push(LeftParen);
					state = Start;
				},
				(SeparateWs, _) => state = StrArg(i),

				(StrArg(slice_start), ' ') | (StrArg(slice_start), '\t') => {
					token_list.push(Arg(&command_line[slice_start..i]));
					state = SeparateWs;
				},
				(StrArg(slice_start), ')') => {
					token_list.push(Arg(&command_line[slice_start..i]));
					state = Start;
				}
				(StrArg(_), _) => {},
			}
		}
	}
}

use std::slice::Iter;

fn generate_tree<'a>(token_list: Iter<Vec<Token>>) -> Result<Command<'a>, &'static str> {
	Err("Empy commands are not allowed")
	/*use self::State::*;
	let mut cmd = Command::new();

	let mut state = State::Start;
	let slice_start = 0; // whatever, doesn't matter
	let oldI = 0;

	loop {
		if let Some((i, curr_char)) = char_it.next() {
			match (state, curr_char) {
				(Start, ')') => return (Err("Empty subcommands are not allowed"), state),
				(Start, ' ') | (Start, '\t') => {},
				(Start, '(') => {
					// do recursion
					let (res, recur_state) = tokenize(char_it);
					if recur_state != SubcommandEnd {
						panic!("ERROR");
					} else if let Err(_) = res {
						return (res, state);
					} else {
						cmd.cmd.push(Param::Cmd(res));
					}
				},
				(Start, _) => {
					state = StrCmd;
					slice_start = i;
				},

				(StrCmd, ')') | (SeparateWs, ')') | (StrArg, ')') => {
					state = SubcommandEnd;
					return (Ok(cmd), state);
				},

				(StrCmd, ' ') | (StrCmd, '\t') => {
				//	cmd.cmd = &
				},
				(StrCmd, _) => {},

				(SeparateWs, ' ') | (SeparateWs, '\t') => {},
				(SeparateWs, _) => {
					state = StrArg;
					cmd.params.push(Param::Arg(curr_char.to_string()));
				},

				(StrArg, ' ') | (StrArg, '\t') => {
					state = SeparateWs;
					// append string to arg vector
				},
				(StrArg, _) => {
					let i = cmd.params.len() - 1;
					if let Param::Arg(ref mut str_arg) = cmd.params[i] {
						str_arg.push(curr_char);
					} else {
						unreachable!();
					}
					//push character
				},

				(SubcommandEnd, _) => {
					unreachable!();
				},
			}
			oldI = i;
		} else {
			if state == StrArg {
				// append string to arg array
			}
			break;BNF
		}
	}

	(Ok(cmd), state)*/
}

/*fn control_machine(command_line: &str) -> Result<Command, &'static str> {
	use self::State::*;

	let (res, state) = tokenize(command_line.char_indices(), command_line);

	if state != Start && state != StrArg {
		Err("There is an error")
	} else {
		res
	}
}*/


struct Command<'a> {
	cmd: &'a str,
	params: Vec<Param<'a>>,
}
impl<'a> Command<'a> {
	fn new(cmd: &str) -> Command {
		Command {
			cmd: cmd,
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
enum Param<'a> {
	Arg(String),
	Cmd(Command<'a>),

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
