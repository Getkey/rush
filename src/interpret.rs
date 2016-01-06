extern crate builtins;

pub fn read(command_line: &str) {
	if let Some((command, expressions)) = parse(command_line) {
		match command {
			"" => {},
			"cd" => builtins::cd(&expressions),
			"exit" => builtins::exit(&expressions),
			_ => invoke(command, &expressions),
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
