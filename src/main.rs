pub mod rash;
pub mod errors;

use rash::Rash;
use errors::RashError;


fn main() {
	let mut rash = Rash::new();
	let maybe_command = rash.handle_args();
	rash.init();

	if let Some(command) = maybe_command {
		// -c mode, command passed as an argument
		rash.execute(command);
		return
	}

	let mut input = String::new();
	loop {
		rash.display_prompt();
		if let Err(e) = rash.read_line(&mut input) {
			match e {
				RashError::Eof => break,
				_ => println!("unhandled err {:?}", e)
			}
		}
		if input.len() < 1 { continue }
		rash.execute(input);
		input = String::new();
	}
}