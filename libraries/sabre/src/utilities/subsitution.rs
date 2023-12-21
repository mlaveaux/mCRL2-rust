use ahash::AHashSet;
use mcrl2::{aterm::{ATerm, ATermTrait}, aterm_builder::{TermBuilder, Yield}, symbol::SymbolTrait};
use mcrl2::aterm_pool::TermPool;

/// Creates a new term where a subterm is replaced with another term.
///
/// # Parameters
/// 't'             -   The original term
/// 'new_subterm'   -   The subterm that will be injected
/// 'p'             -   The place in 't' on which 'new_subterm' will be placed,
///                     given as a slice of position indexes
///
/// # Example
/// The term is constructed bottom up. As an an example take the term s(s(a)).
/// Lets say we want to replace the a with the term 0.
/// Then we traverse the term until we have arrived at a and replace it with 0.
/// We then construct s(0) (using the maximally shared term pool)
/// We then construct s(s(0)) (using the maximally shared term pool)
pub fn substitute<'a>(tp: &mut TermPool, t: &'a impl ATermTrait<'a>, new_subterm: ATerm, p: &[usize]) -> ATerm {
    substitute_rec(tp, t, new_subterm, p, 0)
}

/// The recursive implementation for subsitute
///
/// 'depth'         -   Used to keep track of the depth in 't'. Function should be called with
///                     'depth' = 0.
fn substitute_rec<'a>(
    tp: &mut TermPool,
    t: &'a impl ATermTrait<'a>,
    new_subterm: ATerm,
    p: &[usize],
    depth: usize,
) -> ATerm {
    if p.len() == depth {
        // in this case we have arrived at the place where 'new_subterm' needs to be injected
        new_subterm
    } else {
        // else recurse deeper into 't'
        let new_child_index = p[depth] - 1;
        let new_child = substitute_rec(tp, &t.arg(new_child_index), new_subterm, p, depth + 1);

        let mut args: Vec<ATerm> = t.arguments().map(|t| t.protect()).collect();
        args[new_child_index] = new_child;

        tp.create(&t.get_head_symbol(), &args)
    }
}

/// Converts an [ATerm] to an untyped data expression.
pub fn to_data_expression(tp: &mut TermPool, t: &ATerm, variables: &AHashSet<String>) -> ATerm {
    let mut builder = TermBuilder::<ATerm, ATerm>::new();

    builder.evaluate(tp, t.clone(), |tp, args, t| {
        debug_assert!(!t.is_int(), "Term cannot be an aterm_int, although not sure why");

        if variables.contains(t.get_head_symbol().name()) {
            // Convert a constant variable, for example 'x', into an untyped variable.
            Ok(Yield::Term(tp.create_variable(t.get_head_symbol().name()).into()))
        } else if t.get_head_symbol().arity() == 0 {
            Ok(Yield::Term(tp.create_data_function_symbol(t.get_head_symbol().name()).into()))
        } else {
            // This is a function symbol applied to a number of arguments (higher order terms not allowed)
            let head = tp.create_data_function_symbol(t.get_head_symbol().name());
            
            for arg in t.arguments() {
                args.push(arg.protect());
            }

            Ok(Yield::Construct(head.into()))
        }
    }, |tp, input, args| {
            Ok(tp.create_data_application(&input, args).into())
        }
    ).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::utilities::{get_position, ExplicitPosition};

    use super::*;

    #[test]
    fn test_substitute() {
        let mut term_pool = TermPool::new();

        let t = term_pool.from_string("s(s(a))").unwrap();
        let t0 = term_pool.from_string("0").unwrap();

        // substitute the a for 0 in the term s(s(a))
        let result = substitute(&mut term_pool, &t, t0.clone(), &vec![1, 1]);

        // Check that indeed the new term as a 0 at position 1.1.
        assert_eq!(
            t0,
            get_position(&result, &ExplicitPosition::new(&vec![1, 1])).protect()
        );
    }

    #[test]
    fn test_to_data_expression() {
        let mut term_pool = TermPool::new();

        let t = term_pool.from_string("s(s(a))").unwrap();

        let _expression = to_data_expression(&mut term_pool, &t, &AHashSet::from_iter(["a".to_string()]));
    }
}
