use std::fs;
use std::{env, error::Error, fs::File};

use toml::{map::Map, Table, Value};

use std::io::Write;

/// Write every environment variable in the variables array.
fn write_env(writer: &mut impl Write, variables: &[&'static str]) -> Result<(), Box<dyn Error>> {
    for var in variables {
        writeln!(writer, "{} = \"{}\"", var, env::var(var).unwrap_or_default())?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    for (from, to) in env::vars() {
        println!("{} to {}", from, to);
    }

    let mut file = File::create("../../target/Compilation.toml")?;

    // Write the developement location.
    let mut table = Map::new();

    let mut sabrec = Table::new();
    sabrec.insert("path".to_string(), Value::String(fs::canonicalize(".")?.to_string_lossy().to_string()));
    table.insert("sabrec".to_string(), Value::Table(sabrec));
    
    writeln!(file, "{}", table)?;

    // Write compilation related environment variables to the configuration file.
    writeln!(file, "[env]")?;
    write_env(&mut file, &["RUSTFLAGS", "CFLAGS", "CXXFLAGS"])?;

    Ok(())
}