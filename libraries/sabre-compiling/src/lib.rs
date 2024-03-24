use std::{env::temp_dir, error::Error, ffi::c_void, fs::{self, File}, io::Write, path::{Path, PathBuf}};

use duct::cmd;
use indoc::indoc;
use libloading::{Library, Symbol};
use log::info;
use mcrl2::data::DataExpression;
use sabre::{RewriteEngine, RewriteSpecification};

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

        // Create the directory structure for a Cargo project
        let generated_dir = Path::new("temp");
        let source_dir = PathBuf::from(generated_dir).join("src");

        if !generated_dir.exists() {
            fs::create_dir(&generated_dir)?;
        }

        if !source_dir.exists() {
            fs::create_dir(&source_dir)?;
        }

        // Write the cargo configuration
        {
            let mut file = File::create(PathBuf::from(generated_dir).join("Cargo.toml"))?;
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
            .dir(generated_dir)
            .run()?;
        println!("ok.");

        // Load it back in and call the rewriter.
        unsafe {
            let library = Library::new(PathBuf::from(generated_dir).join("./target/debug/sabre_generated.dll"))?;
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

    #[test]
    fn test_compilation() {
        let spec = RewriteSpecification::default();

        SabreCompiling::new(&spec).unwrap();
    }
}