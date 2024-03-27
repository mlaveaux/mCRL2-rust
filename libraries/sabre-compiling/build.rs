use std::{env, error::Error, fs::File};

use std::io::Write;

/// Write every environment variable in the variables array.
fn write_env(writer: &mut impl Write, variables: &[&'static str]) -> Result<(), Box<dyn Error>> {
    for var in variables {
        writeln!(writer, "{} = \"{}\"", var, env::var(var).unwrap_or_default())?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Write compiler related environment variables to a configuration file.

    for (from, to) in env::vars() {
        println!("{} to {}", from, to);
    }

    let mut file = File::create("./Compilation.toml")?;
    writeln!(file, "[env]")?;
    write_env(&mut file, &["RUSTFLAGS", "CFLAGS", "CXXFLAGS"])?;

    Ok(())
}