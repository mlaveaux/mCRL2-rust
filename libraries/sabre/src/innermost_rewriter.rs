use std::{cell::RefCell, fmt, rc::Rc, ops::Deref};

// use itertools::Itertools;

use itertools::Itertools;
use log::{trace, info};
use mcrl2::{
    aterm::{ATerm, TermPool, ATermTrait, Protected, ATermRef, Markable, Todo, Protector},
    data::{BoolSort, DataExpressionRef, DataFunctionSymbolRef, DataExpression, DataApplication},
};

use crate::{
    set_automaton::{
        check_equivalence_classes,
        EnhancedMatchAnnouncement, SetAutomaton,
    },
    utilities::get_position,
    RewriteEngine, RewriteSpecification, RewritingStatistics,
};

/// Innermost Adaptive Pattern Matching Automaton (APMA) rewrite engine.
pub struct InnermostRewriter {
    tp: Rc<RefCell<TermPool>>,
    apma: SetAutomaton,
}

impl RewriteEngine for InnermostRewriter {
    fn rewrite(&mut self, t: DataExpression) -> DataExpression {
        let mut stats = RewritingStatistics::default();

        let result = InnermostRewriter::rewrite_aux(&mut self.tp.borrow_mut(), &self.apma, t, &mut stats);
        info!("{} rewrites, {} single steps and {} symbol comparisons", stats.recursions, stats.rewrite_steps, stats.symbol_comparisons);
        result
    }
}

impl InnermostRewriter {
    pub fn new(tp: Rc<RefCell<TermPool>>, spec: &RewriteSpecification) -> InnermostRewriter {

        let apma =  SetAutomaton::new(spec, true);
        info!("ATerm pool: {}", tp.borrow());
        InnermostRewriter {
            tp: tp.clone(),
            apma,
        }
    }

    /// Function to rewrite a term 't'. The elements of the automaton 'states' and 'tp' are passed
    /// as separate parameters to satisfy the borrow checker.
    pub(crate) fn rewrite_aux(
        tp: &mut TermPool,
        automaton: &SetAutomaton,
        term: DataExpression,
        stats: &mut RewritingStatistics,
    ) -> DataExpression {
        debug_assert!(!term.is_default(), "Cannot rewrite the default term");

        stats.recursions += 1;

        let mut stack = InnerStack::default();        
        let mut write_terms =  stack.terms.write();
        let mut write_configs =  stack.configs.write();
        write_terms.push(DataExpressionRef::default());
        InnerStack::add_rewrite(&mut write_configs, &mut write_terms, term.copy(), 0);
        drop(write_terms);
        drop(write_configs);
        
        trace!("{}", stack);

        loop {
            let mut write_configs = stack.configs.write();
            if let Some(config) = write_configs.pop() {
                match config {
                    Config::Rewrite(result) => {
                        let mut write_terms = stack.terms.write();
                        let term = write_terms.pop().unwrap();

                        let term = DataExpressionRef::from(term.copy());
                        let symbol = term.data_function_symbol();
                        let arguments = term.data_arguments();

                        // For all the argument we reserve space on the stack.
                        let top_of_stack = write_terms.len();
                        for _ in 0..arguments.len() {
                            write_terms.push(Default::default());
                        }

                        //stack.add_result(symbol, arguments.len(), result);
                        let symbol = write_configs.protect(&symbol.into());
                        write_configs.push(Config::Construct(symbol.into(), arguments.len(), result));
                        for (offset, arg) in arguments.into_iter().enumerate() {
                            InnerStack::add_rewrite(&mut write_configs, &mut write_terms, arg.into(), top_of_stack + offset);
                        }
                    }
                    Config::Construct(symbol, arity, index) => {
                        // Take the last arity arguments.
                        let mut terms = stack.terms.write();
                        let length = terms.len();

                        let arguments = &terms[length - arity..];

                        let term: DataExpression = if arguments.is_empty() {
                            symbol.protect().into()
                        } else {
                            DataApplication::from_refs(tp, &symbol.copy().into(), arguments).into()
                        };

                        // Remove the arguments from the stack.
                        terms.drain(length - arity..);

                        match InnermostRewriter::find_match(tp, automaton, &term, stats) {
                            Some(ema) => {
                                // TODO: This ignores the first element of the stack, but that is kind of difficult to deal with.
                                let top_of_stack = terms.len();
                                terms.reserve(ema.stack_size - 1); // We already reserved space for the result.
                                for _ in 0..ema.stack_size - 1 {
                                    terms.push(Default::default());
                                }

                                let mut first = true;
                                for config in (&ema.innermost_stack.read()).iter() {
                                    match config {
                                        Config::Construct(symbol, arity, offset) => {
                                            if first {
                                                // The first result must be placed on the original result index.
                                                InnerStack::add_result(&mut write_configs, symbol.copy(), *arity, index);
                                            } else {
                                                // Otherwise, we put it on the end of the stack.
                                                InnerStack::add_result(&mut write_configs,
                                                    symbol.copy(),
                                                    *arity,
                                                    top_of_stack + offset - 1,
                                                );                  
                                            }
                                        }
                                        Config::Rewrite(_) => {
                                            panic!("This case should not happen");
                                        }
                                    }
                                    first = false;
                                }
                                trace!(
                                    "{}, {}, {}",
                                    ema.stack_size,
                                    ema.variables.iter().format_with(", ", |element, f| {
                                        f(&format_args!("{} -> {}", element.0, element.1))
                                    }),
                                    ema.innermost_stack.read().iter().format("\n")
                                );

                                debug_assert!(ema.stack_size != 1 || ema.variables.len() <= 1, "There can only be a single variable in the right hand side");
                                if ema.stack_size == 1 && ema.variables.len() == 1{
                                    // This is a special case where we place the result on the correct position immediately.
                                    // The right hand side is only a variable
                                    let t = terms.protect(&get_position(term.deref(), &ema.variables[0].0));
                                    terms[index] = t.into();
                                } else {
                                    for (position, index) in &ema.variables {
                                        // Add the positions to the stack.
                                        let t = terms.protect(&get_position(term.deref(), position));
                                        terms[top_of_stack + index - 1] = t.into();
                                    }
                                }

                                stats.rewrite_steps += 1;
                                trace!("applying rule {}", ema.announcement.rule);
                            }
                            None => {
                                // Add the term on the stack.
                                let t = terms.protect(&term);
                                terms[index] = t.into();
                            }
                        }
                    }
                }

                //trace!("{}", stack);

                for (index, term) in stack.terms.write().iter().enumerate() {
                    if term.is_default() {
                        debug_assert!(
                            write_configs.iter().any(|x| {
                                match x {
                                    Config::Construct(_, _, result) => index == *result,
                                    Config::Rewrite(result) => index == *result,
                                }
                            }),
                            "This default term {index} is not a result of any operation."
                        );
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

        let mut write_terms = stack
            .terms
            .write();

        write_terms
            .pop()
            .expect("The result should be the last element on the stack")
            .protect()
    }

    /// Use the APMA to find a match for the given term.
    fn find_match<'a>(
        tp: &mut TermPool,
        automaton: &'a SetAutomaton,
        t: &DataExpression,
        stats: &mut RewritingStatistics,
    ) -> Option<&'a EnhancedMatchAnnouncement> {
        // Start at the initial state
        let mut state_index = 0;
        loop {
            let state = &automaton.states[state_index];

            // Get the symbol at the position state.label
            stats.symbol_comparisons += 1;
            let pos: DataExpressionRef = get_position(t.deref(), &state.label).into();
            let symbol = pos.data_function_symbol();

            // Get the transition for the label and check if there is a pattern match
            if let Some(transition) = automaton.transitions.get(&(state_index, symbol.operation_id())) {
                for ema in &transition.announcements {
                    if check_equivalence_classes(t, &ema.equivalence_classes)
                        && InnermostRewriter::check_conditions(tp, automaton, t, ema, stats)
                    {
                        // We found a matching pattern
                        return Some(ema);
                    }
                }

                // If there is no pattern match we check if the transition has a destination state
                if transition.destinations.is_empty() {
                    // If there is no destination state there is no pattern match
                    return None;
                } else {
                    // Continue matching in the next state
                    state_index = transition.destinations.first().unwrap().1;
                }
            } else {
                return None;
            }
        }
    }

    /// Given a term with head symbol 't_head' and subterms 't_subterms' and an EnhancedMatchAnnouncement,
    /// check if the conditions hold.
    fn check_conditions(
        tp: &mut TermPool,
        automaton: &SetAutomaton,
        t: &ATerm,
        ema: &EnhancedMatchAnnouncement,
        stats: &mut RewritingStatistics,
    ) -> bool {
        for c in &ema.conditions {
            let rhs: DataExpression = c.semi_compressed_rhs.evaluate(t, tp).into();
            let lhs: DataExpression = c.semi_compressed_lhs.evaluate(t, tp).into();

            let rhs_normal = InnermostRewriter::rewrite_aux(tp, automaton, rhs, stats);
            let lhs_normal = if lhs == BoolSort::true_term().into() {
                // TODO: Store the conditions in a better way. REC now uses a list of equalities while mCRL2 specifications have a simple condition.
                lhs
            } else {
                InnermostRewriter::rewrite_aux(tp, automaton, lhs, stats)
            };

            if lhs_normal != rhs_normal && c.equality || lhs_normal == rhs_normal && !c.equality {
                return false;
            }
        }

        true
    }
}

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Config {
    ///
    // Result(usize),
    /// Rewrite the top of the stack and put result at the given index.
    Rewrite(usize), 
    /// Constructs function symbol with given arity at the given index.
    Construct(DataFunctionSymbolRef<'static>, usize, usize), 
}

impl Markable for Config {
    fn mark(&self, todo: Todo<'_>) {
        match self {
            Config::Construct(t, _, _) => {
                let t: ATermRef<'_> = t.copy().into();
                t.mark(todo);
            },
            _ => {

            }
        }
    }

    fn contains_term(&self, term: &ATermRef<'_>) -> bool {
        match self {
            Config::Construct(t, _, _) => {
                term == &<DataFunctionSymbolRef as Into<ATermRef>>::into(t.copy())
            },
            _ => {
                false
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            Config::Construct(_, _, _) => {
                1
            },
            _ => {
                0
            }
        }
    }
}

/// This stack is used to avoid recursion and also to keep track of terms in
/// normal forms by explicitly representing the rewrites of a right hand
/// side.
#[derive(Default)]
struct InnerStack {
    configs: Protected<Vec<Config>>,
    terms: Protected<Vec<DataExpressionRef<'static>>>,
}
 
impl fmt::Display for InnerStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Terms: [")?;
        for (i, term) in self.terms.read().iter().enumerate() {
            writeln!(f, "{}\t{:?}", i, term)?;
        }
        writeln!(f, "]")?;

        writeln!(f, "Configs: [")?;
        for config in self.configs.read().iter() {
            writeln!(f, "\t{}", config)?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Config::Rewrite(result) => write!(f, "Rewrite({})", result),
            Config::Construct(symbol, arity, result) => {
                write!(f, "Construct({}, {}, {})", symbol, arity, result)
            }
        }
    }
}

impl InnerStack {

    /// Indicate that the given 
    fn add_result<'a>(write_configs: &mut Protector<Vec<Config>>, symbol: DataFunctionSymbolRef<'a>, arity: usize, index: usize) {
        let symbol = write_configs.protect(&symbol.into());
        write_configs.push(Config::Construct(symbol.into(), arity, index));
    }

    fn add_rewrite<'a>(write_configs: &mut Protector<Vec<Config>>, write_terms: &mut Protector<Vec<DataExpressionRef<'static>>>, term: DataExpressionRef<'a>, index: usize) {
        let term = write_terms.protect(&term);
        write_configs.push(Config::Rewrite(index));
        write_terms.push(term.into());
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use ahash::AHashSet;
    use mcrl2::aterm::{TermPool, random_term};

    use test_log::test;

    use crate::{
        utilities::to_untyped_data_expression, InnermostRewriter, RewriteEngine, RewriteSpecification,
    };

    #[test]
    fn test_innermost_simple() {
        let tp = Rc::new(RefCell::new(TermPool::new()));

        let spec = RewriteSpecification {
            rewrite_rules: vec![],
            constructors: vec![],
        };
        let mut inner = InnermostRewriter::new(tp.clone(), &spec);

        let term = random_term(
            &mut tp.borrow_mut(),
            &[("f".to_string(), 2)],
            &["a".to_string(), "b".to_string()],
            5,
        );
        let term = to_untyped_data_expression(&mut tp.borrow_mut(), &term, &AHashSet::new());

        assert_eq!(
            inner.rewrite(term.clone().into()),
            term.into(),
            "Should be in normal form for no rewrite rules"
        );
    }
}
