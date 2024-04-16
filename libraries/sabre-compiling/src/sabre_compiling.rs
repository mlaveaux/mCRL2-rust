use std::{
    cell::RefCell,
    error::Error,
    path::{Path, PathBuf},
    rc::Rc,
};

use libloading::{Library, Symbol};
use log::info;
use temp_dir::TempDir;
use toml::Table;

use mcrl2::{aterm::TermPool, data::DataExpression};
use sabre::{set_automaton::SetAutomaton, RewriteEngine, RewriteSpecification};

use crate::{generate, library::RuntimeLibrary};

pub struct SabreCompilingRewriter {
    library: Library,
    //rewrite_func: Symbol<unsafe extern fn() -> u32>,
}

impl RewriteEngine for SabreCompilingRewriter {
    fn rewrite(&mut self, term: DataExpression) -> DataExpression {
        // TODO: This ought to be stored somewhere for repeated calls.
        unsafe {
            let func: Symbol<extern "C" fn()> = self.library.get(b"rewrite_term").unwrap();

            func();

            term
        }
    }
}

impl SabreCompilingRewriter {
    /// Creates a new compiling rewriter for the given specifications.
    ///
    /// - use_local_workspace: Use the developement version of the toolset instead of referring to the github one.
    /// - use_local_tmp: Use a relative 'tmp' directory instead of using the system directory. Mostly used for debugging purposes.
    pub fn new(
        _tp: Rc<RefCell<TermPool>>,
        spec: &RewriteSpecification,
        use_local_workspace: bool,
        use_local_tmp: bool,
    ) -> Result<SabreCompilingRewriter, Box<dyn Error>> {
        let system_tmp_dir = TempDir::new()?;
        let temp_dir = if use_local_tmp {
            &Path::new("./tmp")
        } else {
            system_tmp_dir.path()
        };

        let mut dependencies = vec![];

        if use_local_workspace {
            let compilation_toml =
                include_str!("../../../target/Compilation.toml").parse::<Table>()?;
            let path = compilation_toml
                .get("sabrec")
                .ok_or("Missing [sabre] section)")?
                .get("path")
                .ok_or("Missing path entry")?
                .as_str()
                .ok_or("Not a string")?;

            info!("Using local dependency {}", path);
            dependencies.push(format!(
                "mcrl2 = {{ path = '{}' }}",
                PathBuf::from(path).join("../mcrl2").to_string_lossy()
            ));
        } else {
            info!("Using git dependency https://github.com/mlaveaux/mCRL2-rust.git");
            dependencies.push(
                "sabre-ffi = { git = 'https://github.com/mlaveaux/mCRL2-rust.git' }".to_string(),
            );
        }

        let mut compilation_crate = RuntimeLibrary::new(temp_dir, dependencies)?;

        // Generate the automata used for matching
        let _apma = SetAutomaton::new(spec, |_| (), true);

        // Write the output source file(s).
        generate(compilation_crate.source_dir())?;

        let library = compilation_crate.compile()?;
        Ok(SabreCompilingRewriter { library })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_log::test;

    #[test]
    fn test_compilation() {
        let spec = RewriteSpecification::default();
        let tp = Rc::new(RefCell::new(TermPool::new()));

        SabreCompilingRewriter::new(tp, &spec, true, true).unwrap();
    }
}
