use std::{cell::RefCell, rc::Rc, ops::Deref};

use log::{debug, trace, info};
use mcrl2::{aterm::TermPool, data::{DataExpressionRef, BoolSort, DataExpression}};

use crate::{
    RewriteSpecification,
    set_automaton::{
        check_equivalence_classes, EnhancedMatchAnnouncement,
        SetAutomaton,
    },
    utilities::{get_position, Configuration, ConfigurationStack, SideInfo, SideInfoType},
};

/// A shared trait for all the rewriters
pub trait RewriteEngine {
    /// Rewrites the given term into normal form.
    fn rewrite(&mut self, term: DataExpression) -> DataExpression;
}

#[derive(Default)]
pub struct RewritingStatistics {
    /// Count the number of rewrite rules applied
    pub rewrite_steps: usize, 
    /// Counts the number of times symbols are compared.
    pub symbol_comparisons: usize, 
    /// The number of times rewrite is called recursively (to rewrite conditions etc)
    pub recursions: usize, 
}

// A set automaton based rewrite engine described in  Mark Bouwman, Rick Erkens:
// Term Rewriting Based On Set Automaton Matching. CoRR abs/2202.08687 (2022)
pub struct SabreRewriter {
    term_pool: Rc<RefCell<TermPool>>,
    automaton: SetAutomaton,
}

impl RewriteEngine for SabreRewriter {
    fn rewrite(&mut self, term: DataExpression) -> DataExpression {
        self.stack_based_normalise(term)
    }
}

impl SabreRewriter {
    pub fn new(tp: Rc<RefCell<TermPool>>, spec: &RewriteSpecification) -> Self {

        let automaton =  SetAutomaton::new(spec, false);
        info!("ATerm pool: {}", tp.borrow());
        SabreRewriter {
            term_pool: tp.clone(),
            automaton,
        }
    }

    /// Function to rewrite a term. See the module documentation.
    pub fn stack_based_normalise(&mut self, t: DataExpression) -> DataExpression {
        let mut stats = RewritingStatistics::default();
        
        let result = SabreRewriter::stack_based_normalise_aux(
            &mut self.term_pool.borrow_mut(),
            &self.automaton,
            t,
            &mut stats,
        );
        info!("{} rewrites, {} single steps and {} symbol comparisons", stats.recursions, stats.rewrite_steps, stats.symbol_comparisons);
        result
    }

    /// The _aux function splits the [TermPool] pool and the [SetAutomaton] to make borrow checker happy.
    /// We can now mutate the term pool and read the state and transition information at the same time
    fn stack_based_normalise_aux(
        tp: &mut TermPool,
        automaton: &SetAutomaton,
        t: DataExpression,
        stats: &mut RewritingStatistics,
    ) -> DataExpression {
        // We explore the configuration tree depth first using a ConfigurationStack
        let mut cs = ConfigurationStack::new(0, t);

        // Big loop until we know we have a normal form
        'outer: loop {
            // Inner loop so that we can easily break; to the next iteration
            'skip_point: loop {
                trace!("{}", cs);

                // Check if there is any configuration leaf left to explore, if not we have found a normal form
                if let Some(leaf_index) = cs.get_unexplored_leaf() {
                    let leaf = &mut cs.stack[leaf_index];

                    // A "side stack" is used besides the configuration stack to
                    // remember a couple of things. There are 4 options.

                    // 1. There is nothing on the side stack for this
                    //    configuration. This means we have never seen this
                    //    configuration before. It is a bud that needs to be
                    //    explored.

                    // In the remaining three cases we have seen the
                    // configuration before and have pruned back, either because
                    // of applying a rewrite rule or just because our depth
                    // first search has hit the bottom and needs to explore a
                    // new branch.

                    // 2. There is a side branch. That means we had a hyper
                    //    transition. The configuration has multiple children in
                    //    the overall tree. We have already explored some of these
                    //    child configurations and parked the remaining on the side
                    //    stack. We are going to explore the next child
                    //    configuration.

                    // 3. There is a delayed rewrite rule. We had found a
                    //    matching rewrite rule the first time visiting this
                    //    configuration but did not want to apply it yet. At the
                    //    moment this is the case for "duplicating" rewrite rules
                    //    that copy some subterms. We first examine side branches
                    //    on the side stack, meaning that we have explored all
                    //    child configurations. Which, in turn, means that the
                    //    subterms of the term in the current configuration are in
                    //    normal form. Thus the duplicating rewrite rule only
                    //    duplicates terms that are in normal form.

                    // 4. There is another type of delayed rewrite rule: one
                    //    that is non-linear or has a condition. We had found a
                    //    matching rewrite rule the first time visiting this
                    //    configuration but our strategy dictates that we only
                    //    perform the condition check and check on the equivalence
                    //    of positions when the subterms are in normal form. We
                    //    perform the checks and apply the rewrite rule if it
                    //    indeed matches.
                    match ConfigurationStack::pop_side_branch_leaf(
                        &mut cs.side_branch_stack,
                        leaf_index,
                    ) {
                        None => {
                            // Observe a symbol according to the state label of the set automaton.
                            let pos: DataExpressionRef = get_position(leaf.subterm.deref(), &automaton.states[leaf.state].label).into();
                            let function_symbol = pos.data_function_symbol();
                            stats.symbol_comparisons += 1;

                            // Get the transition belonging to the observed symbol
                            if let Some(tr) = &automaton
                                .transitions
                                .get(&(leaf.state, function_symbol.operation_id()))
                            {
                                // Loop over the match announcements of the transition
                                for ema in &tr.announcements {
                                    if ema.conditions.is_empty()
                                        && ema.equivalence_classes.is_empty()
                                    {
                                        if ema.is_duplicating {
                                            // We do not want to apply duplicating rules straight away
                                            cs.side_branch_stack.push(SideInfo {
                                                corresponding_configuration: leaf_index,
                                                info: SideInfoType::DelayedRewriteRule(ema),
                                            });
                                        } else {
                                            // For a rewrite rule that is not duplicating or has a condition we just apply it straight away
                                            SabreRewriter::apply_rewrite_rule(
                                                tp,
                                                automaton,
                                                ema,
                                                leaf_index,
                                                &mut cs,
                                                stats,
                                            );
                                            break 'skip_point;
                                        }
                                    } else {
                                        // We delay the condition checks
                                        cs.side_branch_stack.push(SideInfo {
                                            corresponding_configuration: leaf_index,
                                            info: SideInfoType::EquivalenceAndConditionCheck(ema),
                                        });
                                    }
                                }

                                if tr.destinations.is_empty() {
                                    // If there is no destination we are done matching and go back to the previous
                                    // configuration on the stack with information on the side stack.
                                    // Note, it could be that we stay at the same configuration and apply a rewrite
                                    // rule that was just discovered whilst exploring this configuration.
                                    let prev = cs.get_prev_with_side_info();
                                    cs.current_node = prev;
                                    if let Some(n) = prev {
                                        cs.jump_back(n, tp);
                                    }
                                } else {
                                    // Grow the bud; if there is more than one destination a SideBranch object will be placed on the side stack
                                    let tr_slice = tr.destinations.as_slice();
                                    cs.grow(leaf_index, tr_slice);
                                }
                            } else {
                                // There is no outgoing edges for the head symbol of this term and the stack is empty.
                                // TODO: Not sure if this is necessary or returning leaf.subterm is sufficient.
                                return cs.compute_final_term(tp);
                            }
                        }
                        Some(sit) => {
                            match sit {
                                SideInfoType::SideBranch(sb) => {
                                    // If there is a SideBranch pick the next child configuration
                                    cs.grow(leaf_index, sb);
                                }
                                SideInfoType::DelayedRewriteRule(ema) => {
                                    // apply the delayed rewrite rule
                                    SabreRewriter::apply_rewrite_rule(
                                        tp,
                                        automaton,
                                        ema,
                                        leaf_index,
                                        &mut cs,
                                        stats,
                                    );
                                }
                                SideInfoType::EquivalenceAndConditionCheck(ema) => {
                                    // Apply the delayed rewrite rule if the conditions hold
                                    if check_equivalence_classes(
                                        &leaf.subterm,
                                        &ema.equivalence_classes,
                                    ) && SabreRewriter::conditions_hold(
                                        tp, automaton, ema, leaf, stats,
                                    ) {
                                        SabreRewriter::apply_rewrite_rule(
                                            tp,
                                            automaton,
                                            ema,
                                            leaf_index,
                                            &mut cs,
                                            stats,
                                        );
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // No configuration left to explore, we have found a normal form
                    break 'outer;
                }
            }
        }

        cs.compute_final_term(tp)
    }

    /// Apply a rewrite rule and prune back
    fn apply_rewrite_rule(
        tp: &mut TermPool,
        automaton: &SetAutomaton,
        ema: &EnhancedMatchAnnouncement,
        leaf_index: usize,
        cs: &mut ConfigurationStack<'_>,
        stats: &mut RewritingStatistics,
    ) {
        stats.rewrite_steps += 1;
        let leaf_subterm = &cs.stack[leaf_index].subterm;

        // Computes the new subterm of the configuration
        let new_subterm = ema
            .semi_compressed_rhs
            .evaluate(&get_position(leaf_subterm.deref(), &ema.announcement.position), tp).into();

        debug!(
            "rewrote {} to {} using rule {}",
            &leaf_subterm, &new_subterm, ema.announcement.rule
        );

        // The match announcement tells us how far we need to prune back.
        let prune_point = leaf_index - ema.announcement.symbols_seen;
        cs.prune(tp, automaton, prune_point, new_subterm);
    }

    /// Checks conditions and subterm equality of non-linear patterns.
    fn conditions_hold(
        tp: &mut TermPool,
        automaton: &SetAutomaton,
        ema: &EnhancedMatchAnnouncement,
        leaf: &Configuration,
        stats: &mut RewritingStatistics,
    ) -> bool {
        for c in &ema.conditions {
            let subterm = get_position(leaf.subterm.deref(), &ema.announcement.position);

            let rhs: DataExpression = c.semi_compressed_rhs.evaluate(&subterm, tp).into();
            let lhs: DataExpression = c.semi_compressed_lhs.evaluate(&subterm, tp).into();

            // Equality => lhs == rhs.
            if !c.equality || lhs != rhs {
                let rhs_normal =
                    SabreRewriter::stack_based_normalise_aux(tp, automaton, rhs, stats);
                let lhs_normal = if lhs == BoolSort::true_term().into() {
                    // TODO: Store the conditions in a better way. REC now uses a list of equalities while mCRL2 specifications have a simple condition.
                    lhs
                } else {
                    SabreRewriter::stack_based_normalise_aux(tp, automaton, lhs, stats)
                };

                // If lhs != rhs && !equality OR equality && lhs == rhs.
                if (!c.equality && lhs_normal == rhs_normal)
                    || (c.equality && lhs_normal != rhs_normal)
                {
                    return false;
                }
            }
        }

        true
    }
}
