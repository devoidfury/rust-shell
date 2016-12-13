extern crate libc;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;
use std::os::unix::io::IntoRawFd;

use errors::RashError;


const SHELL_NAME: &'static str = "rash";


pub fn isatty(fd: libc::c_int) -> bool { unsafe { libc::isatty(fd) != 0 } }


pub struct Rash {
	pub context: HashMap<String, String>,
	pub last_return: i32,
	pub interactive: bool
}

impl Rash {
	pub fn new() -> Rash {
		let context = HashMap::new();
		let interactive = isatty(libc::STDIN_FILENO);
		Rash {
			context: context,
			last_return: 0,
			interactive: interactive
		}
	}

	pub fn init(&mut self) {
		// todo: initialize context
		if self.interactive {
			// todo: load history
			println!("the {} you actually want!\n", SHELL_NAME);
			//println!("isatty {:?}", isatty(STDIN_FILENO));
		}
	}

	pub fn handle_args(&mut self) -> Option<String> {
		let mut args = env::args();
		self.context.insert("0".to_string(), args.next().unwrap());

		if args.len() == 0 { return None }

		let arg = args.next().unwrap();

		let mut immediate = false;
		let mut immediate_command = None;
		let mut opt_end = false;

		let load_file = match arg.as_ref() {
			// command mode; executes command from argument and exits
			"-c" => {
				self.interactive = false;
				immediate = true;
				None
			},
			// explicitly read commands from stdin; this is the default
			"-s" => None,
			// explicitly set interactive mode
			"-i" => {
				self.interactive = true;
				None
			},
			// `--` as first arg is to be consumed and marks end of options
			"--" => { opt_end = true; None },
			// `-` as first arg is to be consumed and ignored
			"-" => None,
			_ => {
				let arg_ = arg.clone();
				let mut chars = arg_.chars();
				match chars.next().unwrap() {
					// todo: `set` options
					'-' | '+' => {
						None
					},
					// this is for executing from a file; `rash somescript.sh`
					_ => Some(arg)
				}
			}
		};

		if let Some(input_filename) = load_file {
			self.load_input_file(&input_filename);
		}

		let mut argcount = 1;
		let mut maybe_command_name = false;

		while let Some(nextarg) = args.next() {
			// handle arg
			let arg_ = nextarg.clone();
			let mut chars = arg_.chars();
			match chars.next().unwrap() {
				c @ '-' | c @ '+' if !opt_end => {
					match chars.next() {
						// `--` marks end of shell options
						Some('-') if c == '-' => { opt_end = true; },
						Some(_) => {
							// todo: `set` options
						},
						// undefined behavior, just ignore
						None => {}
					}
					// todo: `set` options
				},
				// command mode, handle the command argument
				// after this we're done with sh options, all command_name or positional
				_ if immediate => {
					opt_end = true;
					immediate = false;
					immediate_command = Some(nextarg);
					maybe_command_name = true;
				}
				_ if maybe_command_name => {
					// handle optional command_name arg in -c mode
					maybe_command_name = false;
					*self.context.get_mut("0").unwrap() = nextarg;
				}
				_ => {
					// other arguments as positional parameters; sets $1, $2, ...
					self.context.insert(argcount.to_string(), nextarg);
					argcount += 1;
				}
			}
		}

		if immediate {
			println!("{}: -c: option requires an argument", SHELL_NAME);
			self.last_return = 2;
			process::exit(self.last_return);
		}

		immediate_command
	}

	// this is the load mechanism for executing from a file; `rash somescript.sh`
	fn load_input_file(&mut self, input_filename: &str) {
		self.interactive = false;
		// println!("opening file {}", input_filename);
		let err = match File::open(input_filename) {
			Ok(f) => {
				let input_fd = f.into_raw_fd();
				unsafe { libc::dup2(input_fd, libc::STDIN_FILENO); }
				return
			},
			Err(e) => e
		};

		match err.raw_os_error() {
			Some(2) => {
				self.last_return = 127;
				println!("{}: no such file or directory: {}", SHELL_NAME, input_filename);
			},
			Some(errno) => {
				self.last_return = errno;
				println!("unexpected errno {}\n  err: {}", errno, err);
			},
			None => {
				self.last_return = 2;
				println!("err {:?}", err);
			}
		}
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
			Ok(size) if size > 0 => {
				let buff_copy = buff.clone();
				buff.clear();
				buff.push_str(buff_copy.trim());
				Ok(size)
			},
			Ok(_) => Err(RashError::Eof),
			Err(e) => Err(RashError::Io(e))
		}
	}

	// fn read_line_interactive(&mut self, buff: &mut String) -> Result<usize, io::Error> {
	// 	let prompt = rash.prompt();
	// }

	pub fn execute(&mut self, input: String) {
		// todo: implement proper sh lex
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

impl Drop for Rash {
	fn drop(&mut self) {
		// todo: save history in interactive mode
		io::stdout().flush().unwrap();
		process::exit(self.last_return);
	}
}
