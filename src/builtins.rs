pub fn cd(path: &str) {
	use std::env;
	use std::path::Path;

	match env::set_current_dir(Path::new(path)) {
		Ok(_) => {},
		Err(err) => println!("{}", err),
	}
}

pub fn exit(status: &str) {
	match status.parse::<u8>() {
		Ok(sta) => println!("{}", sta),
		Err(err) => println!("{}", err),
	}
}
