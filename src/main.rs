use clap::Parser;
use tenv::{run, CommandArgs};

fn main() {
    // Get CLI args
    let args: CommandArgs = CommandArgs::parse();
    // Run program and print error if there is one
    if let Err(e) = run(&args) {
        println!("Error running command with TEnv: {e}");
    }
}
