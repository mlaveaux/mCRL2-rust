use std::{cell::RefCell, rc::Rc, fmt};

use mcrl2_rust::{atermpp::{ATerm, TermPool}, data::DataFunctionSymbol};

use crate::{
    set_automaton::{
        get_data_function_symbol, EnhancedMatchAnnouncement, SetAutomaton, get_data_arguments, check_equivalence_classes,
    },
    RewriteEngine, RewriteSpecification, RewritingStatistics, utilities::get_position,
};

#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Config {
    Rewrite(usize),
    Construct(DataFunctionSymbol, usize, usize), // Constructs f with arity at the given index.
}

#[derive(Default)]
struct InnerStack
{
    configs: Vec<Config>,
    terms: Vec<ATerm>,
}

impl fmt::Display for InnerStack
{
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
            Config::Rewrite(index) => write!(f, "Rewrite({})", index),
            Config::Construct(symbol, arity, index) => write!(f, "Construct({}, {}, {})", symbol, arity, index)
        }
    }

}

impl InnerStack
{
    fn add_result(&mut self, symbol: DataFunctionSymbol, arity: usize, index: usize) {
        self.configs.push(Config::Construct(symbol, arity, index));
    }

    fn add_rewrite(&mut self, term: ATerm) {
        self.configs.push(Config::Rewrite(self.terms.len()));
        self.terms.push(ATerm::default());
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
            term_pool,
            apma: SetAutomaton::new(spec, true, false),
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
        stack.add_rewrite(term);

        loop {
            // println!("{}", stack);

            match stack.configs.pop() {
                Some(config) => {
                    match config {
                        Config::Rewrite(index) => {
                            let term = stack.terms.pop().expect("There should be a last element");
                            
                            // Rewrite all the subterms.
                            let symbol = get_data_function_symbol(&term);
                            let arguments = get_data_arguments(&term);                          
                            stack.add_result(symbol, arguments.len(), index);
                            
                            for arg in arguments.into_iter() {
                                stack.add_rewrite(arg);
                            }
                        }
                        Config::Construct(symbol, arity, index) => {
                            // Take the last arity arguments.
                            let arguments = &stack.terms[stack.terms.len() - arity..];
                            
                            let term: ATerm = if arguments.is_empty() {
                                symbol.into()
                            } else {
                                tp.create_data_application(&symbol.into(), &arguments).into()
                            };

                            // Remove the arguments from the stack.
                            if arity > 0 {
                                stack.terms.drain(stack.terms.len() - arity..);
                            }
                            
                            match InnermostRewriter::find_match(tp, automaton, &term, stats) {
                                Some(ema) =>  {                                    
                                    let top_of_stack = stack.terms.len() - 1; // We replace the result term
                                    stack.terms.reserve(ema.stack_size - 1);
                                    for _ in 0..ema.stack_size - 1 {
                                        stack.terms.push(ATerm::default());
                                    }

                                    let mut first = true;
                                    for config in &ema.innermost_stack {
                                        match config {
                                            Config::Construct(symbol, arity, offset) => {
                                                if first {
                                                    // The first result must be placed on the original result.
                                                    stack.add_result(symbol.clone(), *arity, index); 
                                                } else {
                                                    // Otherwise, we put it on the end of the stack.
                                                    stack.add_result(symbol.clone(), *arity, top_of_stack + offset);
                                                }
                                            },
                                            Config::Rewrite(_) => {
                                                panic!("This case should not happen");
                                            }
                                        }
                                        first = false;
                                    }

                                    for (position, index) in &ema.positions {
                                        // Add the positions to the stack.
                                        stack.terms[top_of_stack + index] = get_position(&term, position);
                                    }

                                    stats.rewrite_steps += 1;
                                    // println!("applying rule {}", ema.announcement.rule);
                                },
                                None => {
                                    // Add the term on the stack.
                                    stack.terms[index] = term;
                                }
                            }
                        }
                    }
                }
                None => { break; }
            }
        }
        
        assert!(stack.terms.len() == 1, "Expect exactly one term on the result stack");
        return stack.terms.pop().expect("The result should be the last element on the stack");
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
            let symbol = get_data_function_symbol(&get_position(t, &state.label));

            // Get the transition for the label and check if there is a pattern match
            let transition = &state.transitions[symbol.operation_id()];
            for ema in &transition.announcements {
                if check_equivalence_classes(&t, &ema.equivalence_classes) && InnermostRewriter::check_conditions(tp, automaton, t, ema, stats) {
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
