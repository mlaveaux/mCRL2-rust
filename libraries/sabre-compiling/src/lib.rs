use std::{error::Error, fs::{self, File}, io::Write, path::PathBuf};

use duct::cmd;
use indoc::indoc;
use libloading::{Library, Symbol};
use log::info;
use mcrl2::data::DataExpression;
use sabre::{set_automaton::SetAutomaton, RewriteEngine, RewriteSpecification};
use temp_dir::TempDir;

pub struct SabreCompiling {
    library: Library,
    //rewrite_func: Symbol<unsafe extern fn() -> u32>,
}

impl RewriteEngine for SabreCompiling {
    fn rewrite(&mut self, term: DataExpression) -> DataExpression {
        term
    }
}

impl SabreCompiling {
    
    pub fn new(spec: &RewriteSpecification) -> Result<SabreCompiling, Box<dyn Error>> {
        let apma = SetAutomaton::new(spec, |_| (), true);

        // Create the directory structure for a Cargo project
        let temp_directory = TempDir::new().unwrap();
        let temp_dir = temp_directory.path();

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


        // Write the output source file.
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
        cmd("cargo", &["build", "--lib"])
            .dir(temp_dir)
            .run()?;
        println!("ok.");

        // Figure out the path to the library (it is based on platform)
        let mut path = PathBuf::from(generated_dir).join("./target/debug/libsabre_generated.so");
        if !path.exists() {
            path = PathBuf::from(generated_dir).join("./target/debug/sabre_generated.lib");
            if !path.exists() {
                return Err("Could not find the compiled library!".into());
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
        env_logger::init();

        let spec = RewriteSpecification::default();

        SabreCompiling::new(&spec).unwrap();
    }
}