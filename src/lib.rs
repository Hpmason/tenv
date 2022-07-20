use std::{
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

fn get_env_var_assignments<T>(arg_iter: &mut Peekable<T>) -> Result<Vec<ArgAssignment>, Error>
where
    T: Iterator<Item = String>,
{
    let mut env_vars = Vec::new();
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
        env_vars.push(ArgAssignment {
            var_name,
            var_value,
        });
    }
    Ok(env_vars)
}

///
pub fn run(command_args: CommandArgs) -> Result<ExitStatus, io::Error> {
    set_args(&command_args.env_vars);
    let status = Command::new(command_args.command)
        .args(command_args.command_args)
        .status();
    unset_args(&command_args.env_vars);
    Ok(status?)
}

fn set_args(var_assignments: &Vec<ArgAssignment>) {
    for assignment in var_assignments {
        std::env::set_var(&assignment.var_name, &assignment.var_value);
    }
}

fn unset_args(var_assignments: &Vec<ArgAssignment>) {
    for assignment in var_assignments {
        std::env::remove_var(&assignment.var_name);
    }
}

#[derive(Debug)]
/// Params for env-handler
pub struct CommandArgs {
    /// list of env var assignments
    env_vars: Vec<ArgAssignment>,
    /// command and args to be run
    command: String,
    command_args: Vec<String>,
}

#[derive(Debug)]
struct ArgAssignment {
    var_name: String,
    var_value: String,
}
