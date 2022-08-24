use std::{
    collections::HashMap,
    env, io,
    process::{Command, ExitStatus}, path::PathBuf, ffi::OsString,
};

use clap::Parser;

#[derive(Debug, Parser)]
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

pub(crate) mod helpers {
    use std::ffi::OsString;

    /// Expand path if it is an argfile arg
    pub fn expand_argfile_path(arg: String) -> OsString {
        // If it starts with the prefix, it is an argfile arg
        if arg.starts_with(argfile::PREFIX) {
            // get path by itself
            let path = arg.strip_prefix(argfile::PREFIX).expect("Already checked for this prefix");
            // Expand path
            if let Ok(expanded) = shellexpand::full(&path) {
                // re-add the prefix and convert to OsString
                return OsString::from(format!("{}{}", argfile::PREFIX, expanded))
            }
        }
        // If not an argfile arg or error expanding, just return arg as an OsString
        arg.into()
    }
    /// Return true if line would be a comment if in an argfile
    pub fn is_argfile_comment(arg: &OsString) -> bool {
        arg.to_string_lossy().starts_with('#')
    }
}

impl CommandArgs {
    /// Get list of args for `std::process::Command` (i.e. ["-c", `program_name`, ...`args`])
    pub fn get_all_args() -> Result<Self, io::Error> {
        // Expand all args and convert to OsString
        let args_plain_iter = env::args()
            .map(helpers::expand_argfile_path);
        
        // Get args and parse any from argfile if provided
        let mut args_with_argfile = argfile::expand_args_from(
            args_plain_iter,
            argfile::parse_fromfile,
            argfile::PREFIX,
        )?;
        
        // Filter out comments (lines starting with '#')
        args_with_argfile = args_with_argfile
            .into_iter()
            .filter(|arg| !helpers::is_argfile_comment(arg))
            .collect();
        
        // Get CLI args (parse_from is from `clap::derive::Parser`)
        let mut command_args: Self = Self::parse_from(args_with_argfile);
        // Trim any spaces in env var name
        command_args.env_vars.iter_mut().for_each(|(k, _v)| {
            *k = k.trim().to_string();
        });
        // Trim any spaces in path
        command_args.path_additions.iter_mut().for_each(|path| {
            *path = path.trim().to_string();
        });
        Ok(command_args)
    }
    /// Generate new PATH by prepending path additions to existing PATH
    fn get_prepended_path(&self, env_vars: &Option<HashMap<String, String>>) -> Option<OsString> {
        if self.path_additions.is_empty() {
            return None;
        }
        // Canonicalize paths so we can add to PATH
        let mut path_additions: Vec<String> = self.path_additions
            .iter()
            .map(|s| {
                let abso_path = dunce::simplified(&PathBuf::from(s))
                    .to_string_lossy()
                    // Sometimes there is white space at the beginning or end
                    .trim()
                    .to_string();
                
                // Expand ~ and other env vars
                let expanded = shellexpand::full_with_context_no_errors::<String, _, _, PathBuf, _>(
                    &abso_path,
                    || {None},
                    |var_name| {
                        if let Some(env_map) = &env_vars {
                            // Get from our HashMap
                            env_map.get(var_name)
                        }
                        else {
                            None
                        }
                    }
                );
                expanded.to_string()
            })
            .collect();
        // get original path variable
        let original_path: Vec<String> = env::split_paths(&env::var("PATH").unwrap_or_default())
            // Convert paths to String
            .map(|path| path.to_string_lossy().to_string())
            .collect();

        // Add PATH to the end of the path additions
        path_additions.extend(original_path);

        // join paths to get our new PATH environment variable
        let new_path = env::join_paths(path_additions).expect("could not join paths");
        Some(new_path)
    }

    fn get_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars_hashmap = HashMap::new();
        for (var_name, value) in self.env_vars.clone() {
            let expanded = shellexpand::full_with_context_no_errors::<String, _, _, PathBuf, _>(
                &value, 
                dirs::home_dir,
                |key| {
                    env_vars_hashmap.get(key)
                } 
            );
            env_vars_hashmap.insert(var_name, expanded.to_string());
        }
        env_vars_hashmap
    }

    fn get_arg_list(&self) -> Vec<String> {
        let (_, flag) = get_shell_and_flag();
        let mut args = vec![flag.to_string(), self.program.clone()];
        args.extend(self.args.clone());
        args
    }
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

/// ctrl+c handler so that tenv itself can't be interrupted.
/// Commands run by tenv can still be cancelled, therefore ending the execution of tenv.
/// Not doing this causes problems with starship on powershell.
pub fn init_ctrlc_handler() -> Result<(), ctrlc::Error> {
    // Set empty ctrl+c handler, just so doesn't stop tenv itself when ctrl+c entered
    ctrlc::set_handler(move || {
        // println!("ctrl+c pressed");
    })
}

/// Runs command, while setting environment variables before unsets them after command is completed
pub fn run(command_args: &CommandArgs) -> Result<ExitStatus, io::Error> {
    // Convert Vec of env vars to HashMap
    let hash_map_vars: HashMap<String, String> = command_args.get_env_vars();

    // Get shell and appropriate flag to run command through OS shell
    let (shell, _) = get_shell_and_flag();
    // Combine flag with program and its args [flag, program name, rest of args]
    let final_args = command_args.get_arg_list();

    // Build command with shell
    let mut command = Command::new(shell);
    // Add args for shell to run command
    command
        .args(&final_args)
        // Set env variables
        .envs(&hash_map_vars);
    
    // If path_additions passed to CLI, get and set to new path
    if let Some(new_path) = command_args.get_prepended_path(&Some(hash_map_vars)) {
        // Set PATH env var
        command.env("PATH", new_path);
    }

    // Run command and return status
    command.status()
}



/// Return ("powershell", "-Command") for windows or ("bash", "-c") for any other OS
const fn get_shell_and_flag<'a>() -> (&'a str, &'a str) {
    if cfg!(windows) {
        ("powershell", "-Command")
    } else {
        ("bash", "-c")
    }
}
