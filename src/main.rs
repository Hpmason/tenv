use clap::Parser;
use tenv::{run, CommandArgs, TEnvError};
fn main() -> Result<(), TEnvError> {
    let args: CommandArgs = CommandArgs::parse();
    // println!("{args:?}");
    if let Err(e) = run(args) {
        println!("Error running command with TEnv: {e}");
    }
    Ok(())
}
