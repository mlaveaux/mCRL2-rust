use std::{cell::RefCell, rc::Rc};

use mcrl2_rust::{atermpp::{ATerm, TermPool}, data::DataFunctionSymbol};

use crate::{
    set_automaton::{
        get_data_function_symbol, get_data_position, EnhancedMatchAnnouncement, SetAutomaton, get_data_arguments,
    },
    RewriteEngine, RewriteSpecification, RewritingStatistics,
};

enum Config {
    Rewrite(),
    Result(DataFunctionSymbol, usize),
}

#[derive(Default)]
struct InnerStack
{
    configs: Vec<Config>,
    terms: Vec<ATerm>,
}

impl InnerStack
{
    fn add_result(&mut self, symbol: DataFunctionSymbol, arity: usize) {
        self.configs.push(Config::Result(symbol, arity));
    }

    fn add_rewrite(&mut self, term: ATerm) {
        self.configs.push(Config::Rewrite());
        self.terms.push(term);
    }
}

/// Innermost Adaptive Pattern Matching Automaton (APMA) rewrite engine.
/// Implements the RewriteEngine trait. An APMA uses a modified SetAutomaton.
/// The SetAutomaton::construct function has an 'apma' parameter. If it is set to true.
/// An APMA is created. Construction is almost identical with the difference that no fresh goals are created.
/// It only matches on the root position.
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
            match stack.configs.pop() {
                Some(config) => {
                    match config {
                        Config::Rewrite() => {
                            let term = stack.terms.pop().expect("There should be a last element");
                            
                            // Rewrite all the subterms.
                            let symbol = get_data_function_symbol(&term);
                            let arguments = get_data_arguments(&term);
                            
                            stack.add_result(symbol, arguments.len());
                            for arg in arguments.into_iter() {
                                stack.add_rewrite(arg);
                            }
                        }
                        Config::Result(symbol, arity) => {
                            // Take the last arity arguments.
                            let arguments = &stack.terms[stack.terms.len() - arity..stack.terms.len()];
                            
                            let term: ATerm = if arguments.is_empty() {
                                symbol.into()
                            } else {
                                tp.create_data_application(&symbol.into(), &arguments).into()
                            };

                            // Remove the arguments from the stack
                            stack.terms.drain(stack.terms.len() - arity..);
                            
                            match InnermostRewriter::find_match(tp, automaton, &term, stats) {
                                Some(ema) =>  {
                                    let result = ema.semi_compressed_rhs.evaluate(&term, tp);
                                    // println!("rewrote {} to {} using rule {}", term, result, ema.announcement.rule);
                                    stack.add_rewrite(result);
                                },
                                None => {
                                    // Add the term on the stack.
                                    stack.terms.push(term);
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
        //println!("term {}", t);

        // Start at the initial state
        let mut state_index = 0;
        loop {
            let state = &automaton.states[state_index];

            // Get the symbol at the position state.label
            let symbol = get_data_function_symbol(&get_data_position(t, &state.label));

            //println!("matching: {}", symbol);

            // Get the transition for the label and check if there is a pattern match
            let transition = &state.transitions[symbol.operation_id()];
            for ema in &transition.announcements {
                //println!("announcing {}", ema);

                let mut conditions_hold = true;

                // Check conditions if there are any
                if !ema.conditions.is_empty() {
                    conditions_hold =
                        InnermostRewriter::check_conditions(tp, automaton, t, ema, stats);
                }

                // Check equivalence of subterms for non-linear patterns
                'ec_check: for ec in &ema.equivalence_classes {
                    if ec.positions.len() > 1 {
                        let mut iter_pos = ec.positions.iter();
                        let first_pos = iter_pos.next().unwrap();
                        let first_term = get_data_position(t, first_pos);

                        for other_pos in iter_pos {
                            let other_term = get_data_position(t, other_pos);
                            if first_term != other_term {
                                conditions_hold = false;
                                break 'ec_check;
                            }
                        }
                    }
                }

                if conditions_hold {
                    // We found a matching pattern
                    return Some(ema);
                }
            }

            // If there is no pattern match we check if the transition has a destination state
            if transition.destinations.is_empty() {
                // If there is no destination state there is no pattern match
                //println!("no match");
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

            let holds = (lhs_normal == rhs_normal && c.equality)
                || (lhs_normal != rhs_normal && !c.equality);
            if !holds {
                return false;
            }
        }

        true
    }
}
