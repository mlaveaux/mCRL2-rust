use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use indoc::indoc;

pub fn generate(source_dir: &Path) -> Result<(), Box<dyn Error>> {
    {
        let mut file = File::create(PathBuf::from(source_dir).join("lib.rs"))?;

        writeln!(
            &mut file,
            indoc! {"
            use mcrl2::{{aterm::{{ATerm, TermPool}}, data::{{DataApplication, DataExpression, DataExpressionRef}}}};

            /// Generic rewrite function
            #[no_mangle]
            pub unsafe extern \"C\" fn rewrite_term(term: DataExpression) -> DataExpression {{
                let mut arguments: Vec<ATerm> = vec![];
                let mut tp = TermPool::new();
            
                for arg in term.data_arguments() {{
                    let t: DataExpressionRef<'_> = arg.into();
            
                    arguments.push(rewrite_term(t.protect()).into());
                }}
            
                DataApplication::new(&mut tp, &term.data_function_symbol(), &arguments).into()
            }}
            "}
        )?;
    }

    Ok(())
}
