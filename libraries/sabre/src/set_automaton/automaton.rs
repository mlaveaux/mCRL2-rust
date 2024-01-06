use std::{collections::VecDeque, fmt::Debug, time::Instant};

use ahash::HashMap;
use log::{debug, warn, log_enabled, trace, info};
use mcrl2::{aterm::{ATerm, ATermTrait, ATermRef, ATermArgs}, data::{DataFunctionSymbol, DataFunctionSymbolRef, is_data_variable, is_data_application, is_data_function_symbol, is_data_abstraction, is_data_where_clause, is_data_untyped_identifier}};
use smallvec::{smallvec, SmallVec};

use crate::{
    rewrite_specification::{RewriteSpecification, Rule},
    utilities::ExplicitPosition,
};

use super::{EnhancedMatchAnnouncement, MatchAnnouncement, MatchGoal};

// The Set Automaton used for matching based on
pub struct SetAutomaton {
    pub(crate) states: Vec<State>,
}

#[derive(Clone, Debug)]
pub struct Transition {
    pub(crate) symbol: DataFunctionSymbol,
    pub(crate) announcements: SmallVec<[EnhancedMatchAnnouncement; 1]>,
    pub(crate) destinations: SmallVec<[(ExplicitPosition, usize); 1]>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct MatchObligation {
    pub pattern: ATerm,
    pub position: ExplicitPosition,
}

#[derive(Debug)]
enum GoalsOrInitial {
    InitialState,
    Goals(Vec<MatchGoal>),
}

/// Adds the given function symbol to the indexed symbols. Errors when a
/// function symbol is overloaded with different arities.
fn add_symbol(
    function_symbol: DataFunctionSymbol,
    arity: usize,
    symbols: &mut Vec<(DataFunctionSymbol, usize)>,
) {
    let index = function_symbol.operation_id();

    if index >= symbols.len() {
        symbols.resize(index + 1, Default::default());
    }

    if symbols[index] == Default::default() {
        symbols[index] = (function_symbol, arity);
    } else {
        assert!(
            symbols[index].1 == arity,
            "Function symbol {} occurs with arity {} and {}",
            function_symbol,
            arity,
            &symbols[index].1
        );
    }
}

/// Returns false iff this is a higher order term, of the shape t(t_0, ..., t_n), or an unknown term.
fn is_supported_term(t: ATermRef<'_>) -> bool {
    for subterm in t.iter() {
        if is_data_application(subterm.borrow()) && !is_data_function_symbol(subterm.arg(0)) {
            warn!("{} is higher order", subterm);
            return false;
        } else if is_data_abstraction(subterm.borrow())
            || is_data_where_clause(subterm.borrow())
            || is_data_untyped_identifier(subterm.borrow())
        {
            warn!("{} contains unsupported construct", subterm);
            return false;
        }
    }

    true
}

/// Checks whether the set automaton can use this rule, no higher order rules or binders.
fn is_supported_rule(rule: &Rule) -> bool {
    // There should be no terms of the shape t(t0,...,t_n)
    if !is_supported_term(rule.rhs.borrow()) || !is_supported_term(rule.lhs.borrow()) {
        return false;
    }

    for cond in &rule.conditions {
        if !is_supported_term(cond.rhs.borrow()) || !is_supported_term(cond.lhs.borrow()) {
            return false;
        }
    }

    true
}

/// Finds all data symbols in the term and adds them to the symbol index.
fn find_symbols(t: ATermRef<'_>, symbols: &mut Vec<(DataFunctionSymbol, usize)>) {
    if is_data_function_symbol(t.borrow()) {
        add_symbol(t.protect().into(), 0, symbols);
    } else if is_data_application(t.borrow()) {
        let mut args = t.arguments();

        // REC specifications should never contain this so it can be a debug error.
        let head = args.next().unwrap();
        assert!(
            is_data_function_symbol(head.borrow()),
            "Higher order term rewrite systems are not supported"
        );

        add_symbol(head.protect().into(), args.len(), symbols);
        for arg in args {
            find_symbols(arg, symbols);
        }
    } else if !is_data_variable(t.borrow()) {
        panic!("Unexpected term {:?}", t);
    }
}

impl SetAutomaton {
    pub fn new(spec: &RewriteSpecification, apma: bool) -> SetAutomaton {
        let start = Instant::now();

        // States are labelled s0, s1, s2, etcetera. state_counter keeps track of count.
        let mut state_counter: usize = 1;

        // Remove rules that we cannot deal with
        let supported_rules: Vec<Rule> = spec
            .rewrite_rules
            .iter()
            .filter(|rule| {
                is_supported_rule(rule)
            })
            .map(Rule::clone)
            .collect();

        // Find the indices of all the function symbols.
        let symbols = {
            let mut symbols = vec![];

            for rule in &supported_rules {
                find_symbols(rule.lhs.borrow(), &mut symbols);
                find_symbols(rule.rhs.borrow(), &mut symbols);

                for cond in &rule.conditions {
                    find_symbols(cond.lhs.borrow(), &mut symbols);
                    find_symbols(cond.rhs.borrow(), &mut symbols);
                }
            }

            symbols
        };

        for (index, (symbol, arity)) in symbols.iter().enumerate() {
            trace!("{}: {} {}", index, symbol, arity);
        }

        // The initial state has a match goals for each pattern. For each pattern l there is a match goal
        // with one obligation l@ε and announcement l@ε.
        let mut match_goals = Vec::<MatchGoal>::new();
        for rr in &supported_rules {
            match_goals.push(MatchGoal {
                obligations: vec![MatchObligation {
                    pattern: rr.lhs.clone(),
                    position: ExplicitPosition::empty_pos(),
                }],
                announcement: MatchAnnouncement {
                    rule: (*rr).clone(),
                    position: ExplicitPosition::empty_pos(),
                    symbols_seen: 0,
                },
            });
        }

        // Match goals need to be sorted so that we can easily check whether a state with a certain
        // set of match goals already exists.
        match_goals.sort();

        // Create the initial state
        let initial_state = State {
            label: ExplicitPosition::empty_pos(),
            transitions: Vec::with_capacity(symbols.len()),
            match_goals: match_goals.clone(),
        };

        // HashMap from goals to state number
        let mut map_goals_state = HashMap::default();

        // Queue of states that still need to be explored
        let mut queue = VecDeque::new();
        queue.push_back(0);

        map_goals_state.insert(match_goals, 0);

        let mut states = vec![initial_state];
        let mut num_of_transitions = 0;

        // Pick a state to explore
        while let Some(s_index) = queue.pop_front() {

            for (symbol, arity) in &symbols {
                let (outputs, destinations) = states.get(s_index).unwrap().derive_transition(
                        &symbol,
                        *arity,
                        &supported_rules,
                        apma,
                    );                    

                // Associate an EnhancedMatchAnnouncement to every transition
                let mut announcements: SmallVec<[EnhancedMatchAnnouncement; 1]> = outputs
                    .into_iter()
                    .map(|x| {
                        EnhancedMatchAnnouncement::new(x)
                    })
                    .collect();

                announcements.sort_by(|ema1, ema2| {
                    ema1.announcement.position.cmp(&ema2.announcement.position)
                });

                // Create transition
                let mut transition = Transition {
                    symbol: symbol.clone(),
                    announcements,
                    destinations: smallvec![],
                };

                // For the destinations we convert the match goal destinations to states
                let mut dest_states = smallvec![];

                // Loop over the hypertransitions
                for (pos, goals_or_initial) in destinations {
                    // Match goals need to be sorted so that we can easily check whether a state with a certain
                    // set of match goals already exists.
                    if let GoalsOrInitial::Goals(goals) = goals_or_initial {
                        if map_goals_state.contains_key(&goals) {
                            // The destination state already exists
                            dest_states.push((pos, *map_goals_state.get(&goals).unwrap()))
                        } else if !goals.is_empty() {
                            // The destination state does not yet exist, create it
                            let new_state = State::new(goals.clone(), symbols.len());
                            states.push(new_state);
                            dest_states.push((pos, state_counter));
                            map_goals_state.insert(goals, state_counter);
                            queue.push_back(state_counter);
                            state_counter += 1;
                        }
                    } else {
                        // The transition is to the initial state
                        dest_states.push((pos, 0));
                    }
                }

                // Add the resulting outgoing transition to the state.
                transition.destinations = dest_states;
                states
                    .get_mut(s_index)
                    .unwrap()
                    .transitions
                    .push(transition);
                num_of_transitions += 1;
            }

            info!("Queue size {}, currently {} states and {} transitions", queue.len(), states.len(), num_of_transitions);
        }

        // Clear the match goals since they are only for debugging purposes.
        if !log_enabled!(log::Level::Debug) {
            for state in &mut states {
                state.match_goals.clear();
            }
        }
        info!("Created set automaton (states: {}, transitions: {}, apma: {}) in {} ms", states.len(), num_of_transitions, apma, (Instant::now() - start).as_millis());

        let result = SetAutomaton { states };
        debug!("{}", result);
        
        result
    }
}

#[derive(Debug)]
pub struct Derivative {
    pub(crate) completed: Vec<MatchGoal>,
    pub(crate) unchanged: Vec<MatchGoal>,
    pub(crate) reduced: Vec<MatchGoal>,
}

#[derive(Debug)]
pub(crate) struct State {
    pub(crate) label: ExplicitPosition,

    /// Note that transitions are indexed by the index given by the OpId from a function symbol.
    pub(crate) transitions: Vec<Transition>,
    pub(crate) match_goals: Vec<MatchGoal>,
}

/// Returns the data function symbol of the given term
pub fn get_data_function_symbol(term: ATermRef<'_>) -> DataFunctionSymbolRef<'_> {
    // If this is an application it is the first argument, otherwise it's the term itself
    if is_data_application(term.borrow()) {
        term.arg(0).upgrade(&term).into()
    } else {
        term.into()
    }
}

/// Returns the data arguments of the term
pub fn get_data_arguments(term: &impl ATermTrait) -> ATermArgs<'_> {
    if is_data_application(term.borrow()) {
        let mut result = term.arguments();
        result.next();
        result
    } else {
        Default::default()
    }
}

impl State {
    /// Derive transitions from a state given a head symbol. The resulting transition is returned as a tuple
    /// The tuple consists of a vector of outputs and a set of destinations (which are sets of match goals).
    /// We don't use the struct Transition as it requires that the destination is a full state, with name.
    /// Since we don't yet know whether the state already exists we just return a set of match goals as 'state'.
    ///
    /// Parameter symbol is the symbol for which the transition is computed
    fn derive_transition(
        &self,
        symbol: &DataFunctionSymbol,
        arity: usize,
        rewrite_rules: &Vec<Rule>,
        apma: bool,
    ) -> (
        Vec<MatchAnnouncement>,
        Vec<(ExplicitPosition, GoalsOrInitial)>,
    ) {
        if symbol.is_default() {
            // For the default symbols nothing will be created
            return (vec![], vec![]);
        }

        // Computes the derivative containing the goals that are completed, unchanged and reduced
        let mut derivative = self.compute_derivative(&symbol, arity);

        // The outputs/matching patterns of the transitions are those who are completed
        let outputs = derivative
            .completed
            .into_iter()
            .map(|x| x.announcement)
            .collect();

        // The new match goals are the unchanged and reduced match goals.
        let mut new_match_goals = derivative.unchanged;
        new_match_goals.append(&mut derivative.reduced);

        let mut destinations = vec![];
        // If we are building an APMA we do not deepen the position or create a hypertransitions
        // with multiple endpoints
        if apma {
            if !new_match_goals.is_empty() {
                destinations.push((
                    ExplicitPosition::empty_pos(),
                    GoalsOrInitial::Goals(new_match_goals),
                ));
            }
        } else {
            // In case we are building a set automaton we partition the match goals
            let partitioned = MatchGoal::partition(new_match_goals);

            // Get the greatest common prefix and shorten the positions
            let mut positions_per_partition = vec![];
            let mut gcp_length_per_partition = vec![];
            for (p, pos) in partitioned {
                positions_per_partition.push(pos);
                let gcp = MatchGoal::greatest_common_prefix(&p);
                let gcp_length = gcp.len();
                gcp_length_per_partition.push(gcp_length);
                let mut goals = MatchGoal::remove_prefix(p, gcp_length);
                goals.sort_unstable();
                destinations.push((gcp, GoalsOrInitial::Goals(goals)));
            }

            // Handle fresh match goals, they are the positions Label(state).i
            // where i is between 2 and the arity + 2 of the function symbol of
            // the transition. Position 1 is the function symbol.
            for i in 2..arity + 2 {
                let mut pos = self.label.clone();
                pos.indices.push(i);

                // Check if the fresh goals are related to one of the existing partitions
                let mut partition_key = None;
                'outer: for (i, part_pos) in positions_per_partition.iter().enumerate() {
                    for p in part_pos {
                        if MatchGoal::pos_comparable(p, &pos) {
                            partition_key = Some(i);
                            break 'outer;
                        }
                    }
                }

                if let Some(key) = partition_key {
                    // If the fresh goals fall in an existing partition
                    let gcp_length = gcp_length_per_partition[key];
                    let pos = ExplicitPosition {
                        indices: SmallVec::from_slice(&pos.indices[gcp_length..]),
                    };

                    // Add the fresh goals to the partition
                    for rr in rewrite_rules {
                        if let GoalsOrInitial::Goals(goals) = &mut destinations[key].1 {
                            goals.push(MatchGoal {
                                obligations: vec![MatchObligation {
                                    pattern: rr.lhs.clone(),
                                    position: pos.clone(),
                                }],
                                announcement: MatchAnnouncement {
                                    rule: (*rr).clone(),
                                    position: pos.clone(),
                                    symbols_seen: 0,
                                },
                            });
                        }
                    }
                } else {
                    // The transition is simply to the initial state
                    // GoalsOrInitial::InitialState avoids unnecessary work of creating all these fresh goals
                    destinations.push((pos, GoalsOrInitial::InitialState));
                }
            }
        }

        // Sort the destination such that transitions which do not deepen the position are listed first
        destinations.sort_unstable_by(|(pos1, _), (pos2, _)| pos1.cmp(pos2));
        (outputs, destinations)
    }

    /// For a transition 'symbol' of state 'self' this function computes which match goals are
    /// completed, unchanged and reduced.
    fn compute_derivative(
        &self,
        symbol: &DataFunctionSymbol,
        arity: usize,
    ) -> Derivative {
        let mut result = Derivative {
            completed: vec![],
            unchanged: vec![],
            reduced: vec![],
        };

        for mg in &self.match_goals {
            debug_assert!(
                !mg.obligations.is_empty(),
                "The obligations should never be empty, should be completed then"
            );

            // Completed match goals
            if mg.obligations.len() == 1
                && mg.obligations.iter().any(|mo| {
                    mo.position == self.label
                        && get_data_function_symbol(mo.pattern.borrow()) == symbol.borrow()
                        && get_data_arguments(&mo.pattern.borrow())
                            .all(|x| is_data_variable(x.borrow())) // Again skip the function symbol
                })
            {
                result.completed.push(mg.clone());
            } else if mg.obligations.iter().any(|mo| {
                mo.position == self.label && get_data_function_symbol(mo.pattern.borrow()) != symbol.borrow()
            }) {
                // Match goal is discarded since head symbol does not match.
            } else if mg.obligations.iter().all(|mo| mo.position != self.label) {
                // Unchanged match goals
                let mut mg = mg.clone();
                if mg.announcement.rule.lhs != mg.obligations.first().unwrap().pattern {
                    mg.announcement.symbols_seen += 1;
                }

                result.unchanged.push(mg);
            } else {
                // Reduce match obligations
                let mut mg = mg.clone();
                let mut new_obligations = vec![];

                for mo in mg.obligations {
                    if get_data_function_symbol(mo.pattern.borrow()) == symbol.borrow() && mo.position == self.label
                    {
                        // Reduced match obligation
                        for (index, t) in get_data_arguments(&mo.pattern.borrow()).enumerate() {
                            assert!(index < arity, "This pattern associates function symbol {:?} with different arities {} and {}", symbol, index+1, arity);

                            if !is_data_variable(t.borrow()) {
                                let mut new_pos = mo.position.clone();
                                new_pos.indices.push(index + 2);
                                new_obligations.push(MatchObligation {
                                    pattern: t.protect(),
                                    position: new_pos,
                                });
                            }
                        }
                    } else {
                        // remains unchanged
                        new_obligations.push(mo.clone());
                    }
                }

                new_obligations
                    .sort_unstable_by(|mo1, mo2| mo1.position.len().cmp(&mo2.position.len()));
                mg.obligations = new_obligations;
                mg.announcement.symbols_seen += 1;

                result.reduced.push(mg);
            }
        }

        trace!(
            "compute_derivative(symbol {}, label {})",
            symbol, self.label
        );
        trace!("Match goals: {{");
        for mg in &self.match_goals {
            debug!("\t {}", mg);
        }

        trace!("}}");
        trace!("Completed: {{");
        for mg in &result.completed {
            debug!("\t {}", mg);
        }

        trace!("}}");
        trace!("Unchanged: {{");
        for mg in &result.unchanged {
            debug!("\t {}", mg);
        }

        trace!("}}");
        trace!("Reduced: {{");
        for mg in &result.reduced {
            trace!("\t {}", mg);
        }
        trace!("}}");

        result
    }

    /// Create a state from a set of match goals
    fn new(goals: Vec<MatchGoal>, num_transitions: usize) -> State {
        // The label of the state is taken from a match obligation of a root match goal.
        let mut label: Option<ExplicitPosition> = None;
        // Go through all match goals until a root match goal is found
        for goal in &goals {
            if goal.announcement.position == ExplicitPosition::empty_pos() {
                // Find the shortest match obligation position.
                // This design decision was taken as it presumably has two advantages.
                // 1. Patterns that overlap will be more quickly distinguished, potentially decreasing
                // the size of the automaton.
                // 2. The average lookahead may be shorter.
                if label.is_none() {
                    label = Some(goal.obligations.first().unwrap().position.clone());
                }

                for obligation in &goal.obligations {
                    if let Some(l) = &label {
                        if &obligation.position < l {
                            label = Some(obligation.position.clone());
                        }
                    }
                }
            }
        }

        State {
            label: label.unwrap(),
            transitions: Vec::with_capacity(num_transitions), // Transitions need to be added later
            match_goals: goals,
        }
    }
}

#[cfg(test)]
mod tests {
    //use mcrl2::data::DataSpecification;

    //use super::SetAutomaton;

    // Creating this automaton takes too much time.
    /*#[test]
    fn test_automaton_from_data_spec() {
        let data_spec_text = include_str!("../../../benchmarks/cases/add16.dataspec");
        let data_spec = DataSpecification::new(data_spec_text);

        let _ = SetAutomaton::new(&data_spec.into(), false, false);
    }*/
}
