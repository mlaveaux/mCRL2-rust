use ahash::AHashSet;
use mcrl2_sys::atermpp::TermPool;

use crate::{utilities::to_data_expression, Rule};

/// Create a rewrite rule lhs -> rhs with the given names being variables.
pub(crate) fn create_rewrite_rule(
    tp: &mut TermPool,
    lhs: &str,
    rhs: &str,
    variables: &[&str],
) -> Rule {
    let lhs = tp.from_string(lhs).unwrap();
    let rhs = tp.from_string(rhs).unwrap();
    let mut vars = AHashSet::new();
    for var in variables {
        vars.insert(var.to_string());
    }

    Rule {
        conditions: vec![],
        lhs: to_data_expression(tp, &lhs, &vars),
        rhs: to_data_expression(tp, &rhs, &vars),
    }
}