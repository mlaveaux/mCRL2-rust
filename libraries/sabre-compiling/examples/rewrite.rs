use mcrl2::aterm::ATerm;
use mcrl2::aterm::TermPool;
use mcrl2::data::DataApplication;
use mcrl2::data::DataExpression;
use mcrl2::data::DataExpressionRef;

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

fn main() {}
