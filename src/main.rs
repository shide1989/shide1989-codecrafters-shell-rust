use anyhow::{Error, Result};
use log::debug;
use std::os::unix::fs::PermissionsExt;
use std::process::ExitCode;
use std::{env, fs};

#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::Command;

const BUILTINS: [&str; 3] = ["echo", "exit", "type"];

pub fn find_exec(binary: &str) -> String {
    let paths = env::var("PATH").unwrap_or_default();
    for path in env::split_paths(&paths) {
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
            if file_name.to_str().expect("Failed to get filename") == binary {
                let metadata = fs::metadata(&path).expect("Failed to get metadata");
                if metadata.permissions().mode() & 0o111 != 0 {
                    return String::from(path_name);
                }
            }
        }
    }

    String::from("")
}

fn exec_command(input: &str, args: &[String]) -> Result<String, Error> {
    match input {
        "exit" => std::process::exit(0),
        "echo" => Ok(format!("{}", args.join(" "))),
        "type" => {
            if BUILTINS.contains(&args[0].as_str()) {
                return Ok(format!("{} is a shell builtin", &args[0]));
            }
            let exec_path = find_exec(&args[0]);
            debug!("exec_path {:?}", exec_path);

            if exec_path.is_empty() {
                Ok(format!("{}: not found", &args[0]))
            } else {
                Ok(format!("{} is {}", &args[0], exec_path))
            }
        }
        _ => {
            let exec_path = find_exec(input);
            if exec_path.is_empty() {
                return Ok(format!("{}: command not found", input));
            }
            let mut process = Command::new(input)
                .args(args)
                .spawn()
                .expect("Failed to execute command");
            match process.wait() {
                Ok(_) => Ok(String::new()),
                Err(error) => Err(Error::new(error)),
            }
        }
    }
}

fn read() -> String {
    let mut buffer = String::new();
    print!("$ ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read line");
    buffer.trim().to_string()
}

fn eval(input: &str) -> Result<String, Error> {
    // Parse the command and args
    let values = shlex::split(input).unwrap();
    let cmd = values.first();
    if cmd.is_none() {
        return Ok(String::new());
    }
    let args = values.split_at(1).1;
    exec_command(cmd.unwrap(), args)
}

fn print(s: &str) {
    if (!s.is_empty()) {
        println!("{}", s);
    }
    io::stdout().flush().unwrap();
}

fn main() -> Result<ExitCode> {
    env_logger::init();

    loop {
        // Read: Display a prompt and wait for user input
        let input = read();
        debug!("input: {}", input);

        // Eval: Parse and execute the command
        match eval(&input) {
            Ok(output) => print(&output),
            Err(err) => {
                // Print: Display the output or error message
                print(&err.to_string());
                return Ok(ExitCode::FAILURE);
            }
        }
        //Loop again
    }
}
