use std::{error::Error, fs::File, io::Write, path::Path};

use abi_stable::library::RootModule;
use duct::cmd;
use indoc::indoc;
use log::info;
use mcrl2::data::DataExpression;
use sabre::{RewriteEngine, RewriteSpecification};

use crate::SabreCompiledRef;

pub struct SabreCompiling {

    library: SabreCompiledRef,
}

impl RewriteEngine for SabreCompiling {
    fn rewrite(&mut self, term: DataExpression) -> DataExpression {
        self.library.rewrite();
        term
    }
}

impl SabreCompiling {
    
    pub fn new(spec: &RewriteSpecification) -> Result<SabreCompiling, Box<dyn Error>> {

        // Write the output source file.
        let mut file = File::create("sabre-compiled/src/generated.rs")?;
        writeln!(&mut file, indoc! {"
            pub fn rewrite_term() {{
                println!(\"Hello world!\");
            }}
        "})?;

        // Compile the dynamic object.
        info!("Compiling...");
        cmd("cargo", &["build", "--lib"])
            .dir("sabre-compiled/")
            .run()?;
        println!("ok.");

        // Load it back in and call the rewriter.
        Ok(SabreCompiling {
            library: SabreCompiledRef::load_from_directory(&Path::new("../../target/debug"))?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

//     #[test]
//     fn test_compilation() {
//         let spec = RewriteSpecification::default();

//         SabreCompiling::new(&spec).unwrap();
//     }
}