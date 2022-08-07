use clap::Parser;
use tenv::{init_ctrlc_handler, run, CommandArgs};

fn main() {
    // Get CLI args
    let args: CommandArgs = CommandArgs::parse();
    // Initialize control+C handler. If error, just let user know it failed to set handler
    if init_ctrlc_handler().is_err() {
        println!("tenv: could not set ctrl+c handler, so issues may arise if ctrl+c is entered");
    }
    // Run program and print error if there is one
    if let Err(e) = run(&args) {
        println!("Error running command with TEnv: {e}");
    }
}
