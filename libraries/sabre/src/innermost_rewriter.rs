use std::{cell::RefCell, fmt, rc::Rc};

// use itertools::Itertools;

use mcrl2::{
    atermpp::{ATerm, TermPool},
    data::DataFunctionSymbol,
};

use crate::{
    set_automaton::{
        check_equivalence_classes, get_data_arguments, get_data_function_symbol,
        EnhancedMatchAnnouncement, SetAutomaton,
    },
    utilities::get_position,
    RewriteEngine, RewriteSpecification, RewritingStatistics,
};

#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Config {
    Rewrite(usize), // Rewrite the top of the stack and put result at the given index.
    Construct(DataFunctionSymbol, usize, usize), // Constructs f with arity at the given index.
}

#[derive(Default)]
struct InnerStack {
    configs: Vec<Config>,
    terms: Vec<ATerm>,
}

impl fmt::Display for InnerStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Terms: [")?;
        for (i, term) in self.terms.iter().enumerate() {
            writeln!(f, "{}\t{}", i, term)?;
        }
        writeln!(f, "]")?;

        writeln!(f, "Configs: [")?;
        for config in &self.configs {
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
    fn add_result(&mut self, symbol: DataFunctionSymbol, arity: usize, index: usize) {
        self.configs.push(Config::Construct(symbol, arity, index));
    }

    fn add_rewrite(&mut self, term: ATerm, index: usize) {
        self.configs.push(Config::Rewrite(index));
        self.terms.push(term);
    }
}

/// Innermost Adaptive Pattern Matching Automaton (APMA) rewrite engine.
pub struct InnermostRewriter {
    term_pool: Rc<RefCell<TermPool>>,
    apma: SetAutomaton,
}

impl RewriteEngine for InnermostRewriter {
    fn rewrite(&mut self, t: ATerm) -> ATerm {
        let mut stats = RewritingStatistics::default();

        InnermostRewriter::rewrite_aux(&mut self.term_pool.borrow_mut(), &self.apma, t, &mut stats)
    }
}

impl InnermostRewriter {
    pub fn new(term_pool: Rc<RefCell<TermPool>>, spec: &RewriteSpecification) -> InnermostRewriter {
        InnermostRewriter {
            term_pool: term_pool.clone(),
            apma: SetAutomaton::new(&mut term_pool.borrow_mut(), spec, true, false),
        }
    }

    /// Function to rewrite a term 't'. The elements of the automaton 'states' and 'tp' are passed
    /// as separate parameters to satisfy the borrow checker.
    pub(crate) fn rewrite_aux(
        tp: &mut TermPool,
        automaton: &SetAutomaton,
        term: ATerm,
        stats: &mut RewritingStatistics,
    ) -> ATerm {
        let mut stack = InnerStack::default();
        stack.terms.push(ATerm::default());
        stack.add_rewrite(term, 0);
        
        // println!("{}", stack);

        while let Some(config) = stack.configs.pop() {
            match config {
                Config::Rewrite(result) => {
                    let term = stack.terms.pop().unwrap();

                    let symbol = get_data_function_symbol(tp, &term);
                    let arguments = get_data_arguments(tp, &term);

                    // For all the argument we reserve space on the stack.
                    let top_of_stack = stack.terms.len();
                    for _ in 0..arguments.len() {
                        stack.terms.push(ATerm::default());
                    }

                    stack.add_result(symbol, arguments.len(), result);
                    for (offset, arg) in arguments.into_iter().enumerate() {
                        stack.add_rewrite(arg, top_of_stack + offset);
                    }
                }
                Config::Construct(symbol, arity, index) => {
                    // Take the last arity arguments.
                    let arguments = &stack.terms[stack.terms.len() - arity..];

                    let term: ATerm = if arguments.is_empty() {
                        symbol.into()
                    } else {
                        tp.create_data_application(&symbol.into(), arguments).into()
                    };

                    // Remove the arguments from the stack.
                    stack.terms.drain(stack.terms.len() - arity..);

                    match InnermostRewriter::find_match(tp, automaton, &term, stats) {
                        Some(ema) => {
                            // TODO: This ignores the first element of the stack, but that is kind of difficult to deal with.
                            let top_of_stack = stack.terms.len();
                            stack.terms.reserve(ema.stack_size - 1); // We already reserved space for the result.
                            for _ in 0..ema.stack_size - 1 {
                                stack.terms.push(ATerm::default());
                            }

                            let mut first = true;
                            for config in &ema.innermost_stack {
                                match config {
                                    Config::Construct(symbol, arity, offset) => {
                                        if first {
                                            // The first result must be placed on the original result index.
                                            stack.add_result(symbol.clone(), *arity, index);
                                        } else {
                                            // Otherwise, we put it on the end of the stack.
                                            stack.add_result(
                                                symbol.clone(),
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

                            /*
                            println!(
                                "{}, {}, {}",
                                ema.stack_size,
                                ema.variables.iter().format_with(", ", |element, f| {
                                    f(&format_args!("{} -> {}", element.0, element.1))
                                }),
                                ema.innermost_stack.iter().format("\n")
                            );
                            */

                            debug_assert!(ema.stack_size != 1 || ema.variables.len() <= 1, "There can only be a single variable in the right hand side");
                            if ema.stack_size == 1 && ema.variables.len() == 1{
                                // This is a special case where we place the result on the correct position immediately.
                                // The right hand side is only a variable
                                stack.terms[index] = get_position(&term, &ema.variables[0].0);
                            } else {
                                for (position, index) in &ema.variables {
                                    // Add the positions to the stack.
                                    stack.terms[top_of_stack + index - 1] =
                                        get_position(&term, position);
                                }
                            }

                            stats.rewrite_steps += 1;
                            // println!("applying rule {}", ema.announcement.rule);
                        }
                        None => {
                            // Add the term on the stack.
                            stack.terms[index] = term;
                        }
                    }
                }
            }

            /*/
            println!("{}", stack);

            for (index, term) in stack.terms.iter().enumerate() {
                if term.is_default() {
                    debug_assert!(
                        stack.configs.iter().any(|x| {
                            match x {
                                Config::Construct(_, _, result) => index == *result,
                                Config::Rewrite(result) => index == *result,
                            }
                        }),
                        "This default term {index} is not a result of any operation."
                    );
                }
            }
            */
        }

        debug_assert!(
            stack.terms.len() == 1,
            "Expect exactly one term on the result stack"
        );

        stack
            .terms
            .pop()
            .expect("The result should be the last element on the stack")
    }

    /// Use the APMA to find a match for the given term.
    fn find_match<'a>(
        tp: &mut TermPool,
        automaton: &'a SetAutomaton,
        t: &ATerm,
        stats: &mut RewritingStatistics,
    ) -> Option<&'a EnhancedMatchAnnouncement> {
        // Start at the initial state
        let mut state_index = 0;
        loop {
            let state = &automaton.states[state_index];

            // Get the symbol at the position state.label
            stats.symbol_comparisons += 1;
            let symbol = get_data_function_symbol(tp, &get_position(t, &state.label));

            // Get the transition for the label and check if there is a pattern match
            if let Some(transition) = state.transitions.get(symbol.operation_id()) {
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
                // Function symbol does not occur in the set automaton, so cannot match.
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
            let rhs = c.semi_compressed_rhs.evaluate(t, tp);
            let lhs = c.semi_compressed_lhs.evaluate(t, tp);

            let rhs_normal = InnermostRewriter::rewrite_aux(tp, automaton, rhs, stats);
            let lhs_normal = InnermostRewriter::rewrite_aux(tp, automaton, lhs, stats);

            if lhs_normal != rhs_normal && c.equality || lhs_normal == rhs_normal && !c.equality {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use ahash::AHashSet;
    use mcrl2::atermpp::{random_term, TermPool};

    use crate::{
        utilities::to_data_expression, InnermostRewriter, RewriteEngine, RewriteSpecification,
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
        let term = to_data_expression(&mut tp.borrow_mut(), &term, &AHashSet::new());

        assert_eq!(
            inner.rewrite(term.clone()),
            term,
            "Should be in normal form for no rewrite rules"
        );
    }
}
