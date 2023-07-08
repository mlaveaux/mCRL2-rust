use ahash::AHashSet;
use mcrl2_rust::atermpp::{ATerm, TermPool};

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

/// Create a random term consisting of the given symbol and constants. Performs
/// iterations number of constructions, and uses chance_duplicates to choose the
/// amount of subterms that are duplicated.
pub(crate) fn random_term(
    tp: &mut TermPool,
    symbols: &[(String, usize)],
    constants: &[String],
    iterations: usize,
) -> ATerm {
    use rand::prelude::IteratorRandom;

    assert!(
        !constants.is_empty(),
        "We need constants to be able to create a term"
    );

    let mut subterms = AHashSet::<ATerm>::from_iter(
        constants
            .iter()
            .map(|name| {
                let symbol = tp.create_symbol(name, 0);
                tp.create(&symbol, &[])
            }),
    );

    let mut rng = rand::thread_rng();
    let mut result = ATerm::default();
    for _ in 0..iterations {
        let (symbol, arity) = symbols.iter().choose(&mut rng).unwrap();

        let mut arguments = vec![];
        for _ in 0..*arity {
            arguments.push(subterms.iter().choose(&mut rng).unwrap().clone());
        }

        let symbol = tp.create_symbol(symbol, *arity);
        result = tp.create(&symbol, &arguments);

        // Make this term available as another subterm that can be used.
        subterms.insert(result.clone());
    }

    result
}
