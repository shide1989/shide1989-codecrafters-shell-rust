use bytes::{BufMut, BytesMut};
use log::debug;
#[allow(unused_imports)]
use std::io::{self, Write};

const VALID_COMMANDS: [&str; 1] = ["echo"];

fn main() {
    env_logger::init();

    let mut buffer = String::new();
    while true {
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line");
        let input = buffer.trim();
        if input.is_empty() {
            return;
        }

        debug!("input: {}", input);

        match shlex::split(input) {
            Some(args) => {
                debug!("args: {:?}", args);
                for arg in args {
                    if !VALID_COMMANDS.contains(&arg.as_str()) {
                        println!("{}: command not found", arg);
                    }
                }
            }
            None => {}
        }
        buffer.clear();
    }
}
