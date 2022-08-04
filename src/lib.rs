use std::{
    collections::HashMap,
    env, fs, io,
    process::{Command, ExitStatus},
};

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct CommandArgs {
    /// List of env var assignments to be set when running program
    #[clap(short('e'), value_parser(parse_key_val::<String, String>))]
    env_vars: Vec<(String, String)>,
    /// Entries to add to the path variable when running program
    #[clap(short('p'))]
    path_additions: Vec<String>,
    /// Program being run
    #[clap(required(true))]
    program: String,
    /// Args for program to be run
    #[clap(last(true))]
    args: Vec<String>,
}

/// Parse a single key-value pair
/// taken from <https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html>
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
pub fn run(command_args: &CommandArgs) -> Result<ExitStatus, io::Error> {
    // Convert Vec of env vars to HashMap
    let hash_map_vars: HashMap<String, String> =
        command_args.env_vars.clone().into_iter().collect();
    
    // Get shell and appropriate flag to run command through OS shell
    let (shell, flag) = get_shell_and_flag();
    // Combine flag with program and its args [flag, program name, rest of args]
    let final_args = {
        let mut args=  vec![flag.to_string(), command_args.program.clone()];
        args.extend(command_args.args.clone());
        args
    };

    // Build command with shell
    let mut command = Command::new(shell);
    // Add args for shell to run command
    command.args(&final_args)
        // Set env variables
        .envs(hash_map_vars);
    // If path_additions passed to CLI, get and set to new path
    if !command_args.path_additions.is_empty() {
        // Generate new PATH
        let new_path = get_appended_path(&command_args.path_additions);
        // Set PATH env var
        command.env("PATH", new_path);
    }
    
    // Run command and return status 
    command.status()
}

/// Generate new PATH from appending path additions to existing PATH 
fn get_appended_path(path_additions: &[String]) -> String {
    // Canonicalize paths so we can add to PATH
    let mut path_additions: Vec<String> = path_additions
        .iter()
        .flat_map(|s| {
            fs::canonicalize(s)
                .map(|can_path| can_path.to_string_lossy().to_string())
        })
        .collect();
    // get original path variable
    let original_path: Vec<String> = env::split_paths(&env::var("PATH").unwrap_or_default())
        // Convert paths to String
        .map(|path| path.to_string_lossy().to_string())
        .collect();

    // Add out additions to the beginning of the PATH
    path_additions.extend(original_path);
    
    // join paths to get our new PATH environment variable
    let new_path = env::join_paths(original_path).expect("could not join paths");
    new_path.to_string_lossy().to_string()
}

/// Return ("powershell", "-Command") for windows or ("bash", "-c") for any other OS
const fn get_shell_and_flag<'a>() -> (&'a str, &'a str) {
    if cfg!(windows) {
        ("powershell", "-Command")
    } else {
        ("bash", "-c")
    }
}