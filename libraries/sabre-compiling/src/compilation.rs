use std::{fs::File, error::Error, io::Write};




pub fn compile() -> Result<(), Box<dyn Error>> {


    let mut file = File::create("output.rs")?;

    writeln!(&mut file, "
        fn main() {{
            println!(\"Hello world!\");
        }}
    ")?;

    

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