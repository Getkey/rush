macro_rules! print_flush {
	($($arg:tt)*) => ({
		use std::io;
		use std::io::Write;

		print!($($arg)*);
		io::stdout().flush().ok();
	})
}
