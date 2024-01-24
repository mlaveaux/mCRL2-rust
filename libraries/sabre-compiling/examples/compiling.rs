use mcrl2::data::DataExpression;
use sabre::{RewriteEngine, RewriteSpecification};
use sabre_compiling::SabreCompiling;


fn main() {
    let spec = RewriteSpecification::default();

    let mut sabrec = SabreCompiling::new(&spec).unwrap();

    sabrec.rewrite(DataExpression::default());
}