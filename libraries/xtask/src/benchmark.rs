use std::{error::Error, fs, env, io::BufRead};

use duct::cmd;
use regex::Regex;

pub fn benchmark() -> Result<(), Box<dyn Error>> {
    // Build the tool with the correct settings
    cmd!("cargo", "build", "--profile", "bench", "--bin", "mcrl2rewrite").run()?;

    // Using which is a bit unnecessary, but it deals nicely with .exe on Windows and can also be used to do other searching.
    let cwd = env::current_dir()?;
    let mcrl2_rewrite = which::which_in("mcrl2rewrite", Some("target/release/"), cwd)?;

    let mcrl2_rewrite_timing = Regex::new(r#"Innermost rewrite took ([0-9]*) ms"#).unwrap();

    // Keep track of the resulting timing for every benchmark.
    let mut results = vec![];

    // Take every benchmark
    for file in fs::read_dir("examples/REC/mcrl2")? {
        let path = file?.path();

        // We take the dataspec file, and append the expressions ourselves.
        if path.extension().is_some_and(|ext| { ext == "dataspec" }) {
            let data_spec = path.clone();
            let expressions = path.with_extension("expressions");

            let benchmark = path.file_stem().unwrap();
            let benchmark = String::from(benchmark.to_string_lossy());

            println!("Benchmarking {}", benchmark);
            
            // Strip the beginning UNC path even through technically correct hyperfine does not deal with it properly.

            match cmd!("timeout",
                "600",
                &mcrl2_rewrite,
                "rewrite",
                "innermost",
                format!("{}", data_spec.to_string_lossy()), 
                format!("{}", expressions.to_string_lossy())
            )
            .stdout_capture()
            .run() {
                Ok(result) => {
                    // Parse the standard output to read the rewriting time and insert it into results.
                    for line in result.stdout.lines() {     
                        if let Some(result ) = mcrl2_rewrite_timing.captures(&line.unwrap()) {
                            let (_, [grp1]) = result.extract();                            
                            let timing: usize = grp1.parse()?;

                            println!("Benchmark {} timing {} milliseconds", benchmark, timing);
                            results.push((benchmark.clone(), timing))
                        }
                    }
                },
                Err(err) => {
                    println!("Benchmark {} timed out or crashed", benchmark);
                    results.push((benchmark.clone(), usize::MAX));

                    println!("Command failed {:?}", err);
                }
            };
        }
    }

    // Print the results in alphabetical order.
    results.sort_unstable_by(|(name, _), (name2, _)| {
        human_sort::compare(name, name2)
    });

    for (benchmark, timing) in results {        
        println!("{: >30} | {: >10}", benchmark, timing);
    }

    Ok(())
}
