extern crate builtins;

use std::slice::Iter;
use std::borrow::Cow;

type CmdLine<'a> = Vec<Param<'a>>;

pub fn read(command_line: &str) {
	match tokenize(command_line) {
		Ok(token_list) => {
			let mut token_iter = token_list.iter();
			match generate_tree(&mut token_iter) {
				Ok(ref tree) => execute_command(tree),
				Err(err) => eprintln!("Parse error: {}", err),
			}
		},
		Err(err) => eprintln!("Tokenization error: {}", err),
	}
}

#[derive(Copy, Clone)] // this enum takes only 1B more than usize so copying it is no big deal
enum TokenMachineState {
	Start,
	SeparateWs,
	StrWord(usize),
	WordFirstQuote,
	QuotedWord(usize),
}

#[derive(PartialEq, Debug)] // used for tests
enum Token<'a> {
	LeftParen,
	RightParen,
	Word(&'a str),
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
				(Start, '\'') => state = WordFirstQuote,
				(Start, _) => state = StrWord(i),

				(WordFirstQuote, '\'') => token_list.push(Word("")),
				(WordFirstQuote, _) => state = QuotedWord(i),

				(QuotedWord(slice_start), '\'') => {
					token_list.push(Word(&command_line[slice_start..i]));
					state = SeparateWs;
				},

				(StrWord(slice_start), ')') => {
					token_list.push(Word(&command_line[slice_start..i]));
					token_list.push(RightParen);
					state = SeparateWs;
				},
				(StrWord(slice_start), ' ') | (StrWord(slice_start), '\t') => {
					token_list.push(Word(&command_line[slice_start..i]));
					state = SeparateWs;
				},
				(StrWord(_), '(') => return Err("Unexpected '(' in command or argument"),
				(StrWord(_), _) | (QuotedWord(_), _) => {},

				(SeparateWs, ' ') | (SeparateWs, '\t') => {},
				(SeparateWs, ')') => token_list.push(RightParen),
				(SeparateWs, '(') => {
					token_list.push(LeftParen);
					state = Start;
				},
				(SeparateWs, '\'') => state = WordFirstQuote,
				(SeparateWs, _) => state = StrWord(i),

			}
		} else {
			// there might be a last token undergoing the token creation process
			// it has to be appended to the token list
			match state {
				StrWord(slice_start) => token_list.push(Word(&command_line[slice_start..])),
				WordFirstQuote | QuotedWord(_) => return Err("Unclosed string literal"),
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

fn generate_tree<'a>(token_iter: &mut Iter<Token<'a>>) -> Result<CmdLine<'a>, &'static str> {
	use self::ParseMachineState::*;

	let (res, state) = parse(token_iter);

	if state != CollectArg && res.is_ok() { // accepting state when not doing recursion
		Err("Missing matching parenthese")
	} else {
		res
	}
}

fn parse<'a>(token_iter: &mut Iter<Token<'a>>) -> (Result<CmdLine<'a>, &'static str>, ParseMachineState) {
	use self::ParseMachineState::*;
	use self::Token::*;
	use self::Param::*;

	let mut state = Start;
	let mut cmd_line: CmdLine = Vec::new();

	loop {
		if let Some(token) = token_iter.next() {
			match (state, token) {
				(Start, &Word(cmd_name)) | (CollectArg, &Word(cmd_name)) => {
					cmd_line.push(Arg(cmd_name));
					state = CollectArg;
				},
				(Start, &LeftParen) | (CollectArg, &LeftParen) => {
					// do recursion
					let (res, recur_state) = parse(token_iter);
					match res {
						Err(_) => return (res, state),
						Ok(subcmd) => {
							if recur_state != SubcommandEnd {
								return (Err("Error in subcommand"), state);
							} else {
								cmd_line.push(Cmd(subcmd));
								state = CollectArg;
							}
						},
					}
				},
				(Start, &RightParen) => {
					return (Err("Unexpected empty subcommand"), state)
				},

				(CollectArg, &RightParen) => {
					return (Ok(cmd_line), SubcommandEnd);
				},

				(SubcommandEnd, _) => unreachable!(),
			}
		} else {
			break;
		}
	}

	if state == Start {
		(Err("Subcommand never closed"), state)
	} else {
		(Ok(cmd_line), state)
	}
}



fn get_final<'a>(cmd_line: &'a CmdLine<'a>) -> Vec<String> {
	// the calls to `into_owned()` are ugly, is there a way to get rid of them?
	let mut final_arglist: Vec<String> = Vec::with_capacity(cmd_line.len());

	for arg in cmd_line.iter() { // substitue subcommands recursively
		final_arglist.push(arg.as_arg().into_owned());
	}

	final_arglist
}
fn execute_command<'a>(cmd_line: &'a CmdLine<'a>) {//returns stdout - possibly return a array of strings?
	let arglist = get_final(cmd_line);
	let actual_arglist: Vec<&str> = arglist.iter().map(|arg| &arg[..]).collect();

	match actual_arglist[0] { // there is always at least a command
		"" => {},
		"cd" => builtins::cd(&actual_arglist[1..]),
		"exit" => builtins::exit(&actual_arglist[1..]),
		_ => {
			use std::process::Command;
			match Command::new(actual_arglist[0])
				.args(&actual_arglist[1..])
				.spawn() {
					Ok(mut subproc) => {
						match subproc.wait() {
							Ok(_) => {},
							Err(err) => eprintln!("Program exited with error code: {:?}", err.kind()),
						}
					},
					Err(err) => eprintln!("{}", err),
			}
		},
	};
}
fn execute_subcommand<'a>(cmd_line: &CmdLine<'a>) -> Cow<'a, str> {
	let arglist = get_final(cmd_line);
	let actual_arglist: Vec<&str> = arglist.iter().map(|arg| &arg[..]).collect();

	return match actual_arglist[0] { // ther is always at least a command
		"" => Cow::Borrowed(""),
		"cd" => {
			builtins::cd(&actual_arglist[1..]);
			Cow::Borrowed("")
		},
		"exit" => {
			builtins::exit(&actual_arglist[1..]);
			Cow::Borrowed("")
		},
		_ => {
			use std::process::Command;
			match Command::new(actual_arglist[0])
				.args(&actual_arglist[1..])
				.output() {
					Ok(output) => {
						use std::string::String;

						unsafe {
							Cow::Owned(String::from_utf8_unchecked(output.stdout))
						}
					},
					Err(err) => {
						eprintln!("{}", err);
						Cow::Borrowed("")
					},
			}
		},
	};
}
enum Param<'a> {
	Arg(&'a str),
	Cmd(CmdLine<'a>),

}
impl<'a> Param<'a> {
	fn as_arg(&'a self) -> Cow<'a, str> {
		match self {
			&Param::Cmd(ref subcmd) => execute_subcommand(subcmd),
			&Param::Arg(arg_str) => Cow::Borrowed(arg_str),
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
			[Word("echo"), Word("foo"), LeftParen, Word("bar"), RightParen],
			tokenize("echo foo (bar)").unwrap().as_slice()
		);

		assert_eq!(
			[Word("echo")],
			tokenize("echo").unwrap().as_slice()
		);

		assert_eq!(
			[Word("echo"), Word("foo")],
			tokenize("echo foo").unwrap().as_slice()
		);

		assert_eq!(
			[LeftParen, Word("echo"), RightParen, Word("foo")],
			tokenize("(echo) foo").unwrap().as_slice()
		);
		assert_eq!(
			[LeftParen, Word("echo"), Word("echo"), RightParen, Word("foo")],
			tokenize("(echo echo) foo").unwrap().as_slice()
		);
	}

	#[test]
	fn tokenize_string_literal() {
		assert_eq!(
			[Word("echo hole"), Word("stuff")],
			tokenize("'echo hole' stuff").unwrap().as_slice()
		);
		assert_eq!(
			[Word("echo"), Word("foo bar")],
			tokenize("echo 'foo bar'").unwrap().as_slice()
		);
		assert_eq!(
			[Word("echo"), Word("foo"), Word("bar")],
			tokenize("'echo' 'foo' 'bar'").unwrap().as_slice()
		);
		assert_eq!(
			[Word("foo bar               baz"), Word("qux"), Word(" quux")],
			tokenize("'foo bar               baz' qux ' quux'").unwrap().as_slice()
		);
	}

	#[test]
	#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: \"Unclosed string literal\"")]
	fn tokenize_unclosed_string_literal_cmd() {
		tokenize("'echo foo").unwrap();
	}

	#[test]
	#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: \"Unclosed string literal\"")]
	fn tokenize_unclosed_string_literal_arg() {
		tokenize("echo 'foo bar").unwrap();
	}

	#[test]
	#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: \"Unclosed string literal\"")]
	fn tokenize_unclosed_string_literal_end() {
		tokenize("echo '").unwrap();
	}

	#[test]
	fn tokenize_proper_though_not_for_the_parser() {
		assert_eq!(
			[LeftParen, LeftParen, LeftParen, Word("echo"), Word("lol"), RightParen, RightParen, RightParen, Word("stuff")],
			tokenize("(((echo lol))) stuff").unwrap().as_slice()
		);
		assert_eq!(
			[Word("garbage"), RightParen],
			tokenize("garbage)").unwrap().as_slice()
		);
	}

	#[test]
	#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: \"Unexpected \\'(\\' in command or argument\"")]
	fn tokenize_paren_in_cmd_middle() {
		tokenize("ech(o").unwrap();
	}
	#[test]
	#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: \"Unexpected \\'(\\' in command or argument\"")]
	fn tokenize_paren_in_arg_middle() {
		tokenize("echo oops(typo").unwrap();
	}
	#[test]
	#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: \"Unexpected \\'(\\' in command or argument\"")]
	fn tokenize_paren_in_cmd_end() {
		tokenize("echo( stuff").unwrap();
	}
	#[test]
	#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: \"Unexpected \\'(\\' in command or argument\"")]
	fn tokenize_paren_in_arg_end() {
		tokenize("echo oops(").unwrap();
	}
}
