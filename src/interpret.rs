extern crate builtins;

use std::slice::Iter;
use std::borrow::Cow;

pub fn read(command_line: &str) {
	match tokenize(command_line) {
		Ok(token_list) => {
			let mut token_iter = token_list.iter();
			match generate_tree(&mut token_iter) {
				Ok(mut tree) => tree.execute_command(),
				Err(err) => println_stderr!("Parse error: {}", err),
			}
		},
		Err(err) => println_stderr!("Tokenization error: {}", err),
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

fn tokenize(command_line: &str) -> Result<Vec<Token>, &'static str> {
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
					state = SeparateWs;
				},
				(StrCmd(slice_start), ' ') | (StrCmd(slice_start), '\t') => {
					token_list.push(Cmd(&command_line[slice_start..i]));
					state = SeparateWs;
				},

				(SeparateWs, ' ') | (SeparateWs, '\t') => {},
				(SeparateWs, ')') => token_list.push(RightParen),
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
					state = SeparateWs;
				}

				(StrCmd(_), '(') | (StrArg(_), '(') => return Err("Unexpected '(' in command or argument"),
				(StrCmd(_), _) | (StrArg(_), _) => {},
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

	Ok(token_list)
}

#[derive(PartialEq, Copy, Clone)] // this enum takes 8b therefore copying it is better than having a 32b or 64b pointer
enum ParseMachineState {
	Start,
	CollectArg,
	SubcommandEnd,
}

fn generate_tree<'a>(token_iter: &mut Iter<Token<'a, 'a>>) -> Result<Command<'a>, &'static str> {
	use self::ParseMachineState::*;

	let (res, state) = parse(token_iter);

	if state != CollectArg && res.is_ok() { // accepting state when not doing recursion
		Err("Missing matching parenthese")
	} else {
		res
	}
}

fn parse<'a>(token_iter: &mut Iter<Token<'a, 'a>>) -> (Result<Command<'a>, &'static str>, ParseMachineState) {
	use self::ParseMachineState::*;
	use self::Token::*;
	use std::{ptr, mem};

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
								return (Err("Error in subcommand"), state);
							} else {
								unsafe {
									ptr::write(&mut cmd.cmd, Box::new(Param::Cmd(subcmd)));
								}
								state = CollectArg;
							}
						},
					}
				},
				(Start, &RightParen) => {
					mem::forget(cmd.cmd);
					return (Err("Unexpected ')' at start of (sub)command"), state)
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
								return (Err("Error in subcommand"), state);
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
		(Err("Subcommand never closed"), state)
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
	fn get_final(&'a mut self) -> (String, Vec<String>) {
		// the calls to `into_owned()` are ugly, is there a way to get rid of them?
		let mut final_arglist: Vec<String> = Vec::with_capacity(self.params.len());

		for arg in self.params.iter_mut() { // substitue subcommands recursively
			final_arglist.push(arg.as_arg().into_owned());
		}

		(self.cmd.as_arg().into_owned(), final_arglist)
	}
	fn execute_command(&'a mut self) {//returns stdout - possibly return a array of strings?
		let (cmd, arglist) = self.get_final();
		let actual_arglist: Vec<&str> = arglist.iter().map(|arg| &arg[..]).collect();

		match &cmd[..] {
			"" => {},
			"cd" => builtins::cd(&actual_arglist),
			"exit" => builtins::exit(&actual_arglist),
			_ => {
				use std::process::Command;
				match Command::new(cmd)
					.args(&actual_arglist)
					.spawn() {
						Ok(mut subproc) => {
							subproc.wait();
						},
						Err(err) => println_stderr!("{}", err),
				}
			},
		};
	}
	fn execute_subcommand(&'a mut self) -> Cow<'a, str> {
		let (cmd, arglist) = self.get_final();
		let actual_arglist: Vec<&str> = arglist.iter().map(|arg| &arg[..]).collect();

		return match &cmd[..] {
			"" => Cow::Borrowed(""),
			"cd" => {
				builtins::cd(&actual_arglist);
				Cow::Borrowed("")
			},
			"exit" => {
				builtins::exit(&actual_arglist);
				Cow::Borrowed("")
			},
			_ => {
				use std::process::Command;
				match Command::new(cmd)
					.args(&actual_arglist)
					.output() {
						Ok(output) => {
							use std::string::String;

							unsafe {
								Cow::Owned(String::from_utf8_unchecked(output.stdout))
							}
						},
						Err(err) => {
							println_stderr!("{}", err);
							Cow::Borrowed("")
						},
				}
			},
		};
	}
}
enum Param<'a> {
	Arg(&'a str),
	Cmd(Command<'a>),

}
impl<'a> Param<'a> {
	fn as_arg(&'a mut self) -> Cow<'a, str> {
		match self {
			&mut Param::Cmd(ref mut subcmd) => subcmd.execute_subcommand(),
			&mut Param::Arg(arg_str) => Cow::Borrowed(arg_str),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::tokenize;
	use super::Token::*;

	#[test]
	fn empty_command() {
		assert_eq!(
			[LeftParen, RightParen],
			tokenize("()").unwrap().as_slice()
		);
	}

	#[test]
	fn tokenize_proper() {
		assert_eq!(
			[Cmd("echo"), Arg("foo"), LeftParen, Cmd("bar"), RightParen],
			tokenize("echo foo (bar)").unwrap().as_slice()
		);

		assert_eq!(
			[Cmd("echo")],
			tokenize("echo").unwrap().as_slice()
		);

		assert_eq!(
			[Cmd("echo"), Arg("foo")],
			tokenize("echo foo").unwrap().as_slice()
		);

		assert_eq!(
			[LeftParen, Cmd("echo"), RightParen, Arg("foo")],
			tokenize("(echo) foo").unwrap().as_slice()
		);
		assert_eq!(
			[LeftParen, Cmd("echo"), Arg("echo"), RightParen, Arg("foo")],
			tokenize("(echo echo) foo").unwrap().as_slice()
		);
	}
	#[test]
	fn tokenize_proper_though_not_for_the_parser() {
		assert_eq!(
			[LeftParen, LeftParen, LeftParen, Cmd("echo"), Arg("lol"), RightParen, RightParen, RightParen, Arg("stuff")],
			tokenize("(((echo lol))) stuff").unwrap().as_slice()
		);
		assert_eq!(
			[Cmd("garbage"), RightParen],
			tokenize("garbage)").unwrap().as_slice()
		);
	}

	#[test]
	#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
	fn tokenize_paren_in_cmd_middle() {
		tokenize("ech(o").unwrap();
	}
	#[test]
	#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
	fn tokenize_paren_in_arg_middle() {
		tokenize("echo oops(typo").unwrap();
	}
	#[test]
	#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
	fn tokenize_paren_in_cmd_end() {
		tokenize("echo( stuff").unwrap();
	}
	#[test]
	#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
	fn tokenize_paren_in_arg_end() {
		tokenize("echo oops(").unwrap();
	}
}
