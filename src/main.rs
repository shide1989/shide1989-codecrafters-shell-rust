use anyhow::Result;
use std::fmt::Display;
use std::io::{self, Error, ErrorKind, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::process::ExitCode;
use std::{env, fs};

const BUILTINS: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];

pub fn find_exec(binary: &str) -> String {
    let paths = env::var("PATH").unwrap_or_default();
    for path in env::split_paths(&paths) {
        if !path.exists() {
            continue;
        }

        for entry in fs::read_dir(path).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = entry.file_name();
            if file_name.display().to_string() == binary {
                let metadata = fs::metadata(&path).expect("Failed to get metadata");
                if metadata.permissions().mode() & 0o111 != 0 {
                    return String::from(path.display().to_string());
                }
            }
        }
    }

    String::from("")
}

fn exec_command(input: &str, args: &Vec<String>) -> Result<String, Error> {
    match input {
        "exit" => std::process::exit(0),
        "echo" => Ok(format!("{}", args.join(" "))),
        "pwd" => Ok(env::current_dir()?.display().to_string()),
        "cd" => {
            let hd = env::home_dir().expect("Failed to get home directory");
            let path = if args[0] == "~" {
                Path::new(hd.to_str().unwrap())
            } else {
                Path::new(&args[0])
            };
            env::set_current_dir(path).map_err(|_| {
                Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "{input}: {}: No such file or directory",
                        path.to_str().unwrap()
                    ),
                )
            })?;
            Ok(String::new())
        }
        "type" => {
            if BUILTINS.contains(&args[0].as_str()) {
                return Ok(format!("{} is a shell builtin", &args[0]));
            }
            let exec_path = find_exec(&args[0]);
            if exec_path.is_empty() {
                Err(Error::new(
                    ErrorKind::NotFound,
                    format!("{}: not found", &args[0]),
                ))
            } else {
                Ok(format!("{} is {}", &args[0], exec_path))
            }
        }
        _ => {
            let exec_path = find_exec(input);
            if exec_path.is_empty() {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("{input}: command not found"),
                ));
            }
            let mut process = Command::new(input)
                .args(args)
                .spawn()
                .expect("Failed to execute command");
            match process.wait() {
                Ok(_) => Ok(String::new()),
                Err(error) => Err(error),
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

fn parse(input: &str) -> (Option<String>, Vec<String>) {
    // Parse the command and args
    let values = match shlex::split(input) {
        Some(values) => values,
        None => return (None, Vec::new()),
    };

    let cmd = values.first().cloned();
    if cmd.is_none() {
        return (None, Vec::new());
    }
    let args = values.split_at(1).1.to_vec();
    (cmd, args)
}

fn print(s: &str) {
    if !s.is_empty() {
        println!("{s}");
    }
    io::stdout().flush().unwrap();
}

fn main() -> Result<ExitCode> {
    env_logger::init();

    loop {
        // Read: Display a prompt and wait for user input
        let input = read();

        let (cmd, args) = parse(input.trim());

        let cmd = match cmd {
            Some(cmd) => cmd,
            None => continue,
        };

        // Eval: Parse and execute the command
        match exec_command(&cmd, &args) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{output}");
                }
                io::stdout().flush()?;
            }
            Err(err) => {
                // Print: Display the output or error message
                print(format!("{}", err).as_str());
                // return Ok(ExitCode::FAILURE);
            }
        }
        //Loop again
    }
}
