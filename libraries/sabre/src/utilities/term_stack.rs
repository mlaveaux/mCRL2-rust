use std::fmt;

use ahash::{HashMap, HashMapExt};
use log::trace;
use mcrl2::{aterm::{ATermRef, Markable, Protected, TermPool, Todo}, data::{is_data_expression, is_data_machine_number, is_data_variable, DataApplication, DataExpression, DataExpressionRef, DataFunctionSymbolRef, DataVariable}};

use crate::{utilities::InnermostStack, Rule};

use super::{ExplicitPosition, PositionIterator};


/// A stack used to represent a term with free variables that can be constructed
/// efficiently.
///
/// It stores as much as possible in the term pool. Due to variables it cannot
/// be fully compressed. For variables it stores the position in the lhs of a
/// rewrite rule where the concrete term can be found that will replace the
/// variable.
///
#[derive(Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TermStack {
    /// The innermost rewrite stack for the right hand side and the positions that must be added to the stack.
    pub(crate) innermost_stack: Protected<Vec<Config>>,
    pub(crate) variables: Vec<(ExplicitPosition, usize)>,
    pub(crate) stack_size: usize,
}

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Config {
    /// Rewrite the top of the stack and put result at the given index.
    Rewrite(usize),
    /// Constructs function symbol with given arity at the given index.
    Construct(DataFunctionSymbolRef<'static>, usize, usize),
    /// A concrete term to be placed at the current position in the stack.
    Term(DataExpressionRef<'static>, usize),
    /// Yields the given index as returned term.
    Return(),
}

impl Markable for Config {
    fn mark(&self, todo: Todo<'_>) {
        if let Config::Construct(t, _, _) = self {
            let t: ATermRef<'_> = t.copy().into();
            t.mark(todo);
        }
    }

    fn contains_term(&self, term: &ATermRef<'_>) -> bool {
        if let Config::Construct(t, _, _) = self {
            term == &<DataFunctionSymbolRef as Into<ATermRef>>::into(t.copy())
        } else {
            false
        }
    }

    fn len(&self) -> usize {
        if let Config::Construct(_, _, _) = self {
            1
        } else {
            0
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Config::Rewrite(result) => write!(f, "Rewrite({})", result),
            Config::Construct(symbol, arity, result) => {
                write!(f, "Construct({}, {}, {})", symbol, arity, result)
            }
            Config::Term(term, result) => {
                write!(f, "Term({}, {})", term, result)
            }
            Config::Return() => write!(f, "Return()"),
        }
    }
}

impl TermStack {
    /// Construct a new right-hand stack for a given equation/rewrite rule.
    pub fn new(rule: &Rule) -> TermStack {
        Self::from_term(&rule.rhs.copy(), &create_var_map(&rule.lhs.copy().into()))
    }

    /// 
    pub fn from_term(term: &DataExpressionRef, var_map: &HashMap<DataVariable, ExplicitPosition>) -> TermStack {
        // Compute the extra information for the InnermostRewriter.
        let mut innermost_stack: Protected<Vec<Config>> = Protected::new(vec![]);
        let mut variables = vec![];
        let mut stack_size = 0;

        for (term, position) in PositionIterator::new(term.copy().into()) {
            if let Some(index) = position.indices.last() {
                if *index == 1 {
                    continue; // Skip the function symbol.
                }
            }

            if is_data_variable(&term) {
                variables.push((
                    var_map
                        .get(&term.protect())
                        .expect(
                            "All variables in the right hand side must occur in the left hand side",
                        )
                        .clone(),
                    stack_size,
                ));
                stack_size += 1;
            } else if is_data_machine_number(&term) {
                // Skip SortId(@NoValue) and OpId
            } else if is_data_expression(&term) {
                let t: DataExpressionRef = term.into();
                let arity = t.data_arguments().len();
                let mut write = innermost_stack.write();
                let symbol = write.protect(&t.data_function_symbol().into());
                write.push(Config::Construct(symbol.into(), arity, stack_size));
                stack_size += 1;
            } else {
                // Skip intermediate terms such as UntypeSortUnknown.
            }
        }

        TermStack {
            innermost_stack,
            stack_size,
            variables,
        }
    }

    pub fn evaluate(&self, tp: &mut TermPool, term: &ATermRef) -> DataExpression {
        let mut builder = TermStackBuilder::new();
        self.evaluate_with(tp, term, &mut builder)
    }

    /// Evaluate the rhs stack for the given term and returns the result.
    pub fn evaluate_with(&self, tp: &mut TermPool, term: &ATermRef, builder: &mut TermStackBuilder) -> DataExpression {
        let stack = &mut builder.stack;
        {
            let mut write = stack.terms.write();
            write.clear();   
            write.push(DataExpressionRef::default());
        }

        InnermostStack::integrate(
            &mut stack.configs.write(),
            &mut stack.terms.write(),
            self,
            &DataExpressionRef::from(term.copy()),
            0,
        );
        loop {
            trace!("{}", stack);

            let mut write_configs = stack.configs.write();
            if let Some(config) = write_configs.pop() {
                match config {
                    Config::Construct(symbol, arity, index) => {
                        // Take the last arity arguments.
                        let mut write_terms = stack.terms.write();
                        let length = write_terms.len();

                        let arguments = &write_terms[length - arity..];

                        let term: DataExpression = if arguments.is_empty() {
                            symbol.protect().into()
                        } else {
                            DataApplication::new(tp, &symbol.copy(), arguments).into()
                        };

                        // Add the term on the stack.
                        write_terms.drain(length - arity..);
                        let t = write_terms.protect(&term);
                        write_terms[index] = t.into();
                    },
                    Config::Term(term, index) => {
                        let mut write_terms = stack.terms.write();
                        let t = write_terms.protect(&term);
                        write_terms[index] = t.into();
                    }
                    Config::Rewrite(_) => {
                        unreachable!("This case should not happen");
                    }
                    Config::Return() => {
                        unreachable!("This case should not happen");
                    }
                }
            } else {
                break;
            }
        }

        debug_assert!(
            stack.terms.read().len() == 1,
            "Expect exactly one term on the result stack"
        );

        let mut write_terms = stack.terms.write();

        write_terms
            .pop()
            .expect("The result should be the last element on the stack")
            .protect()
    }
    
    /// Used to check if a subterm is duplicated, for example "times(s(x), y) =
    /// plus(y, times(x,y))" is duplicating.
    pub(crate) fn contains_duplicate_var_references(&self) -> bool {
        let mut variables = self.variables.clone();
        variables.sort_by_key(|(pos, _)| pos.clone());
        let len = variables.len();
        variables.dedup();

        len == variables.len()
    }
}

impl Clone for TermStack {
    fn clone(&self) -> Self {
        // TODO: It would make sense if Protected could implement Clone.
        let mut innermost_stack: Protected<Vec<Config>> = Protected::new(vec![]);

        let mut write = innermost_stack.write();
        for t in self.innermost_stack.read().iter() {
            match t {
                Config::Rewrite(x) => write.push(Config::Rewrite(*x)),
                Config::Construct(f, x, y) => {
                    let f = write.protect(&f.copy().into());
                    write.push(Config::Construct(f.into(), *x, *y));
                }
                Config::Term(t, y) => {
                    let f = write.protect(&t.copy().into());
                    write.push(Config::Term(f.into(), *y));
                }
                Config::Return() => write.push(Config::Return()),
            }
        }
        drop(write);

        Self {
            variables: self.variables.clone(),
            stack_size: self.stack_size,
            innermost_stack,
        }
    }
}


pub struct TermStackBuilder {
    stack: InnermostStack,

}

impl TermStackBuilder {
    pub fn new() -> Self {
        Self {
            stack: InnermostStack::default(),
        }
    }
}

/// Create a mapping of variables to their position in the given term
pub fn create_var_map(t: &ATermRef) -> HashMap<DataVariable, ExplicitPosition> {
    let mut result = HashMap::new();

    for (term, position) in PositionIterator::new(t.copy()) {
        if is_data_variable(&term) {
            result.insert(term.protect().into(), position.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHashSet;
    use mcrl2::aterm::{apply, ATerm, TermPool};
    use mcrl2::data::DataFunctionSymbol;

    use crate::test_utility::create_rewrite_rule;
    use crate::utilities::to_untyped_data_expression;

    use test_log::test;

    /// Convert terms in variables to a [DataVariable].
    pub fn convert_variables(tp: &mut TermPool, t: &ATerm, variables: &AHashSet<String>) -> ATerm {
        apply(tp, t, &|tp, arg| {
            if variables.contains(arg.get_head_symbol().name()) {
                // Convert a constant variable, for example 'x', into an untyped variable.
                Some(DataVariable::new(tp, &arg.get_head_symbol().name()).into())
            } else {
                None
            }
        })
    }

    #[test]
    fn test_rhs_stack() {
        let mut tp = TermPool::new();

        let rhs_stack = TermStack::new(
            &create_rewrite_rule(&mut tp, "fact(s(N))", "times(s(N), fact(N))", &["N"]).unwrap(),
        );
        let mut expected = Protected::new(vec![]);

        let mut write = expected.write();
        let t = write.protect(&DataFunctionSymbol::new(&mut tp, "times").copy().into());
        write.push(Config::Construct(t.into(), 2, 0));

        let t = write.protect(&DataFunctionSymbol::new(&mut tp, "s").copy().into());
        write.push(Config::Construct(t.into(), 1, 1));

        let t = write.protect(&DataFunctionSymbol::new(&mut tp, "fact").copy().into());
        write.push(Config::Construct(t.into(), 1, 2));
        drop(write);

        // Check if the resulting construction succeeded.
        assert_eq!(
            rhs_stack.innermost_stack, expected,
            "The resulting config stack is not as expected"
        );

        assert_eq!(rhs_stack.stack_size, 5, "The stack size does not match");

        // Test the evaluation
        let lhs = tp.from_string("fact(s(a))").unwrap();
        let lhs_expression = to_untyped_data_expression(&mut tp, &lhs, &AHashSet::new());

        let rhs = tp.from_string("times(s(a), fact(a))").unwrap();
        let rhs_expression = to_untyped_data_expression(&mut tp, &rhs, &AHashSet::new());

        assert_eq!(
            rhs_stack.evaluate(&mut tp, &lhs_expression),
            rhs_expression,
            "The rhs stack does not evaluate to the expected term"
        );
    }

    #[test]
    fn test_rhs_stack_variable() {
        let mut tp = TermPool::new();

        let rhs = TermStack::new(&create_rewrite_rule(&mut tp, "f(x)", "x", &["x"]).unwrap());

        // Check if the resulting construction succeeded.
        assert!(
            rhs.innermost_stack.read().is_empty(),
            "The resulting config stack is not as expected"
        );

        assert_eq!(rhs.stack_size, 1, "The stack size does not match");
    }

    #[test]
    fn test_evaluation() {
        let mut tp = TermPool::new();
        let t_rhs = {
            let tmp = tp.from_string("f(f(a,a),x)").unwrap();
            to_untyped_data_expression(&mut tp, &tmp, &AHashSet::from([String::from("x")]))
        };

        let t = tp.from_string("g(b)").unwrap();
        let t_lhs = to_untyped_data_expression(&mut tp, &t, &AHashSet::new());

        // Make a variable map with only x@1.
        let mut map = HashMap::new();
        map.insert(DataVariable::new(&mut tp, "x"), ExplicitPosition::new(&[1]));

        let sctt = TermStack::from_term(&t_rhs.copy(), &map);

        let t = tp.from_string("f(f(a,a),b)").unwrap();
        let t_expected = to_untyped_data_expression(&mut tp, &t, &AHashSet::new());

        assert_eq!(sctt.evaluate(&mut tp, &t_lhs), t_expected);
    }

    #[test]
    fn test_create_varmap() {
        let mut tp = TermPool::new();
        let t = {
            let tmp = tp.from_string("f(x,x)").unwrap();
            convert_variables(&mut tp, &tmp, &AHashSet::from([String::from("x")]))
        };
        let x = DataVariable::new(&mut tp, "x");

        let map = create_var_map(&t);
        assert!(map.contains_key(&x));
    }

    #[test]
    fn test_is_duplicating() {
        let mut tp = TermPool::new();
        let t_rhs = {
            let tmp = tp.from_string("f(x,x)").unwrap();
            to_untyped_data_expression(&mut tp, &tmp, &AHashSet::from([String::from("x")]))
        };

        // Make a variable map with only x@1.
        let mut map = HashMap::new();
        map.insert(DataVariable::new(&mut tp, "x"), ExplicitPosition::new(&[1]));

        let sctt = TermStack::from_term(&t_rhs.copy(), &map);
        assert!(
            sctt.contains_duplicate_var_references(),
            "This sctt is duplicating"
        );
    }
}