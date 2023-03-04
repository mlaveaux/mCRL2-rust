use std::collections::VecDeque;

use mcrl2_rust::atermpp::TermPool;
use crate::{utilities::{get_position, ExplicitPosition, SemiCompressedTermTree, create_var_map}, rewrite_specification::Rule};
use mcrl2_rust::{atermpp::ATerm};

/// An equivalence class is a variable with (multiple) positions.
/// This is necessary for non-linear patterns.
/// It is used by EnhancedMatchAnnouncement to store what positions need to be compared.
///
/// TODO: there is probably a better term than "equivalence class"
///
/// # Example
/// Suppose we have a pattern f(x,x), where x is a variable.
/// Then it will have one equivalence class storing "x" and the positions 1 and 2.
/// The function equivalences_hold checks whether the term has the same term on those positions.
/// For example, it will returns false on the term f(a, b) and true on the term f(a, a).
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct EquivalenceClass 
{
    variable: ATerm,
    positions: Vec<ExplicitPosition>
}

impl EquivalenceClass 
{
    pub fn equivalences_hold(term: &ATerm, eqs: &Vec<EquivalenceClass>) -> bool 
    {
        eqs.iter().all(|ec| {
            ec.positions.len() < 2 || {
                let mut iter_pos = ec.positions.iter();
                let first = iter_pos.next().unwrap();
                iter_pos.all(|other_pos| {get_position(term, first) == get_position(term, other_pos) })}
        })
    }
}

/// A struct announcing that a match has been made
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct MatchAnnouncement 
{
    pub rule: Rule,
    pub position: ExplicitPosition,
    pub symbols_seen: usize
}

/// Adds the position of a variable to the equivalence classes
fn update_equivalences(ve: &mut Vec<EquivalenceClass>, variable: &ATerm, pos: ExplicitPosition) 
{
    // Check if the variable was seen before
    if ve.iter().any(|ec| { &ec.variable == variable }) 
    {
        for ec in ve.iter_mut() 
        {
            //Find the equivalence class and add the position
            if &ec.variable == variable && !ec.positions.iter().any(|x| { x == &pos }) 
            {
                ec.positions.push(pos.clone());
            }
        }
    } 
    else 
    {
        // If the variable was not found at another position add a new equivalence class
        ve.push(EquivalenceClass { variable: variable.clone(), positions: vec![pos] });
    }
}

/// A condition for an enhanced match announcement.
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct EMACondition 
{
    /// Conditions lhs and rhs are stored in the term pool as much as possible with a SemiCompressedTermTree
    pub semi_compressed_lhs: SemiCompressedTermTree,
    pub semi_compressed_rhs: SemiCompressedTermTree,
    pub equality: bool //whether the lhs and rhs should be equal or different
}

/// An EnhancedMatchAnnouncement is used on transitions. Besides the normal MatchAnnouncement
/// it stores additional information.
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct EnhancedMatchAnnouncement 
{
    pub announcement: MatchAnnouncement,
    /// Positions in the pattern with the same variable, for non-linear patterns
    pub equivalence_classes: Vec<EquivalenceClass>,
    /// Right hand side is stored in the term pool as much as possible with a SemiCompressedTermTree
    pub semi_compressed_rhs: SemiCompressedTermTree,
    pub conditions: Vec<EMACondition>,
    /// Whether the rewrite rule duplicates subterms, e.g. times(s(x), y) = plus(y, times(x, y))
    pub is_duplicating: bool,
}


impl MatchAnnouncement 
{
    /// Derives the positions in a pattern with same variable (for non-linear patters)
    fn derive_equivalence_classes(&self, tp: &TermPool) -> Vec<EquivalenceClass> 
    {
        // A queue is used to keep track of the positions we still need to visit in the pattern
        let mut queue = VecDeque::new();
        queue.push_back(ExplicitPosition::empty_pos()); //push the root position in the queue
        let mut var_equivalences = vec![];

        while !queue.is_empty() 
        {
            // Select a position to inspect
            let pos = queue.pop_front().unwrap();
            let term = get_position(&self.rule.lhs, &pos);

            // The symbol "ω" was used early in development to indicate an abstract variable, not used in REC
            // We need to discard this option because it is not a concrete variable whose position we must match
            if term.get_head_symbol().name() != "ω" 
            {
                // If arity_per_symbol does not contain the head symbol it is a variable
                if term.is_variable() 
                {
                    // Register the position of the variable
                    update_equivalences(&mut var_equivalences, &term, pos);
                } 
                else 
                {
                    // Put all subterms in the queue for exploration
                    for i in 1 .. term.arguments().len() + 1 
                    {
                        let mut sub_term_pos = pos.clone();
                        sub_term_pos.indices.push(i);
                        queue.push_back(sub_term_pos);
                    }
                }
            }
        }

        // Discard variables that only occur once
        var_equivalences.retain(|x| {x.positions.len() > 1});
        var_equivalences
    }

    /// For a match announcement derives an EnhancedMatchAnnouncement, which precompiles some information
    /// for faster rewriting.
    fn derive_redex(&self, tp: &TermPool) -> EnhancedMatchAnnouncement 
    {
        // Create a mapping of where the variables are and derive SemiCompressedTermTrees for the
        // rhs of the rewrite rule and for lhs and rhs of each condition.
        // Also see the documentation of SemiCompressedTermTree
        let var_map = create_var_map(&self.rule.lhs);
        let sctt_rhs = SemiCompressedTermTree::from_term(self.rule.rhs.clone(), &var_map);
        let mut conditions = vec![];

        for c in &self.rule.conditions 
        {
            let ema_condition = EMACondition {
                semi_compressed_lhs: SemiCompressedTermTree::from_term(c.lhs.clone(), &var_map),
                semi_compressed_rhs: SemiCompressedTermTree::from_term(c.rhs.clone(), &var_map),
                equality: c.equality
            };
            conditions.push(ema_condition);
        }

        let is_duplicating = sctt_rhs.contains_duplicate_var_references();

        EnhancedMatchAnnouncement {
            announcement: self.clone(),
            equivalence_classes: self.derive_equivalence_classes(tp),
            semi_compressed_rhs: sctt_rhs,
            conditions,
            is_duplicating,
        }
    }
}

/* 
impl MatchGoal {
    /// Derive the greatest common prefix (gcp) of the announcement and obligation positions
    /// of a list of match goals.
    fn greatest_common_prefix(goals: &Vec<MatchGoal>) -> ExplicitPosition {
        //gcp is empty if there are not match goals
        if goals.is_empty() {
            return ExplicitPosition::empty_pos();
        }

        //Initialise the prefix with the first match goal, can only shrink afterwards
        let first_match_pos = &goals.get(0).unwrap().announcement.position;
        let mut gcp_length = first_match_pos.len();
        let prefix = &first_match_pos.clone();

        for g in goals {
            //Compare up to gcp_length or the length of the announcement position
            let compare_length = min(gcp_length,g.announcement.position.len());
            //gcp_length shrinks if they are not the same up to compare_length
            gcp_length = MatchGoal::common_prefix_length(&prefix.indices[0..compare_length], &g.announcement.position.indices[0..compare_length]);
            for mo in &g.obligations {
                //Compare up to gcp_length or the length of the match obligation position
                let compare_length = min(gcp_length,mo.position.len());
                //gcp_length shrinks if they are not the same up to compare_length
                gcp_length = MatchGoal::common_prefix_length(&prefix.indices[0..compare_length], &mo.position.indices[0..compare_length]);
            }
        }
        //The gcp is constructed by taking the first gcp_length indices of the first match goal prefix
        let greatest_common_prefix = SmallVec::from_slice(&prefix.indices[0..gcp_length]);
        ExplicitPosition {indices: greatest_common_prefix}
    }

    //Assumes two slices are of the same length and computes to what length they are equal
    fn common_prefix_length(pos1: &[usize], pos2: &[usize]) -> usize {
        if pos1.len() != pos2.len() {panic!("Given arrays should be of the same length.")}
        let mut common_length = 0;
        for i in 0..pos1.len() {
            if pos1.get(i).unwrap() == pos2.get(i).unwrap() {
                common_length += 1;
            } else {break;}
        }
        common_length
    }

    /// Removes the first len position indices of the match goal and obligation positions
    fn remove_prefix(mut goals: Vec<MatchGoal>, len: usize) -> Vec<MatchGoal> {
        for goal in &mut goals {
            //update match announcement
            goal.announcement.position = ExplicitPosition {indices: SmallVec::from_slice(&goal.announcement.position.indices[len..])};
            for mo_index  in 0..goal.obligations.len() {
                let shortened = ExplicitPosition {indices: SmallVec::from_slice(&goal.obligations.get(mo_index).unwrap().position.indices[len..])};
                goal.obligations.get_mut(mo_index).unwrap().position = shortened;
            }
        }
        goals
    }

    /// Checks for two positions whether one is a subposition of the other.
    /// For example 2.2.3 and 2 are comparable. 2.2.3 and 1 are not.
    fn pos_comparable(p1: &ExplicitPosition, p2: &ExplicitPosition) -> bool {
        let mut index = 0;
        loop {
            if p1.len() == index || p2.len() == index {
                return true;
            }
            if p1.indices[index] != p2.indices[index] {
                return false;
            }
            index += 1;
        }
    }

    /// Partition a set of match goals (a transition is split into different states).
    /// There are multiple options for partitioning.
    /// What is now implemented is that goals are related if there match announcement positions
    /// are comparable (they are the same or one is higher), checked using pos_comparable.
    ///
    /// Returns a Vec where each element is a partition containing the goals and the positions.
    fn partition(goals: Vec<MatchGoal>) -> Vec<(Vec<MatchGoal>,Vec<ExplicitPosition>)> {
        let mut partitions = vec![];

        //If one of the goals has a root position all goals are related.
        if goals.iter().any(|g| {g.announcement.position.len() == 0}) {
            let mut all_positions = Vec::new();
            for g in &goals{
                if !all_positions.contains(&g.announcement.position) {
                    all_positions.push(g.announcement.position.clone())
                }
            }
            partitions.push((goals,all_positions));
            return partitions;
        }

        //Create a mapping from positions to goals, goals are represented with an index
        //on function parameter goals
        let mut position_to_goals = HashMap::new();
        let mut all_positions = Vec::new();
        for (i,g) in goals.iter().enumerate() {
            if !all_positions.contains(&g.announcement.position) {
                all_positions.push(g.announcement.position.clone())
            }
            if !position_to_goals.contains_key(&g.announcement.position) {
                position_to_goals.insert(g.announcement.position.clone(), vec![i]);
            } else {
                let vec = position_to_goals.get_mut(&g.announcement.position).unwrap();
                vec.push(i);
            }
        }
        //Sort the positions. They are now in depth first order.
        all_positions.sort_unstable();

        //compute the partitions, finished when all positions are processed
        let mut p_index = 0; //position index
        while p_index < all_positions.len() {
            //Start the partition with a position
            let p = &all_positions[p_index];
            let mut pos_in_partition = vec![];
            pos_in_partition.push(p.clone());
            let mut goals_in_partition = vec![];

            //put the goals with position p in the partition
            let g = position_to_goals.get(&p).unwrap();
            for i in g {
                goals_in_partition.push(goals[*i].clone());
            }

            //Go over the positions until we find a position that is not comparable to p
            //Because all_positions is sorted we know that once we find a position that is not comparable
            //all subsequent positions will also not be comparable.
            //Moreover, all positions in the partition are related to p. p is the highest in the partition.
            p_index += 1;
            while p_index < all_positions.len() && MatchGoal::pos_comparable(p, &all_positions[p_index]) {
                pos_in_partition.push(all_positions[p_index].clone());
                //Put the goals with position all_positions[p_index] in the partition
                let g = position_to_goals.get(&all_positions[p_index]).unwrap();
                for i in g {
                    goals_in_partition.push(goals[*i].clone());
                }
                p_index += 1;
            }
            partitions.push((goals_in_partition,pos_in_partition));
        }
        partitions
    }
}

impl State {
    /* Derive transitions from a state given a head symbol. The resulting transition is returned as a tuple
    The tuple consists of a vector of outputs and a set of destinations (which are sets of match goals).
    We don't use the struct Transition as it requires that the destination is a full state, with name.
    Since we don't yet know whether the state already exists we just return a set of match goals as 'state'.

    Parameter symbol is the symbol for which the transition is computed
     */
    fn derive_transition(&self, symbol: Symbol, rewrite_rules: &Vec<RewriteRule>, tp: &TermPool, arity_per_symbol: &HashMap<Symbol,usize>, apma:bool)
                         -> (Vec<MatchAnnouncement>, Vec<(ExplicitPosition, GoalsOrInitial)>) {
        //Computes the derivative containing the goals that are completed, unchanged and reduced
        let mut derivative = self.compute_derivative(symbol, tp, arity_per_symbol);
        //The outputs/matching patterns of the transitions are those who are completed
        let outputs = derivative.completed.into_iter().map(|x| {x.announcement}).collect();
        let mut new_match_goals = derivative.unchanged;
        new_match_goals.append(&mut derivative.reduced);

        let mut destinations = vec![];
        // If we are building an APMA we do not deepen the position or create a hypertransitions
        // with multiple endpoints
        if apma {
            if !new_match_goals.is_empty() {
                destinations.push((ExplicitPosition::empty_pos(),GoalsOrInitial::Goals(new_match_goals)));
            }
        } else {
            //In case we are building a set automaton we partition the match goals
            let partitioned = MatchGoal::partition(new_match_goals);

            //Get the greatest common prefix and shorten the positions
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

            //Handle fresh match goals, they are the positions Label(state).i
            //where i is between 1 and the arity of the function symbol of the transition
            for i in 1..(arity_per_symbol.get(&symbol).unwrap().clone() + 1) {
                let mut pos = self.label.clone();
                pos.indices.push(i);

                //Check if the fresh goals are related to one of the existing partitions
                let mut partition_key = None;
                'outer: for (i,part_pos) in positions_per_partition.iter().enumerate() {
                    for p in part_pos {
                        if MatchGoal::pos_comparable(p, &pos) {
                            partition_key = Some(i);
                            break 'outer;
                        }
                    }
                }
                if let Some(key) = partition_key {//if the fresh goals fall in an existing partition
                    let gcp_length = gcp_length_per_partition[key];
                    let pos = ExplicitPosition { indices: SmallVec::from_slice(&pos.indices[gcp_length..]) };
                    //Add the fresh goals to the partition
                    for rr in rewrite_rules {
                        if let GoalsOrInitial::Goals(goals) = &mut destinations[key].1 {
                            goals.push(MatchGoal {
                                obligations: vec![MatchObligation { pattern: rr.lhs.clone(), position: pos.clone() }],
                                announcement: MatchAnnouncement { rule: (*rr).clone(), position: pos.clone(), symbols_seen: 0 }
                            });
                        }
                    }
                } else { //the transition is simply to the initial state
                    //GoalsOrInitial::InitialState avoids unnecessary work of creating all these fresh goals
                    destinations.push((pos, GoalsOrInitial::InitialState));
                }
            }
        }
        //Sort so that transitions that do not deepen the position are listed first
        destinations.sort_unstable_by(|x1, x2| {x1.0.cmp(&x2.0)});
        (outputs, destinations)
    }

    /// For a transition 'symbol' of state 'self' this function computes which match goals are
    /// completed, unchanged and reduced.
    fn compute_derivative(&self, symbol: Symbol, tp: &TermPool, arity_per_symbol: &HashMap<Symbol,usize>) -> Derivative {
        let mut result = Derivative {
            completed: vec![],
            unchanged: vec![],
            reduced: vec![]
        };
        for mg in &self.match_goals {
            //Completed match goals
            if mg.obligations.len() == 1 && mg.obligations.iter()
                .any(|mo| {mo.position == self.label && mo.pattern.get_head_symbol() == symbol
                && mo.pattern.get_subterms().iter().all(|x| {!arity_per_symbol.contains_key(&x.get_head_symbol())})}) {
                result.completed.push(mg.clone());
            } else if mg.obligations.iter().any(|mo| {mo.position == self.label && mo.pattern.get_head_symbol() != symbol}) {
                //discard
            //Unchanged match goals
            } else if !mg.obligations.iter().any(|mo| {mo.position == self.label}) {
                let mut mg = mg.clone();
                if mg.announcement.rule.lhs != mg.obligations.first().unwrap().pattern {
                    mg.announcement.symbols_seen += 1;
                }
                result.unchanged.push(mg);
            //Reduced match obligations
            } else if mg.obligations.iter().any(|mo| {mo.position == self.label && mo.pattern.get_head_symbol() == symbol }) {
                let mut mg = mg.clone();
                //reduce obligations
                let mut new_obligations = vec![];
                for mo in mg.obligations {
                    if mo.pattern.get_head_symbol() == symbol && mo.position == self.label {
                        //reduce
                        let mut index = 1;
                        for t in mo.pattern.get_subterms() {
                            if tp.get_head_symbol_string(t) != "ω" {
                                if arity_per_symbol.contains_key(&t.get_head_symbol()) {
                                    let mut new_pos = mo.position.clone();
                                    new_pos.indices.push(index);
                                    new_obligations.push(MatchObligation {
                                        pattern: t.clone(),
                                        position: new_pos
                                    });
                                } else { //variable
                                }
                                index += 1;
                            }
                        }
                    } else {
                        //remains unchanged
                        new_obligations.push(mo.clone());
                    }
                }
                new_obligations.sort_unstable_by(|mo1, mo2| {mo1.position.len().cmp(&mo2.position.len())});
                mg.obligations = new_obligations;
                mg.announcement.symbols_seen += 1;
                result.reduced.push(mg);
            } else {
                println!("{:?}",mg);
            }
        }
        result
    }

    /// Create a state from a set of match goals
    fn new(goals: Vec<MatchGoal>, num_transitions: usize) -> State {
        //The label of the state is taken from a match obligation of a root match goal.
        let mut label : Option<ExplicitPosition>= None;
        //Go through all match goals...
        for g in &goals {
            //...until a root match goal is found
            if g.announcement.position == ExplicitPosition::empty_pos() {
                /*
                Find the shortest match obligation position.
                This design decision was taken as it presumably has two advantages.
                1. Patterns that overlap will be more quickly distinguished, potentially decreasing
                the size of the automaton.
                2. The average lookahead may be shorter.
                 */
                if label.is_none() {
                    label = Some(g.obligations.first().unwrap().position.clone());
                }
                for o in &g.obligations {
                    if let Some(l) = &label {
                        if &o.position < &l {
                            label = Some(o.position.clone());
                        }
                    }
                }
            }
        }
        State{
            label: label.unwrap(),
            transitions: Vec::with_capacity(num_transitions), //transitions need to be added later
            match_goals: goals
        }
    }
}

impl SetAutomaton {
    /// Construct a set automaton. If 'apma' is true construct an APMA instead.
    /// An APMA is just a set automaton that does not partition the match goals on a transition
    /// and does not add fresh goals.
    pub(crate) fn construct(spec: RewriteSpec, apma: bool) -> SetAutomaton {
        //Create the initial state
        let mut initial_state = State {
            label: ExplicitPosition::empty_pos(),
            transitions: Vec::with_capacity(spec.symbols.len()),
            match_goals: vec![]
        };
        //States are labelled s0, s1, s2, etcetera. state_counter keeps track of count.
        let mut state_counter:usize = 1;

        //Initiate the maximally shared term storage
        let mut term_pool = TermPool::new();
        let mut symbols = Vec::new();
        let mut arity_per_symbol= HashMap::new();

        //Insert the symbols in the term pool, important that this is done first.
        //A Symbol is just a usize. By inserting them first we ensure that if there are n symbols
        //then the first n Symbol indexes that are issued are between 0 and n.
        //We can now use a Symbol as an index for the transitions of a state.
        for symbol in spec.symbols {
            let s:Symbol = term_pool.get_or_insert_symbol_index(symbol.clone());
            symbols.push(s);
            arity_per_symbol.insert(s,spec.arity_per_symbol.get(&symbol).unwrap().clone());
        }

        //Store the rewrite rules in the maximally shared term storage
        let mut rewrite_rules = Vec::new();
        for rr in spec.rewrite_rules {
            let lhs = term_pool.construct_term_from_scratch(rr.lhs);
            let rhs  = term_pool.construct_term_from_scratch(rr.rhs);
            let mut conditions = vec![];
            for c in rr.conditions {
                let lhs_cond = term_pool.construct_term_from_scratch(c.lhs);
                let rhs_cond = term_pool.construct_term_from_scratch(c.rhs);
                let condition = Condition{
                    lhs: lhs_cond,
                    rhs: rhs_cond,
                    equality: c.equality
                };
                conditions.push(condition);
            }
            rewrite_rules.push(RewriteRule{lhs,rhs, conditions });
        }


        /* The initial state has a match goals for each pattern. For each pattern l there is a match goal
        with one obligation l@ε and announcement l@ε. */
        for rr in &rewrite_rules {
            initial_state.match_goals.push(MatchGoal{
                obligations: vec![MatchObligation{pattern: rr.lhs.clone(), position: ExplicitPosition::empty_pos()}],
                announcement: MatchAnnouncement {rule: (*rr).clone(), position: ExplicitPosition::empty_pos(), symbols_seen: 0}
            });
        }
        /* Match goals need to be sorted so that we can easily check whether a state with a certain
        set of match goals already exists.*/
        initial_state.match_goals.sort();

        //HashMap from goals to state number
        let mut map_goals_state:HashMap<Vec<MatchGoal>,usize,RandomState> = HashMap::default();

        //Queue of states that still need to be explored
        let mut queue = VecDeque::new();
        queue.push_back(0);
        let mut states = vec![];
        map_goals_state.insert(initial_state.match_goals.clone(),0);
        states.push(initial_state);

        while !queue.is_empty() {
            //Pick a state to explore
            let s_index = queue.pop_front().unwrap();

            //Compute the transitions from the state in parallel using rayon
            let transitions_per_symbol: Vec<_> = symbols.par_iter().map(|s| {(s.clone(),states.get(s_index).unwrap()
                .derive_transition(s.clone(), &rewrite_rules, &term_pool, &arity_per_symbol, apma))}).collect();
            //Loop over all the possible symbols and the associated hypertransition
            for (symbol, (outputs,destinations)) in transitions_per_symbol {
                //Associate an EnhancedMatchAnnouncement to every transition
                let mut announcements:SmallVec<[EnhancedMatchAnnouncement;1]> = outputs.into_iter().map(|x| {x.derive_redex(&term_pool, &arity_per_symbol)}).collect();
                //announcements.sort_by(|ema1,ema2| {ema2.announcement.rule.rhs.get_subterms().len().cmp(&ema1.announcement.rule.rhs.get_subterms().len())});
                announcements.sort_by(|ema1,ema2| {ema1.announcement.position.cmp(&ema2.announcement.position)});
                //Create transition
                let mut transition = Transition {
                    symbol: symbol.clone(),
                    announcements,
                    destinations: smallvec![]
                };
                //For the destinations we convert the match goal destinations to states
                let mut dest_states = smallvec![];
                //Loop over the hypertransitions
                for (pos,goals_or_initial) in destinations {
                    /* Match goals need to be sorted so that we can easily check whether a state with a certain
                    set of match goals already exists.*/
                    if let GoalsOrInitial::Goals(goals) = goals_or_initial {
                        if map_goals_state.contains_key(&goals) {//The destination state already exists
                            dest_states.push((pos,map_goals_state.get(&goals).unwrap().clone()))
                        } else if !goals.is_empty() {
                            //The destination state does not yet exist, create it
                            let new_state = State::new(goals.clone(), symbols.len());
                            states.push(new_state);
                            dest_states.push((pos, state_counter));
                            map_goals_state.insert(goals,state_counter);
                            queue.push_back(state_counter);
                            state_counter += 1;
                        }
                    } else { //The transition is to the initial state
                        dest_states.push((pos,0));
                    }
                }
                transition.destinations = dest_states;
                states.get_mut(s_index).unwrap().transitions.push(transition);
            }
        }

        term_pool.mark_all_terms_as_permanent();
        SetAutomaton{
            states,
            term_pool
        }
    }
}
*/