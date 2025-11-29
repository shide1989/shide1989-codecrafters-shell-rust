use anyhow::Result;
use log::debug;
use std::env;
use std::fs;
use std::process::ExitCode;

#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;

const BUILTINS: [&str; 3] = ["echo", "exit", "type"];
const EXIT_COMMAND: &str = "exit";

#[derive(Debug)]
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
            if BUILTINS.contains(&value.as_str()) {
                println!("{} is a shell builtin", value);
                return (true, 1);
            }
            let bin_path = find_exec(value);
            debug!("bin_path {:?}", bin_path);
            if bin_path.is_empty() {
                println!("{}: not found", value);
            } else {
                println!("{} is {}", value, bin_path);
            }
            (true, 1)
        }
    }
}
fn find_exec(binary: &String) -> String {
    let paths = env::var("PATH").unwrap_or_default();
    for path in env::split_paths(&paths) {
        debug!(
            "Searching for binary {} in {}",
            binary,
            path.to_str().unwrap()
        );
        if !path.exists() {
            continue;
        }
        for entry in fs::read_dir(path).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            let path_name = path.to_str().expect("Failed to convert path to string");
            if !path.is_file() {
                continue;
            }
            let file_name = entry.file_name();
            debug!("testing path {:?} with file_name {:?}", path, file_name);
            if file_name.to_str().expect("Failed to get filename") == binary {
                let metadata = fs::metadata(&path).expect("Failed to get metadata");
                debug!(
                    "{:0o} metadata permissions",
                    metadata.permissions().mode() & 0o111
                );

                if metadata.permissions().mode() & 0o111 != 0 {
                    return String::from(path_name);
                }
            }
        }
    }

    String::from("")
}
fn parse_command(input: &str, args: &[String]) -> Option<Command> {
    match input {
        EXIT_COMMAND => Some(Command::Exit(0)),
        val => match val {
            "echo" => Some(Command::Echo(Vec::from(args))),
            "type" => Some(Command::Type(args.first()?.clone())),
            _ => None,
        },
    }
}

fn main() -> Result<ExitCode> {
    env_logger::init();

    let mut buffer = String::new();
    let mut next = true;
    let mut return_code: u8 = 0;

    while next {
        // Read: Display a prompt and wait for user input
        print!("$ ");
        io::stdout().flush()?;
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
                debug!("{:?}", cmd);
                (next, return_code) = handle_command(&cmd);
            }
        }

        // Print: Display the output or error message
        buffer.clear();

        //Loop again
    }
    debug!("return_code: {}", return_code);
    Ok(ExitCode::from(return_code))
}
