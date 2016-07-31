extern crate builtins;


pub fn read(command_line: &str) {
	let token_list = tokenize(command_line);
	let mut token_iter = token_list.iter();
	match generate_tree(&mut token_iter) {
		Ok(mut tree) => tree.execute_command(),
		Err(err) => println!("{}", err),
	}
}

#[derive(Copy, Clone)] // this enum takes 8b therefore copying it is better than having a 32b or 64b pointer
enum TokenMachineState {
	Start,
	StrCmd(usize),
	SeparateWs,
	StrArg(usize),
}

#[derive(PartialEq, Debug)] // used for tests
enum Token<'a, 'b> {
	LeftParen,
	RightParen,
	Cmd(&'a str),
	Arg(&'b str),
}

fn tokenize(command_line: &str) -> Vec<Token> {
	use self::TokenMachineState::*;
	use self::Token::*;

	let mut state = Start;
	let mut token_list = Vec::new();

	let mut char_it = command_line.char_indices();

	loop {
		if let Some((i, curr_char)) = char_it.next() {
			match (state, curr_char) {
				(Start, ' ') | (Start, '\t') => {},
				(Start, '(') => token_list.push(LeftParen),
				(Start, ')') => token_list.push(RightParen), // this is invalid but it's the parser's job to find it out
				(Start, _) => state = StrCmd(i),

				(StrCmd(slice_start), ')') => {
					token_list.push(Cmd(&command_line[slice_start..i]));
					token_list.push(RightParen);
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
					token_list.push(RightParen);
					state = Start;
				}
				(StrArg(_), _) => {},
			}
		} else {
			// there might be a last token undergoing the token creation process
			// it has to be appended to the token list
			match state {
				StrCmd(slice_start) => token_list.push(Cmd(&command_line[slice_start..])),
				StrArg(slice_start) => token_list.push(Arg(&command_line[slice_start..])),
				_ => {},
			}
			break;
		}
	}

	token_list
}

#[derive(PartialEq, Copy, Clone)] // this enum takes 8b therefore copying it is better than having a 32b or 64b pointer
enum ParseMachineState {
	Start,
	CollectArg,
	SubcommandEnd,
}

use std::slice::Iter;

fn generate_tree<'a>(token_iter: &mut Iter<Token<'a, 'a>>) -> Result<Command<'a>, &'static str> {
	use self::ParseMachineState::*;

	let (res, state) = parse(token_iter);

	if state != CollectArg && res.is_ok() { // accepting state when not doing recursion
		Err("Parse error")
	} else {
		res
	}
}

fn parse<'a>(token_iter: &mut Iter<Token<'a, 'a>>) -> (Result<Command<'a>, &'static str>, ParseMachineState) {
	use self::ParseMachineState::*;
	use self::Token::*;
	use std::ptr;
	use std::mem;

	let mut state = Start;
	let mut cmd = Command::new();

	loop {
		if let Some(token) = token_iter.next() {
			match (state, token) {
				(Start, &Cmd(cmd_name)) => {
					unsafe {
						ptr::write(&mut cmd.cmd, Box::new(Param::Arg(cmd_name)));
					}
					state = CollectArg;
				},
				(Start, &LeftParen) => {
					// do recursion
					let (res, recur_state) = parse(token_iter);
					match res {
						Err(_) => {
							mem::forget(cmd.cmd);
							return (res, state)
						},
						Ok(subcmd) => {
							if recur_state != SubcommandEnd {
								mem::forget(cmd.cmd);
								return (Err("Error"), state);
							} else {
								unsafe {
									ptr::write(&mut cmd.cmd, Box::new(Param::Cmd(subcmd)));
								}
							}
						},
					}
				},
				(Start, &RightParen) => {
					mem::forget(cmd.cmd);
					return (Err("Unexpected ')'"), state)
				},
				(Start, &Arg(_)) => unreachable!(), // if that happens the tokenizer is buggy

				(CollectArg, &Arg(arg_str)) => {
					cmd.params.push(Param::Arg(arg_str));
				}
				(CollectArg, &Cmd(_)) => unreachable!(), // if that happens the tokenizer is buggy
				(CollectArg, &RightParen) => {
					return (Ok(cmd), SubcommandEnd);
				},
				(CollectArg, &LeftParen) => {
					//do recursion
					let (res, recur_state) = parse(token_iter);
					match res {
						Err(_) => return (res, state),
						Ok(subcmd) => {
							if recur_state != SubcommandEnd {
								panic!("ERROR");
							} else {
								cmd.params.push(Param::Cmd(subcmd));
							}
						},
					}
				}

				(SubcommandEnd, _) => unreachable!(),
			}
		} else {
			break;
		}
	}

	if state == Start {
		mem::forget(cmd.cmd);
		(Err("Empty commands are not allowed"), state)
	} else {
		(Ok(cmd), state)
	}
}


struct Command<'a> {
	cmd: Box<Param<'a>>,
	params: Vec<Param<'a>>,
}
impl<'a> Command<'a> {
	fn new() -> Command<'a> {
		use std::mem;

		unsafe {
			Command {
				cmd: mem::uninitialized(),
				params: Vec::new(),
			}
		}
	}
	fn get_final_arglist(&mut self) -> Vec<&str> {
		let mut final_arglist: Vec<&str> = Vec::with_capacity(self.params.len());

		/*for arg in self.params.iter_mut() { // substitue subcommands recursively
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
		}*/

		final_arglist
	}
	fn execute_command(&mut self) {//returns stdout - possibly return a array of strings?
		/*match &self.cmd[..] {
			"" => {},
			/*"cd" => builtins::cd(&self.params),
			"exit" => builtins::exit(&self.params),*/
			_ => invoke_command(self),
		};*/
	}
	fn execute_subcommand(&mut self) -> String {
		/*match &self.cmd[..] {
			/*"" => {},
			"cd" => builtins::cd(&self.params),
			"exit" => builtins::exit(&self.params),*/
			_ => invoke_subcommand(self),
		}*/ "fuck you".to_string()
	}
}
enum Param<'a> {
	Arg(&'a str),
	Cmd(Command<'a>),

}
fn invoke_command(command: &mut Command) {
	use std::process::Command;
	/*match Command::new(&command.cmd)
		.args(&command.get_final_arglist())
		.spawn() {
			Ok(mut subproc) => {
				subproc.wait();
			},
			Err(err) => println!("{}", err),
	}*/
}
fn invoke_subcommand(command: &mut Command) -> String {
	/*{
		for arg in command.get_final_arglist() {
			println!("shitty debugging: {}", arg);
		}
	}
	use std::process::Command;
	let output = Command::new(&command.cmd)
		.args(&command.get_final_arglist())
		.output()
		.expect("Failed to start command");

	String::from_utf8(output.stdout).unwrap()*/ "fuck you".to_string()
}

#[cfg(test)]
mod tests {
	#[test]
	fn tokenize() {
		use super::tokenize;
		use super::Token::*;

		assert_eq!(
			[Cmd("echo"), Arg("lol"), LeftParen, Cmd("srnaeinei"), RightParen],
			tokenize("echo lol (srnaeinei)").as_slice()
		);

		assert_eq!(
			[LeftParen, RightParen],
			tokenize("()").as_slice()
		);

		assert_eq!(
			[Cmd("echo")],
			tokenize("echo").as_slice()
		);

		assert_eq!(
			[Cmd("echo"), Arg("lol")],
			tokenize("echo lol").as_slice()
		);
	}
}
