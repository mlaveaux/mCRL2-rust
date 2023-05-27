use std::{collections::VecDeque};

use ahash::HashMap;
use mcrl2_rust::atermpp::{ATerm, TermFunctionSymbol};
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

#[derive(Debug, Clone)]
pub struct Transition {
    pub(crate) symbol: TermFunctionSymbol,
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

impl SetAutomaton {
    pub(crate) fn construct(spec: RewriteSpecification) -> SetAutomaton {
        // States are labelled s0, s1, s2, etcetera. state_counter keeps track of count.
        let mut state_counter: usize = 1;

        // The initial state has a match goals for each pattern. For each pattern l there is a match goal
        // with one obligation l@ε and announcement l@ε.
        let mut match_goals = Vec::<MatchGoal>::new();
        for rr in &spec.rewrite_rules {
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
            transitions: Vec::with_capacity(spec.symbols.len()),
            match_goals: match_goals.clone(),
        };

        // HashMap from goals to state number
        let mut map_goals_state = HashMap::default();

        //Queue of states that still need to be explored
        let mut queue = VecDeque::new();
        queue.push_back(0);

        map_goals_state.insert(match_goals, 0);

        let mut states = vec![initial_state];

        while !queue.is_empty() {
            // Pick a state to explore
            let s_index = queue.pop_front().unwrap();

            // Compute the transitions from the states
            let transitions_per_symbol: Vec<_> = spec
                .symbols
                .iter()
                .map(|s| {
                    (
                        s.clone(),
                        states.get(s_index).unwrap().derive_transition(
                            s.clone(),
                            1,
                            &spec.rewrite_rules,
                            false,
                        ),
                    )
                })
                .collect();

            println!("{:?}", &transitions_per_symbol);

            // Loop over all the possible symbols and the associated hypertransition
            for (symbol, (outputs, destinations)) in transitions_per_symbol {
                // Associate an EnhancedMatchAnnouncement to every transition
                let mut announcements: SmallVec<[EnhancedMatchAnnouncement; 1]> = outputs
                    .into_iter()
                    .map(|x| x.derive_redex())
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
                            let new_state = State::new(goals.clone(), spec.symbols.len());
                            states.push(new_state);
                            dest_states.push((pos, state_counter));
                            map_goals_state.insert(goals, state_counter);
                            queue.push_back(state_counter);
                            state_counter += 1;
                        }
                    } else {
                        //The transition is to the initial state
                        dest_states.push((pos, 0));
                    }
                }
                transition.destinations = dest_states;
                states
                    .get_mut(s_index)
                    .unwrap()
                    .transitions
                    .push(transition);
            }
        }

        SetAutomaton { states }
    }
}

#[derive(Debug)]
pub struct Derivative {
    completed: Vec<MatchGoal>,
    unchanged: Vec<MatchGoal>,
    reduced: Vec<MatchGoal>,
}

#[derive(Debug)]
pub(crate) struct State {
    pub(crate) label: ExplicitPosition,

    /// Note that transitions are indexed by the index given by the OpId from a function symbol.
    pub(crate) transitions: Vec<Transition>,
    pub(crate) match_goals: Vec<MatchGoal>,
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
        symbol: TermFunctionSymbol,
        arity: usize,
        rewrite_rules: &Vec<Rule>,
        apma: bool,
    ) -> (
        Vec<MatchAnnouncement>,
        Vec<(ExplicitPosition, GoalsOrInitial)>,
    ) {
        // Computes the derivative containing the goals that are completed, unchanged and reduced
        let mut derivative = self.compute_derivative(&symbol);

        // The outputs/matching patterns of the transitions are those who are completed
        let outputs = derivative
            .completed
            .into_iter()
            .map(|x| x.announcement)
            .collect();
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
            // where i is between 1 and the arity of the function symbol of the transition
            for i in 1..arity + 1 {
                let mut pos = self.label.clone();
                pos.indices.push(i);

                //Check if the fresh goals are related to one of the existing partitions
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
                    //the transition is simply to the initial state
                    //GoalsOrInitial::InitialState avoids unnecessary work of creating all these fresh goals
                    destinations.push((pos, GoalsOrInitial::InitialState));
                }
            }
        }
        //Sort so that transitions that do not deepen the position are listed first
        destinations.sort_unstable_by(|x1, x2| x1.0.cmp(&x2.0));
        (outputs, destinations)
    }

    /// For a transition 'symbol' of state 'self' this function computes which match goals are
    /// completed, unchanged and reduced.
    fn compute_derivative(&self, symbol: &TermFunctionSymbol) -> Derivative {
        let mut result = Derivative {
            completed: vec![],
            unchanged: vec![],
            reduced: vec![],
        };

        // For DataAppl the first argument is the function symbol (make this explicit).
        let term_symbol = <TermFunctionSymbol as Into<ATerm>>::into(symbol.clone());

        for mg in &self.match_goals {
            // Completed match goals
            if mg.obligations.len() == 1
                && mg.obligations.iter().any(|mo| {
                    mo.position == self.label
                        && mo.pattern.arg(0) == term_symbol
                        && mo.pattern.arguments().iter().all(|x| !x.is_variable())
                })
            {
                result.completed.push(mg.clone());
            } else if mg
                .obligations
                .iter()
                .any(|mo| mo.position == self.label && mo.pattern.arg(0) != term_symbol)
            {
                // discard
            } else if !mg.obligations.iter().any(|mo| mo.position == self.label) {
                // Unchanged match goals
                let mut mg = mg.clone();
                if mg.announcement.rule.lhs != mg.obligations.first().unwrap().pattern {
                    mg.announcement.symbols_seen += 1;
                }
                result.unchanged.push(mg);
            } else if mg
                .obligations
                .iter()
                .any(|mo| mo.position == self.label && mo.pattern.arg(0) == term_symbol)
            {
                // Reduced match obligations
                let mut mg = mg.clone();
                let mut new_obligations = vec![];
                for mo in mg.obligations {
                    if mo.pattern.arg(0) == term_symbol && mo.position == self.label {
                        // reduce
                        let mut index = 1;
                        for t in mo.pattern.arguments() {
                            if t.arg(0).arg(0).get_head_symbol().name() != "ω" {
                                if t.is_variable() {
                                    let mut new_pos = mo.position.clone();
                                    new_pos.indices.push(index);
                                    new_obligations.push(MatchObligation {
                                        pattern: t.clone(),
                                        position: new_pos,
                                    });
                                } else { //variable
                                }
                                index += 1;
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
            } else {
                println!("{:?}", mg);
            }
        }
        result
    }

    /// Create a state from a set of match goals
    fn new(goals: Vec<MatchGoal>, num_transitions: usize) -> State {
        // The label of the state is taken from a match obligation of a root match goal.
        let mut label: Option<ExplicitPosition> = None;
        // Go through all match goals...
        for g in &goals {
            // ...until a root match goal is found
            if g.announcement.position == ExplicitPosition::empty_pos() {
                // Find the shortest match obligation position.
                // This design decision was taken as it presumably has two advantages.
                // 1. Patterns that overlap will be more quickly distinguished, potentially decreasing
                // the size of the automaton.
                // 2. The average lookahead may be shorter.
                if label.is_none() {
                    label = Some(g.obligations.first().unwrap().position.clone());
                }
                for o in &g.obligations {
                    if let Some(l) = &label {
                        if &o.position < l {
                            label = Some(o.position.clone());
                        }
                    }
                }
            }
        }
        State {
            label: label.unwrap(),
            transitions: Vec::with_capacity(num_transitions), //transitions need to be added later
            match_goals: goals,
        }
    }
}
