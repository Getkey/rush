/* TODO: a neat à la fish history

	use std::env;
	use std::path;

	use std::io::{Read, Write};
	use std::fs;

	let history_path = match env::var("XDG_CONFIG_HOME") {
		Ok(config_path) => {
			let mut config_path = path::PathBuf::from(config_path);
			config_path.push("rush/history.txt");
			config_path
		},
		Err(_) => match env::home_dir() {
			Some(mut home_path) => {
				home_path.push(".config/rush/history.txt");
				home_path
			},
			None => path::PathBuf::from(".config/rush/history.txt"),
		},
	};
	let mut history_dirpath = history_path.clone();
	history_dirpath.pop();
	fs::DirBuilder::new()
		.recursive(true)
		.create(history_dirpath);
	let mut history_file = fs::OpenOptions::new()
		.read(true)
		.write(true)
		.append(true)
		.create(true)
		.open(history_path)
		.unwrap();//TODO: clean this up
	let mut history = io::BufReader::new(&history_file).lines();

*/

pub struct History {
	history: Vec<String>,
	pos: usize,
}
impl History {
	pub fn new() -> History {
		History {
			history: Vec::new(),
			pos: 0,
		}
	}
	pub fn push(&mut self, line: String) {
		self.history.push(line);
		self.pos = self.history.len();
	}
	pub fn previous(&mut self) {
		if self.pos != 0 {
			self.pos = self.pos - 1;
		}
	}
	pub fn next(&mut self) {
		if self.pos != self.history.len() {
			self.pos = self.pos + 1;
		}
	}
	pub fn get_line(&self) -> Option<&str> {
		if self.pos == self.history.len() {
			None
		} else {
			Some(&self.history[self.pos])
		}
	}
}
