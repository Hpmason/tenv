use tenv::{env_args, run, Error};
fn main() -> Result<(), Error> {
    let args = env_args().unwrap();
    println!("{args:?}");
    run(args)?;
    Ok(())
}
