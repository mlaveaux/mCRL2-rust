use std::{error::Error, fs::{self, File}, io::Write, path::{Path, PathBuf}};

use duct::{cmd, Expression};
use indoc::indoc;
use libloading::{Library, Symbol};
use log::info;
use mcrl2::data::DataExpression;
use sabre::{set_automaton::SetAutomaton, RewriteEngine, RewriteSpecification};
use temp_dir::TempDir;
use toml::{map::Map, Table, Value};

pub struct SabreCompiling {
    library: Library,
    //rewrite_func: Symbol<unsafe extern fn() -> u32>,
}

impl RewriteEngine for SabreCompiling {
    fn rewrite(&mut self, term: DataExpression) -> DataExpression {
        term
    }
}

/// Apply the value from compilation_toml for every given variable as an environment variable.
fn apply_env(builder: Expression, compilation_toml: &Map<String, Value>, variables: &[&'static str]) -> Result<Expression, Box<dyn Error>>
{
    let mut result = builder;
    let env = compilation_toml.get("env").ok_or("Missing [env] table")?;

    for var in variables {
        let value = env.get(*var).ok_or("Missing var")?.as_str().ok_or("Not a string")?;

        info!("Setting environment variable {} = {}", var, value);
        result = result.env(var, value);
    }

    Ok(result)
}

impl SabreCompiling {
    
    pub fn new(spec: &RewriteSpecification, use_local_tmp: bool) -> Result<SabreCompiling, Box<dyn Error>> {
        let apma = SetAutomaton::new(spec, |_| (), true);

        let compilation_toml = include_str!("../Compilation.toml").parse::<Table>()?;

        // Create the directory structure for a Cargo project
        let system_tmp_dir = TempDir::new()?;
        let temp_dir = if use_local_tmp {
            &Path::new("./tmp")
        } else {
            system_tmp_dir.path()
        };

        info!("Compiling sabre into directory {}", temp_dir.to_string_lossy());
        let source_dir = PathBuf::from(temp_dir).join("src");

        if !temp_dir.exists() {
            fs::create_dir(&temp_dir)?;
        }

        if !source_dir.exists() {
            fs::create_dir(&source_dir)?;
        }

        // Write the cargo configuration
        {
            let mut file = File::create(PathBuf::from(temp_dir).join("Cargo.toml"))?;
            writeln!(&mut file, indoc! {"
                [package]
                name = \"sabre-generated\"
                edition = \"2021\"
                rust-version = \"1.70.0\"
                version = \"0.1.0\"

                [workspace]
                
                [dependencies]
                
                [lib]
                crate-type = [\"cdylib\", \"rlib\"]            
            "})?;
        }        

        // Ignore the created package.
        {
            let mut file = File::create(PathBuf::from(temp_dir).join(".gitignore"))?;
            writeln!(&mut file, "*.*")?;
        }

        // Write the output source file(s).
        {
            let mut file = File::create(source_dir.join("lib.rs"))?;
            writeln!(&mut file, indoc! {"
                #[no_mangle]
                pub unsafe extern \"C\" fn rewrite_term() {{
                    println!(\"Hello world!\");
                }}
            "})?;
        }

        // Compile the dynamic object.
        info!("Compiling...");
        let mut expr = cmd("cargo", &["build", "--lib"])
            .dir(temp_dir);
        expr = apply_env(expr, &compilation_toml, &["RUSTFLAGS", "CFLAGS", "CXXFLAGS"])?;
        expr.run()?;

        println!("finished.");

        // Figure out the path to the library (it is based on platform: linux, windows and then macos)
        let mut path = PathBuf::from(temp_dir).join("./target/debug/libsabre_generated.so");
        if !path.exists() {
            path = PathBuf::from(temp_dir).join("./target/debug/sabre_generated.dll");
            if !path.exists() {
                path = PathBuf::from(temp_dir).join("./target/debug/libsabre_generated.dylib");
                if !path.exists() {
                    return Err("Could not find the compiled library!".into());
                }
            }
        }

        // Load it back in and call the rewriter.
        unsafe {
            let library = Library::new(&path)?;
            let func: Symbol<extern fn()> = library.get(b"rewrite_term")?;
            
            func();

            Ok(SabreCompiling {
                library,    
            })
        }


    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_log::test;

    #[test]
    fn test_compilation() {
        let spec = RewriteSpecification::default();

        SabreCompiling::new(&spec, true).unwrap();
    }
}