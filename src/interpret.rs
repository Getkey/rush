extern crate builtins;

use std::str::Chars;


pub fn read(command_line: &str) {
	let mut tree = parse_expression(&mut command_line.chars());

	println!("{}", tree.execute());
}

fn parse_expression(char_it: &mut Chars) -> Command {
	let mut cmd = Command::new();

	let mut tokens_are_args = false;
	let mut in_leading_whitespace = true;
	let mut current_arg: Option<String> = None;

	while let Some(char) = char_it.next() {
		match char {
			'(' => cmd.params.push(Param::Cmd(parse_expression(char_it))),
			')' => return cmd,
			' ' | '\t' => { // TODO: add support for other types of whitespace
				if !in_leading_whitespace { // ignore leading whitespace
					tokens_are_args = true;
				} else if let Some(arg_str) = current_arg {
					current_arg = None;
					cmd.params.push(Param::Arg(arg_str));
				}
			},
			_ => {
				in_leading_whitespace = false;
				if tokens_are_args {
					match current_arg {
						Some(ref mut arg_str) => arg_str.push(char),
						None => {
							let mut arg_str = String::new();
							arg_str.push(char);
							current_arg = Option::Some(arg_str);
						},
					}
				} else {
					cmd.cmd.push(char) //TODO: use a &str instead by marking the first `_` byte, more efficient
				}
			},
		}
	}

	cmd
}

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
	fn execute(&mut self) -> String {//returns stdout - possibly return a array of strings?
		let mut final_arglist: Vec<&str> = Vec::with_capacity(self.params.len());

		for arg in self.params.iter_mut() { // substitue subcommands recursively
			let stdout = if let Param::Cmd(ref mut subcommand) = *arg {
				Some(subcommand.execute())
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

		let cmd_str: &str = &self.cmd; // does not infer automatically as of rust 1.9
		match cmd_str {
			"" => {},
			/*"cd" => builtins::cd(&self.params),
			"exit" => builtins::exit(&self.params),*/
			_ => invoke(&self.cmd, &final_arglist),
		};

		"this is a sample stdout return".to_string()
	}
}
enum Param {
	Arg(String),
	Cmd(Command),

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
