use std::{fs::File, error::Error, io::Write};

use indoc::indoc;

pub fn compile() -> Result<(), Box<dyn Error>> {

    // Write the output source file.
    let mut file = File::create("sabre-compiled/src/lib.rs")?;
    writeln!(&mut file, indoc! {"
        fn main() {{
            println!(\"Hello world!\");
        }}
    "})?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation() {

        compile().unwrap();

    }
}