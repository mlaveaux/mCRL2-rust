use std::{collections::{HashMap, HashSet}, env, error::Error, fs::{self, File}, io::{BufRead, Write}, path::Path};

use duct::cmd;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Deserialize, Serialize)]
struct MeasurementEntry {
    rewriter: String,
    benchmark_name: String,
    timing: u32,
}

#[derive(EnumString, Display, PartialEq)]
pub enum Rewriter {
    #[strum(serialize = "innermost")]
    Innermost,
    
    #[strum(serialize = "sabre")]
    Sabre,
    
    #[strum(serialize = "jitty")]
    Jitty,

    #[strum(serialize = "jittyc")]
    JittyCompiling,
}

/// Benchmarks all the REC specifications in the example folder.
/// 
/// - mcrl2 This enables benchmarking the upstream mcrl2rewrite tool 
pub fn benchmark(output_path: impl AsRef<Path>, rewriter: Rewriter) -> Result<(), Box<dyn Error>> {

    // Find the mcrl2rewrite tool based on which rewriter we want to benchmark
    let cwd = env::current_dir()?;

    let mcrl2_rewrite_path = if rewriter == Rewriter::Innermost || rewriter == Rewriter::Sabre {
        // Build the tool with the correct settings
        cmd!(
            "cargo",
            "build",
            "--profile",
            "bench",
            "--bin",
            "mcrl2rewrite"
        )
        .run()?;

        // Using which is a bit unnecessary, but it deals nicely with .exe on Windows and can also be used to do other searching.
        which::which_in("mcrl2rewrite", Some("target/release/"), cwd)?
    } else {
        which::which("mcrl2rewrite")?
    };

    let mcrl2_rewrite_timing = Regex::new(r#"Time.*:\s*([0-9\.]*) ms ±.*$"#)?;

    match rewriter {
        Rewriter::Innermost => Regex::new(r#"Innermost rewrite took ([0-9]*) ms"#)?,
        Rewriter::Sabre => Regex::new(r#"Sabre rewrite took ([0-9]*) ms"#)?,
        Rewriter::Jitty | Rewriter::JittyCompiling => {
            Regex::new(r#"rewriting: ([0-9]*) milliseconds."#)?
        }
    };

    // Create the output directory before creating the file.
    if let Some(parent) = output_path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let mut result_file = File::create(output_path)?;
    
    // Consider all the specifications in the example directory.
    for file in fs::read_dir("examples/REC/mcrl2")? {
        let path = file?.path();

        // We take the dataspec file, and append the expressions ourselves.
        if path.extension().is_some_and(|ext| ext == "dataspec") {
            let data_spec = path.clone();
            let expressions = path.with_extension("expressions");

            let benchmark_name = path.file_stem().unwrap().to_string_lossy();
            println!("Benchmarking {}", benchmark_name);
            
            // Strip the beginning UNC path even through technically correct hyperfine does not deal with it properly.
            match cmd!("timeout",
                "600",
                &mcrl2_rewrite_path,
                "-rjitty",
                "--timings",
                format!("{}", data_spec.to_string_lossy()), 
                format!("{}", expressions.to_string_lossy())
            )
            .stdout_capture()
            .stderr_capture()
            .run() {
            match rewriter {
                Rewriter::Innermost => {
                    write!(&mut command, "{}", " rewrite innermost")?;
                }
                Rewriter::Sabre => {
                    write!(&mut command, "{}", " rewrite sabre")?;
                }
                Rewriter::Jitty => {
                    write!(&mut command, "{}", " -rjitty --timings")?;
                }
                Rewriter::JittyCompiling => {
                    write!(&mut command, "{}", " -rjittyc --timings")?;
                }
            }
                Ok(result) => {
                    // Parse the standard output to read the rewriting time and insert it into results.
                    for line in result.stdout.lines().chain(result.stderr.lines()) {
                        let line = line?;
                        println!("{}", line);

                        if let Some(result) = mcrl2_rewrite_timing.captures(&line) {
                            let (_, [grp1]) = result.extract();
                            let timing: f32 = grp1.parse()?;

                            println!(
                                "Benchmark {} timing {} milliseconds",
                                benchmark_name, timing
                            );

                            // Write the output to the file and include a newline.
                            serde_json::to_writer(
                                &mut result_file,
                                &MeasurementEntry {
                                    rewriter: rewriter.to_string(),
                                    benchmark_name: benchmark_name.to_string(),
                                    timing: timing / 1000.0,
                            writeln!(&result_file)?;
                        }
                    }
                },
                Err(err) => {
                    println!("Benchmark {} timed out or crashed", benchmark_name);
                    println!("Command failed {:?}", err);
                }
            };
        }
    }

    Ok(())
}

fn print_float(value: f32) -> String {
    format!("{:.1}", value)
}

pub fn create_table(json_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let output = fs::read_to_string(json_path)?;

    // Keep track of all the read results.
    let mut results: HashMap<String, HashMap<String, f32>> = HashMap::new();

    // Figure out the list of rewriters used to print '-' values.
    let mut rewriters: HashSet<String> = HashSet::new();

    for line in output.lines() {
        let timing = serde_json::from_str::<MeasurementEntry>(line)?;

        rewriters.insert(timing.rewriter.clone());

        results
            .entry(timing.benchmark_name)
            .and_modify(|e| {
                e.insert(timing.rewriter.clone(), timing.timing);
            })
            .or_insert_with(|| {
                let mut table = HashMap::new();
                table.insert(timing.rewriter.clone(), timing.timing);
                table
            });
    }

    // Print the header of the table.
    let mut first = true;
    for rewriter in &rewriters {
        if first {
            print!("{: >30}", rewriter);
            first = false;
        } else {
            print!("{: >10} |", rewriter);            
        }
    }

    // Print the entries in the table.
    for (benchmark, result) in &results {
        print!("{: >30}", benchmark);

        for rewriter in &rewriters {
            if let Some(timing) = result.get(rewriter) {
                print!("| {: >10}", print_float(*timing));
            } else {
                print!("| {: >10}", "-");
            }
        }
        println!("");
    }

    Ok(())
}
