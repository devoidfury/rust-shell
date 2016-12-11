extern crate libc;

use std::io;
use std::io::prelude::*;
use std::process;
use std::collections::BTreeMap;

use errors::RashError;


const SHELL_NAME: &'static str = "rash";
const STDIN_FILENO: libc::c_int = libc::STDIN_FILENO;
const STDOUT_FILENO: libc::c_int = libc::STDOUT_FILENO;


pub struct Rash {
	pub context: BTreeMap<String, String>,
	pub last_return: i32,
	pub interactive: bool
}

pub fn isatty(fd: libc::c_int) -> bool { unsafe { libc::isatty(fd) != 0 } }

impl Rash {
	pub fn new() -> Rash {
		let context = BTreeMap::new();
		let interactive = isatty(STDIN_FILENO);
		Rash { context: context, last_return: 0, interactive: interactive }
	}

	pub fn init(&mut self) {
		// todo: initialize context
		if self.interactive {
			// todo: load history
			println!("the {} you actually want!\n", SHELL_NAME);
			//println!("isatty {:?}", isatty(STDIN_FILENO));
		}
	}

	pub fn teardown(&mut self) {
		// todo: save history in interactive mode
		io::stdout().flush().unwrap();
		process::exit(self.last_return);
	}

	pub fn display_prompt(&mut self) {
		if self.interactive {
			io::stdout().flush().unwrap();
			print!("$ ");
		}
		io::stdout().flush().unwrap();
	}

	pub fn read_line(&mut self, buff: &mut String) -> Result<usize, RashError> {
		// if self.interactive {
		// 		self.read_line_interactive(buff)
		// } else {
		self.read_line_direct(buff)
		// }
	}

	fn read_line_direct(&mut self, buff: &mut String) -> Result<usize, RashError> {
		let stdin = io::stdin();
		let mut handle = stdin.lock();
		match handle.read_line(buff) {
			Ok(size) => {
				if size > 0 {
					let buff_copy = buff.clone();
					buff.clear();
					buff.push_str(buff_copy.trim());
					Ok(size)
				} else {
					Err(RashError::Eof)
				}
			},
			Err(e) => Err(RashError::Io(e))
		}
	}

	// fn read_line_interactive(&mut self, buff: &mut String) -> Result<usize, io::Error> {
	// 	let prompt = rash.prompt();
	// }

	pub fn execute(&mut self, input: String) {
		let mut raw_args: Vec<&str> = input.trim().split(" ").collect();
		let command_arg = raw_args.remove(0);
		let mut cmd = process::Command::new(command_arg);

		if raw_args.len() > 0 {
			cmd.args(raw_args.as_slice());
		}

		let mut child = match cmd.spawn() {
			Ok(st) => st,
			Err(e) => {
				match e.raw_os_error() {
					Some(2) => {
						self.last_return = 127;
						println!("{}: command not found: {}", SHELL_NAME, command_arg);
						return
					},
					Some(errno) => {
						self.last_return = errno;
						println!("unexpected errno {}\n  err: {}", errno, e);
						return
					},
					None => {
						println!("err {:?}", e);
						self.last_return = 1;
						return
					}
				}
			}
		};

		let ecode = child.wait().expect("failed to wait on child");
		self.last_return = ecode.code().unwrap();
	}
}
