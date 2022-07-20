use std::{
    collections::HashMap,
    env::Args,
    io,
    iter::Peekable,
    process::{Command, ExitStatus},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unable to parse environment variables")]
    ParseEnvironmentVariables,
    #[error("missing command")]
    MissingCommand,
    #[error("error running {0}")]
    CommandRunError(#[from] io::Error),
    #[error("{0}")]
    Other(String),
}
/// Gets command args from environment and converts them to CommandArgs
pub fn env_args() -> Result<CommandArgs, Error> {
    // Args from environment
    let args = std::env::args();
    println!("{args:?}");

    // Make peekable
    let mut peek_args = args.into_iter().peekable();
    // Read off this program's name
    peek_args.next().expect("command name to bein args");

    // Get environment variable assignments
    // [name]=[value]
    let env_vars = get_env_var_assignments(&mut peek_args)?;

    // Next arg should be command
    let command = peek_args.next().ok_or(Error::MissingCommand)?;
    // After command should be any args
    let command_args = get_command_args(peek_args);

    Ok(CommandArgs {
        env_vars,
        command,
        command_args,
    })
}

fn get_command_args(peek_args: Peekable<Args>) -> Vec<String> {
    let mut inner_args = Vec::new();
    for a in peek_args {
        inner_args.push(a);
    }
    inner_args
}

fn get_env_var_assignments<T>(arg_iter: &mut Peekable<T>) -> Result<HashMap<String, String>, Error>
where
    T: Iterator<Item = String>,
{
    let mut env_vars = HashMap::new();
    while let Some(env_assignment) = arg_iter.next_if(|arg| {
        // [arg_name]=[arg_value]
        arg.split("=").count() == 2
    }) {
        // Get var name and value to set to
        let (var_name, var_value) = {
            // Split at '='
            let mut split = env_assignment.split("=");
            // Get from Split iterator
            let name = split.next().expect("already checked for 2 long");
            let value = split.next().expect("already checked for 2 long");
            // Return
            (name.to_string(), value.to_string())
        };
        // Add to vec
        env_vars.insert(var_name, var_value);
    }
    Ok(env_vars)
}

///
pub fn run(command_args: CommandArgs) -> Result<ExitStatus, io::Error> {
    let status = Command::new(&command_args.command)
        .args(&command_args.command_args)
        .envs(&command_args.env_vars)
        .status();

    // Check if error running command
    if status.is_err() {
        // If error, try running it as a system command
        if matches!(status.as_ref().unwrap_err().kind(), io::ErrorKind::NotFound) {
            let status = run_as_system_command(&command_args.clone());
            return Ok(status?);
        }
    }
    Ok(status?)
}

/// If not found as a binary, try running it through cmd or bash directly
fn run_as_system_command(command_args: &CommandArgs) -> Result<ExitStatus, io::Error> {
    // Get shell and command line argument to run command through shell
    let (shell, flag) = if cfg!(unix) || cfg!(linux) {
        ("bash", "-c")
    } else if cfg!(windows) {
        ("powershell", "-Command")
    } else {
        unimplemented!()
    };

    let mut args: Vec<String> = vec![flag.to_string(), command_args.command.clone()];
    args.extend_from_slice(&command_args.command_args);
    println!(
        "Running command `{}` through `{shell}` with args: {args:?}",
        command_args.command
    );
    let status = Command::new(shell)
        .args(args)
        .envs(&command_args.env_vars)
        .status();
    Ok(status?)
}

#[derive(Debug, Clone)]
/// Params for env-handler
pub struct CommandArgs {
    /// list of env var assignments
    env_vars: HashMap<String, String>,
    /// command and args to be run
    command: String,
    command_args: Vec<String>,
}

// #[derive(Debug, Clone)]
// struct ArgAssignment {
//     var_name: String,
//     var_value: String,
// }

#[derive(Debug, Clone)]
struct ArgAssignment(String, String);
