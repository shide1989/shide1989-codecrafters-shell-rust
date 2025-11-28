#[allow(unused_imports)]
use std::io::{self, Write};

const VALID_COMMANDS: [&str;1] = ["echo"];

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Failed to read line");
    let input = buffer.trim();
    if input.is_empty() {
        return;
    }

    match shlex::split(input) {
        Some(args) => {
            for arg in args {
                if !VALID_COMMANDS.contains(&arg.as_str()){
                    print!("{}: command not found", arg);
                }
            }
        },
        None => {}
    }
}
