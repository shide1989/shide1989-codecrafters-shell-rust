use anyhow::{Error, Result};
use bytes::{BufMut, BytesMut};
use log::debug;

#[allow(unused_imports)]
use std::io::{self, Write};

const VALID_COMMANDS: [&str; 1] = ["echo"];
const EXIT_COMMAND: &str = "exit";

fn handle_command(input: &str) -> (bool, u8) {
    match input {
        EXIT_COMMAND => (false, 0u8),
        val => {
            if !VALID_COMMANDS.contains(&val) {
                println!("{}: command not found", input);
                return (true, 0u8);
            }
            (true, 0u8)
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let mut buffer = String::new();
    let mut next = true;
    let mut return_code: u8 = 0;

    while next {
        // Read
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line");
        let input = buffer.trim();

        debug!("input: {}", input);
        // Eval

        match shlex::split(input) {
            Some(args) => {
                debug!("args: {:?}", args);
                for arg in args {
                    (next, return_code) = handle_command(arg.as_str());
                    debug!("next: {:?}", next);
                }
            }
            None => {}
        }
        //Print
        buffer.clear();

        //Loop again
    }
    debug!("return_code: {}", return_code);
    Ok(())
}
