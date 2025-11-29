use anyhow::Result;
use log::debug;

#[allow(unused_imports)]
use std::io::{self, Write};

const EXIT_COMMAND: &str = "exit";

enum Command {
    Exit(u8),
    Echo(Vec<String>),
    Type(String),
}

/// Should handle the command's execution and return (should_continue, return_code)
fn handle_command(cmd: &Command) -> (bool, u8) {
    match cmd {
        Command::Exit(code) => (false, *code),
        Command::Echo(values) => {
            println!("{}", values.join(" "));
            (true, 1)
        }
        Command::Type(value) => {
            let cmd = parse_command(value, &[]);
            if cmd.is_none() && !value.is_empty() {
                println!("{}: not found", value);
            } else if !cmd.is_none() {
                println!("{} is a shell builtin", value);
            }
            (true, 1)
        }
        _ => (true, 1),
    }
}

fn parse_command(input: &str, args: &[String]) -> Option<Command> {
    match input {
        EXIT_COMMAND => Some(Command::Exit(0)),
        val => match val {
            "echo" => Some(Command::Echo(Vec::from(args))),
            "type" => Some(Command::Type(String::from(
                args.first().unwrap_or(&"".to_string()),
            ))),
            _ => None,
        },
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let mut buffer = String::new();
    let mut next = true;
    let mut return_code: u8 = 0;

    while next {
        // Read: Display a prompt and wait for user input
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line");
        let input = buffer.trim();

        debug!("input: {}", input);

        // Eval: Parse and execute the command
        let values = shlex::split(input).unwrap();
        let cmd = values.first();
        if cmd.is_none() {
            continue;
        }
        let cmd = cmd.unwrap();
        let args = values.split_at(1).1;
        match parse_command(cmd, &args) {
            None => {
                if !cmd.is_empty() {
                    println!("{}: command not found", cmd);
                }
            }
            Some(cmd) => {
                (next, return_code) = handle_command(&cmd);
            }
        }

        // Print: Display the output or error message
        buffer.clear();

        //Loop again
    }
    debug!("return_code: {}", return_code);
    Ok(())
}
