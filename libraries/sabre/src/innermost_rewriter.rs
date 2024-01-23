use std::{cell::RefCell, rc::Rc, ops::Deref};

use log::{trace, info, debug};
use mcrl2::{
    aterm::{ATerm, TermPool, ATermTrait},
    data::{DataExpressionRef, DataExpression, DataApplication},
};

use crate::{
    matching::{nonlinear::{check_equivalence_classes, derive_equivalence_classes, EquivalenceClass}, conditions::{extend_conditions, EMACondition}}, set_automaton::{MatchAnnouncement, SetAutomaton}, utilities::{get_position, Config, InnermostStack, RHSStack}, RewriteEngine, RewriteSpecification, RewritingStatistics, Rule
};

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

        let apma =  SetAutomaton::new(spec, AnnouncementInnermost::new, true);
        
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
        automaton: &SetAutomaton<AnnouncementInnermost>,
        term: DataExpression,
        stats: &mut RewritingStatistics,
    ) -> DataExpression {
        debug_assert!(!term.is_default(), "Cannot rewrite the default term");

        stats.recursions += 1;

        let mut stack = InnermostStack::default();        
        let mut write_terms =  stack.terms.write();
        let mut write_configs =  stack.configs.write();
        write_terms.push(DataExpressionRef::default());
        InnermostStack::add_rewrite(&mut write_configs, &mut write_terms, term.copy(), 0);
        drop(write_terms);
        drop(write_configs);        

        loop {
            trace!("{}", stack);

            let mut write_configs = stack.configs.write();
            if let Some(config) = write_configs.pop() {
                match config {
                    Config::Rewrite(result) => {
                        let mut write_terms = stack.terms.write();
                        let term = write_terms.pop().unwrap();

                        let symbol = term.data_function_symbol();
                        let arguments = term.data_arguments();

                        // For all the argument we reserve space on the stack.
                        let top_of_stack = write_terms.len();
                        for _ in 0..arguments.len() {
                            write_terms.push(Default::default());
                        }

                        let symbol = write_configs.protect(&symbol.into());
                        InnermostStack::add_result(&mut write_configs, symbol.into(), arguments.len(), result);
                        for (offset, arg) in arguments.into_iter().enumerate() {
                            InnermostStack::add_rewrite(&mut write_configs, &mut write_terms, arg.into(), top_of_stack + offset);
                        }
                    }
                    Config::Construct(symbol, arity, index) => {
                        // Take the last arity arguments.
                        let mut write_terms = stack.terms.write();
                        let length = write_terms.len();

                        let arguments = &write_terms[length - arity..];

                        let term: DataExpression = if arguments.is_empty() {
                            symbol.protect().into()
                        } else {
                            DataApplication::from_refs(tp, &symbol.copy().into(), arguments).into()
                        };

                        // Remove the arguments from the stack.
                        write_terms.drain(length - arity..);

                        match InnermostRewriter::find_match(tp, automaton, &term, stats) {
                            Some((announcement, annotation)) => {
                                debug!("term {} applying rule {}", term, announcement.rule);
                                InnermostStack::integrate(&mut write_configs, &mut write_terms, &annotation.rhs_stack, &term, index);
                                stats.rewrite_steps += 1;
                            }
                            None => {
                                // Add the term on the stack.
                                let t = write_terms.protect(&term);
                                write_terms[index] = t.into();
                            }
                        }
                    }
                }

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
        automaton: &'a SetAutomaton<AnnouncementInnermost>,
        t: &DataExpression,
        stats: &mut RewritingStatistics,
    ) -> Option<(&'a MatchAnnouncement, &'a AnnouncementInnermost)> {
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
                for (announcement, annotation) in &transition.announcements {
                    if check_equivalence_classes(t, &annotation.equivalence_classes)
                        && InnermostRewriter::check_conditions(tp, automaton, t, annotation, stats)
                    {
                        // We found a matching pattern
                        return Some((announcement, annotation));
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

    /// Checks whether the condition holds for given match announcement.
    fn check_conditions(
        tp: &mut TermPool,
        automaton: &SetAutomaton<AnnouncementInnermost>,
        t: &ATerm,
        announcement: &AnnouncementInnermost,
        stats: &mut RewritingStatistics,
    ) -> bool {
        for c in &announcement.conditions {
            let rhs: DataExpression = c.semi_compressed_rhs.evaluate(t, tp).into();
            let lhs: DataExpression = c.semi_compressed_lhs.evaluate(t, tp).into();

            let rhs_normal = InnermostRewriter::rewrite_aux(tp, automaton, rhs, stats);
            let lhs_normal = if &lhs == tp.true_term() {
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

/// Innermost Adaptive Pattern Matching Automaton (APMA) rewrite engine.
pub struct InnermostRewriter {
    tp: Rc<RefCell<TermPool>>,
    apma: SetAutomaton<AnnouncementInnermost>,
}

pub(crate) struct AnnouncementInnermost {
    /// Positions in the pattern with the same variable, for non-linear patterns
    equivalence_classes: Vec<EquivalenceClass>,

    /// Conditions for the left hand side.
    conditions: Vec<EMACondition>,

    /// The innermost stack for the right hand side of the rewrite rule.
    rhs_stack: RHSStack,
}

impl AnnouncementInnermost {
    fn new(rule: &Rule) -> AnnouncementInnermost {
        AnnouncementInnermost {
            conditions: extend_conditions(rule),
            equivalence_classes: derive_equivalence_classes(rule),
            rhs_stack: RHSStack::new(rule)
        }
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
