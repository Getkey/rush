macro_rules! println_stderr {
	($($arg:tt)*) => ({
		use std::io::Write;

		match writeln!(std::io::stderr(), $($arg)*) {
			Ok(_) => {},
			Err(err) => panic!("{}", err),
		}

	})
}
macro_rules! print_flush {
	($($arg:tt)*) => ({
		use std::io;
		use std::io::Write;

		print!($($arg)*);
		io::stdout().flush().ok();
	})
}
