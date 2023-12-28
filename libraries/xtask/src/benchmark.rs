use std::{error::Error, fs};

use duct::cmd;

pub fn benchmark() -> Result<(), Box<dyn Error>> {
    // Build the tool with the correct settings
    cmd!("cargo", "build", "--profile", "bench", "--bin", "mcrl2rewrite").run()?;

    // Take every benchmark
    for file in fs::read_dir("examples/REC/mcrl2")? {
        let path = file?.path();

        // We take the dataspec file, and append the expressions ourselves.
        if path.extension().is_some_and(|ext| { ext == "dataspec" }) {
            let data_spec = path.clone();
            let expressions = path.with_extension("expressions");

            let benchmark = path.file_stem().unwrap();

            println!("Benchmarking {}", benchmark.to_string_lossy());
            
            // Strip the beginning UNC path even through technically correct hyperfine does not deal with it properly.

            cmd!("target/release/mcrl2rewrite.exe",
                "--rewriter",
                "innermost",
                format!("{}", data_spec.to_string_lossy()), 
                format!("{}", expressions.to_string_lossy())
            ).run()?;
        }
    }

    Ok(())
}
