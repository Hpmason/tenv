use std::{
    collections::HashMap,
    io,
    process::{Command, ExitStatus},
};

use clap::Parser;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TEnvError {
    #[error("unable to parse environment variables")]
    ParseEnvironmentVariables,
    #[error("missing command")]
    MissingCommand,
    #[error("error running {0}")]
    CommandRunError(#[from] io::Error),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct CommandArgs {
    /// list of env var assignments
    #[clap(short('e'), value_parser(parse_key_val::<String, String>))]
    env_vars: Vec<(String, String)>,
    /// Command being run
    #[clap(required(true))]
    command: String,
    /// command and args to be run
    #[clap(last(true))]
    args: Vec<String>,
}

/// Parse a single key-value pair
/// taken from https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
fn parse_key_val<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

/// Runs command, while setting environment variables before unsets them after command is completed
pub fn run(command_args: CommandArgs) -> Result<ExitStatus, io::Error> {
    let hash_map_args: HashMap<String, String> =
        HashMap::from_iter(command_args.env_vars.clone().into_iter());
    let status = Command::new(&command_args.command)
        .args(&command_args.args)
        .envs(hash_map_args)
        .status();

    // Check if error running command
    if status.is_err() {
        // If error, try running it as a system command
        if matches!(status.as_ref().unwrap_err().kind(), io::ErrorKind::NotFound) {
            return run_as_system_command(&command_args);
        }
    }
    status
}

/// If not found as a binary, try running it through cmd or bash directly
fn run_as_system_command(command_args: &CommandArgs) -> Result<ExitStatus, io::Error> {
    // Get shell and command line argument to run command through shell
    let (shell, flag) = if cfg!(windows) {
        ("powershell", "-Command")
    } else {
        ("bash", "-c")
    };

    let mut args: Vec<String> = vec![flag.to_string(), command_args.command.clone()];
    args.extend_from_slice(&command_args.args);
    
    let hash_map_env: HashMap<String, String> =
        HashMap::from_iter(command_args.env_vars.clone().into_iter());
    let status = Command::new(shell).args(args).envs(hash_map_env).status();
    status
}
