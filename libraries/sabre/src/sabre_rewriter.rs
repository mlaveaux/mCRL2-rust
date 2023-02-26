use std::rc::Rc;

//use utilities::set_automaton;
use mcrl2_rust::atermpp::{ATerm, TermPool};
use mcrl2_rust::data::DataSpecification;

/// A shared trait for all the rewriters
trait RewriteEngine
{
    /// Rewrites the given term into normal form.
    fn rewrite(&mut self, term: ATerm) -> ATerm;
}

pub struct RewritingStatistics 
{
    pub rewrite_steps: usize,
    pub symbol_comparisons: usize
}

pub struct SabreRewriter 
{
    term_pool: Rc<TermPool>,
    //automaton: SetAutomaton,
}

impl RewriteEngine for SabreRewriter 
{
    fn rewrite(&mut self, term: ATerm) -> ATerm 
    {
        self.stack_based_normalise(term)
    }
}

impl SabreRewriter
{
    fn new(tp: Rc<TermPool>, spec: DataSpecification) -> Self 
    {
        SabreRewriter {
            term_pool: tp,
            //automaton: SetAutomaton::construct(spec)
        }
    }

    /// Function to rewrite a term. See the module documentation.
    pub fn stack_based_normalise(&mut self, t: ATerm) -> ATerm 
    {
        t
        //SabreRewriter::stack_based_normalise_aux(&mut self.automaton.term_pool, &self.automaton.states, t, stats, true)
    }
    /*
    /// The _aux function splits the term pool and the states to make borrow checker happy.
    /// We can now mutate the term pool and read the state and transition information at the same time
    fn stack_based_normalise_aux(tp: &mut TermPool, states: &Vec<State>, t: StoredTerm, stats: &mut RewritingStatistics, _gc: bool) -> StoredTerm 
    {
        // If a normal form is known for term t return it immediately, especially useful for rewriting conditions
        if let Some(nf) = t.normal_form() {
            return nf.clone();
        }

        // We explore the configuration tree depth first using a ConfigurationStack
        let mut cl = ConfigurationStack::new(0, t.clone());
        let mut rewrites_since_gc:usize = 0;

        // Big loop until we know we have a normal form
        'outer: loop {
            // Inner loop so that we can easily break; to the next iteration
            'skip_point: loop {

                // Check if there is any configuration leaf left to explore, if not we have found a normal form
                if let Some(leaf_index) = cl.get_unexplored_leaf() {
                    let leaf = &mut cl.configuration_stack[leaf_index];
                    
                    // A "side stack" is used besides the configuration stack to remember a couple of things.
                    // There are 4 options.

                    // 1. There is nothing on the side stack for this configuration. This means we have
                    // never seen this configuration before. It is a bud that needs to be explored.

                    // In the remaining three cases we have seen the configuration before and have pruned back,
                    // either because of applying a rewrite rule or just because our depth first search
                    // has hit the bottom and needs to explore a new branch.
                    // 2. There is a side branch. That means we had a hyper transition.
                    // The configuration has multiple children in the overall tree.
                    // We have already explored some of these child configurations and parked the remaining
                    // on the side stack. We are going to explore the next child configuration.
                    // 3. There is a delayed rewrite rule. We had found a matching rewrite rule the first time
                    // visiting this configuration but did not want to apply it yet.
                    // At the moment this is the case for "duplicating" rewrite rules that copy some subterms.
                    // We first examine side branches on the side stack, meaning that we have explored all
                    // child configurations. Which, in turn, means that the subterms of the term
                    // in the current configuration are in normal form.
                    // Thus the duplicating rewrite rule only duplicates terms that are in normal form.
                    // 4. There is another type of delayed rewrite rule: one that is non-linear or has
                    //  a condition. We had found a matching rewrite rule the first time
                    // visiting this configuration but our strategy dictates that we only perform
                    // the condition check and check on the equivalence of positions when the subterms
                    // are in normal form. We perform the checks and apply the rewrite rule if it indeed matches.
                    match ConfigurationStack::pop_side_branch_leaf(&mut cl.side_branch_stack, leaf_index) {
                        None => {
                            //We are exploring a bud. If the subterm in the configuration has a known normal form we use that
                            if let Some(nf) = leaf.subterm.normal_form() {
                                //We should be able to replace the subterm and prune back (regardless of the current subterm).
                                //The question is how far we need to prune back.
                                //A guess would be the highest configuration in the stack with the same position
                                //Put perhaps even higher, due to lookahead
                                if nf == leaf.subterm {
                                    //Go back to a configuration higher up that has a side branch
                                    //If there is no side branch we are finished with rewriting
                                    let prev = cl.get_prev_with_side_info();
                                    cl.current_node = prev;
                                    if let Some(n) = prev {
                                        cl.jump_back(n, tp);
                                    }
                                    break 'skip_point;
                                }
                            }
                            //Observe a symbol according to the state label of the set automaton
                            let symbol = leaf.subterm.get_position(&states[leaf.state].label).get_head_symbol();
                            stats.symbol_comparisons.add_assign(1);

                            //Get the transition belonging to the observed symbol
                            let tr = &states[leaf.state].transitions[symbol];

                            //Loop over the match announcements of the transition
                            for ema in &tr.announcements {
                                if ema.is_duplicating { //We do not want to apply duplicating rules straight away
                                    if ema.equivalence_classes.is_empty() && ema.conditions.is_empty() {
                                        cl.side_branch_stack.push(SideInfo{ corresponding_configuration: leaf_index, info: SideInfoType::DelayedRewriteRule(&ema)});
                                    } else {
                                        cl.side_branch_stack.push(SideInfo{ corresponding_configuration: leaf_index, info: SideInfoType::EquivalenceAndConditionCheck(&ema)});
                                    }
                                } else {
                                    if ema.conditions.is_empty() && ema.equivalence_classes.is_empty() {
                                        //For a rewrite rule that is not duplicating or has a condition we just apply it straight away
                                        SabreRewriter::apply_rewrite_rule(&mut rewrites_since_gc,ema,leaf.subterm.clone(),leaf_index,&mut cl,tp,states,stats);
                                        break 'skip_point;
                                    } else {
                                        //We delay the condition checks
                                        cl.side_branch_stack.push(SideInfo{ corresponding_configuration: leaf_index, info: SideInfoType::EquivalenceAndConditionCheck(&ema)});
                                    }
                                }
                            }
                            if tr.destinations.is_empty() {
                                //If there is no destination we are done matching and go back to the previous
                                //configuration on the stack with information on the side stack.
                                //Note, it could be that we stay at the same configuration and apply a rewrite
                                //rule that was just discovered whilst exploring this configuration.
                                let prev = cl.get_prev_with_side_info();
                                cl.current_node = prev;
                                if let Some(n) = prev {
                                    cl.jump_back(n, tp);
                                }
                            } else {
                                //Grow the bud; if there is more than one destination a SideBranch object will be placed on the side stack
                                let tr_slice = tr.destinations.as_slice();
                                cl.grow( leaf_index, tr_slice);
                            }
                        }
                        Some(sit) => {
                            match sit {
                                SideInfoType::SideBranch(sb) => {
                                    //If there is a SideBranch pick the next child configuration
                                    cl.grow(leaf_index, sb);
                                }
                                SideInfoType::DelayedRewriteRule(ema) => {
                                    //apply the delayed rewrite rule
                                    SabreRewriter::apply_rewrite_rule(&mut rewrites_since_gc,ema,leaf.subterm.clone(),leaf_index,&mut cl,tp,states,stats);
                                }
                                SideInfoType::EquivalenceAndConditionCheck(ema) => {
                                    //Apply the delayed rewrite rule if the conditions hold
                                    if SabreRewriter::conditions_hold(ema,leaf,tp,states,stats) {
                                        SabreRewriter::apply_rewrite_rule(&mut rewrites_since_gc,ema,leaf.subterm.clone(),leaf_index,&mut cl,tp,states,stats);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    //No configuration left to explore, we have found a normal form
                    break 'outer;
                }
            }
        }
        let final_term = cl.compute_final_term(tp);
        //Set the normal form for the original term. That information can now be used later on.
        t.set_normal_form(final_term.clone());
        return final_term;
    }

    /// Apply a rewrite rule and prune back
    #[inline]
    fn apply_rewrite_rule(rewrites_since_gc: &mut usize, ema:&EnhancedMatchAnnouncement, leaf_subterm:StoredTerm, leaf_index:usize, cl: &mut ConfigurationStack, tp: &mut TermPool, states: &Vec<State>, stats: &mut RewritingStatistics) {
        stats.rewrite_steps += 1;
        rewrites_since_gc.add_assign(1);
        //Computes the new subterm of the configuration
        let new_subterm = {
            let new_sub_subterm = ema.semi_compressed_rhs.evaluate(leaf_subterm.get_position(&ema.announcement.position),tp);
            tp.substitute(&leaf_subterm, new_sub_subterm, &ema.announcement.position.indices, ema.announcement.position.len())
        };
        //The match announcement tells us how far we need to prune back.
        let prune_point = leaf_index - ema.announcement.symbols_seen;
        cl.prune(prune_point + 0, new_subterm, tp, states);
    }*/

    /*
    /// Checks conditions and subterm equality of non-linear patterns.
    #[inline]
    fn conditions_hold(ema:&EnhancedMatchAnnouncement, leaf:&Configuration, tp: &mut TermPool, states: &Vec<State>, stats: &mut RewritingStatistics) -> bool 
    {
        for c in &ema.conditions 
        {
            let rhs = c.semi_compressed_rhs.evaluate(&leaf.subterm.get_position(&ema.announcement.position), tp);
            let lhs = c.semi_compressed_lhs.evaluate(&leaf.subterm.get_position(&ema.announcement.position), tp);

            if lhs == rhs && c.equality 
            {
                //do nothing
            } 
            else 
            {
                let rhs_normal = SabreRewriter::stack_based_normalise_aux(tp, states,rhs.clone(), stats,false);
                let lhs_normal = SabreRewriter::stack_based_normalise_aux(tp, states,lhs.clone(), stats,false);
                
                if !(lhs_normal == rhs_normal && c.equality) || (lhs_normal != rhs_normal && !c.equality) 
                {
                    return EquivalenceClass::equivalences_hold(leaf.subterm.clone(), &ema.equivalence_classes);
                }
            }
        }
        
        false
    }
    */
}