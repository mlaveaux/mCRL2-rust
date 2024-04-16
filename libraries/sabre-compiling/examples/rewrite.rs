use mcrl2::{aterm::{ATerm, TermPool}, data::{DataApplication, DataExpression, DataExpressionRef}};

/// Generic rewrite function
#[no_mangle]
pub unsafe extern "C" fn rewrite_term(term: DataExpression) -> DataExpression {
    let mut arguments: Vec<ATerm> = vec![];
    let mut tp = TermPool::new();

    for arg in term.data_arguments() {
        let t: DataExpressionRef<'_> = arg.into();

        arguments.push(rewrite_term(t.protect()).into());
    }

    DataApplication::new(&mut tp, &term.data_function_symbol(), &arguments).into()
}

fn main() {

}