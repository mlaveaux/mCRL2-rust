use std::{cmp::min, collections::VecDeque};

use crate::{
    rewrite_specification::Rule,
    utilities::{create_var_map, get_position, ExplicitPosition, SemiCompressedTermTree},
};
use ahash::{HashMap, HashMapExt};
use mcrl2_rust::atermpp::ATerm;
use smallvec::SmallVec;

use super::MatchObligation;

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
pub struct EquivalenceClass {
    pub(crate) variable: ATerm,
    pub(crate) positions: Vec<ExplicitPosition>,
}

impl EquivalenceClass {
    pub fn equivalences_hold(term: &ATerm, eqs: &[EquivalenceClass]) -> bool {
        eqs.iter().all(|ec| {
            ec.positions.len() < 2 || {
                let mut iter_pos = ec.positions.iter();
                let first = iter_pos.next().unwrap();
                iter_pos.all(|other_pos| get_position(term, first) == get_position(term, other_pos))
            }
        })
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct MatchAnnouncement {
    pub rule: Rule,
    pub position: ExplicitPosition,
    pub symbols_seen: usize,
}

/// A condition for an enhanced match announcement.
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct EMACondition {
    /// Conditions lhs and rhs are stored in the term pool as much as possible with a SemiCompressedTermTree
    pub semi_compressed_lhs: SemiCompressedTermTree,
    pub semi_compressed_rhs: SemiCompressedTermTree,
    /// whether the lhs and rhs should be equal or different
    pub equality: bool,
}

/// An EnhancedMatchAnnouncement is used on transitions. Besides the normal MatchAnnouncement
/// it stores additional information.
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct EnhancedMatchAnnouncement {
    pub(crate) announcement: MatchAnnouncement,
    /// Positions in the pattern with the same variable, for non-linear patterns
    pub equivalence_classes: Vec<EquivalenceClass>,
    /// Right hand side is stored in the term pool as much as possible with a SemiCompressedTermTree
    pub semi_compressed_rhs: SemiCompressedTermTree,
    pub conditions: Vec<EMACondition>,
    /// Whether the rewrite rule duplicates subterms, e.g. times(s(x), y) = plus(y, times(x, y))
    pub is_duplicating: bool,
}

impl MatchAnnouncement {
    /// Derives the positions in a pattern with same variable (for non-linear patters)
    pub fn derive_equivalence_classes(&self) -> Vec<EquivalenceClass> {
        // A queue is used to keep track of the positions we still need to visit in the pattern
        let mut queue = VecDeque::new();
        queue.push_back(ExplicitPosition::empty_pos()); //push the root position in the queue
        let mut var_equivalences = vec![];

        while !queue.is_empty() {
            // Select a position to inspect
            let pos = queue.pop_front().unwrap();
            let term = get_position(&self.rule.lhs, &pos);

            // If arity_per_symbol does not contain the head symbol it is a variable
            if term.is_variable() {
                // Register the position of the variable
                update_equivalences(&mut var_equivalences, &term, pos);
            } else {
                // Put all subterms in the queue for exploration
                for i in 1..term.arguments().len() + 1 {
                    let mut sub_term_pos = pos.clone();
                    sub_term_pos.indices.push(i);
                    queue.push_back(sub_term_pos);
                }
            }
        }

        // Discard variables that only occur once
        var_equivalences.retain(|x| x.positions.len() > 1);
        var_equivalences
    }

    /// For a match announcement derives an EnhancedMatchAnnouncement, which precompiles some information
    /// for faster rewriting.
    pub fn derive_redex(&self) -> EnhancedMatchAnnouncement {
        // Create a mapping of where the variables are and derive SemiCompressedTermTrees for the
        // rhs of the rewrite rule and for lhs and rhs of each condition.
        // Also see the documentation of SemiCompressedTermTree
        let var_map = create_var_map(&self.rule.lhs);
        let sctt_rhs = SemiCompressedTermTree::from_term(self.rule.rhs.clone(), &var_map);
        let mut conditions = vec![];

        for c in &self.rule.conditions {
            let ema_condition = EMACondition {
                semi_compressed_lhs: SemiCompressedTermTree::from_term(c.lhs.clone(), &var_map),
                semi_compressed_rhs: SemiCompressedTermTree::from_term(c.rhs.clone(), &var_map),
                equality: c.equality,
            };
            conditions.push(ema_condition);
        }

        let is_duplicating = sctt_rhs.contains_duplicate_var_references();

        EnhancedMatchAnnouncement {
            announcement: self.clone(),
            equivalence_classes: self.derive_equivalence_classes(),
            semi_compressed_rhs: sctt_rhs,
            conditions,
            is_duplicating,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct MatchGoal {
    pub obligations: Vec<MatchObligation>,
    pub announcement: MatchAnnouncement,
}

impl MatchGoal {
    /// Derive the greatest common prefix (gcp) of the announcement and obligation positions
    /// of a list of match goals.
    pub fn greatest_common_prefix(goals: &Vec<MatchGoal>) -> ExplicitPosition {
        // gcp is empty if there are not match goals
        if goals.is_empty() {
            return ExplicitPosition::empty_pos();
        }

        // Initialise the prefix with the first match goal, can only shrink afterwards
        let first_match_pos = &goals.get(0).unwrap().announcement.position;
        let mut gcp_length = first_match_pos.len();
        let prefix = &first_match_pos.clone();

        for g in goals {
            // Compare up to gcp_length or the length of the announcement position
            let compare_length = min(gcp_length, g.announcement.position.len());
            // gcp_length shrinks if they are not the same up to compare_length
            gcp_length = MatchGoal::common_prefix_length(
                &prefix.indices[0..compare_length],
                &g.announcement.position.indices[0..compare_length],
            );
            for mo in &g.obligations {
                //Compare up to gcp_length or the length of the match obligation position
                let compare_length = min(gcp_length, mo.position.len());
                //gcp_length shrinks if they are not the same up to compare_length
                gcp_length = MatchGoal::common_prefix_length(
                    &prefix.indices[0..compare_length],
                    &mo.position.indices[0..compare_length],
                );
            }
        }
        //The gcp is constructed by taking the first gcp_length indices of the first match goal prefix
        let greatest_common_prefix = SmallVec::from_slice(&prefix.indices[0..gcp_length]);
        ExplicitPosition {
            indices: greatest_common_prefix,
        }
    }

    // Assumes two slices are of the same length and computes to what length they are equal
    fn common_prefix_length(pos1: &[usize], pos2: &[usize]) -> usize {
        assert_eq!(
            pos1.len(),
            pos2.len(),
            "Given arrays should be of the same length."
        );

        let mut common_length = 0;
        for i in 0..pos1.len() {
            if pos1.get(i).unwrap() == pos2.get(i).unwrap() {
                common_length += 1;
            } else {
                break;
            }
        }
        common_length
    }

    /// Removes the first len position indices of the match goal and obligation positions
    pub fn remove_prefix(mut goals: Vec<MatchGoal>, len: usize) -> Vec<MatchGoal> {
        for goal in &mut goals {
            // update match announcement
            goal.announcement.position = ExplicitPosition {
                indices: SmallVec::from_slice(&goal.announcement.position.indices[len..]),
            };
            for mo_index in 0..goal.obligations.len() {
                let shortened = ExplicitPosition {
                    indices: SmallVec::from_slice(
                        &goal.obligations.get(mo_index).unwrap().position.indices[len..],
                    ),
                };
                goal.obligations.get_mut(mo_index).unwrap().position = shortened;
            }
        }
        goals
    }

    /// Checks for two positions whether one is a subposition of the other.
    /// For example 2.2.3 and 2 are comparable. 2.2.3 and 1 are not.
    pub fn pos_comparable(p1: &ExplicitPosition, p2: &ExplicitPosition) -> bool {
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
    pub fn partition(goals: Vec<MatchGoal>) -> Vec<(Vec<MatchGoal>, Vec<ExplicitPosition>)> {
        let mut partitions = vec![];

        //If one of the goals has a root position all goals are related.
        if goals.iter().any(|g| g.announcement.position.is_empty()) {
            let mut all_positions = Vec::new();
            for g in &goals {
                if !all_positions.contains(&g.announcement.position) {
                    all_positions.push(g.announcement.position.clone())
                }
            }
            partitions.push((goals, all_positions));
            return partitions;
        }

        // Create a mapping from positions to goals, goals are represented with an index
        // on function parameter goals
        let mut position_to_goals = HashMap::new();
        let mut all_positions = Vec::new();
        for (i, g) in goals.iter().enumerate() {
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
            // Start the partition with a position
            let p = &all_positions[p_index];
            let mut pos_in_partition = vec![];
            pos_in_partition.push(p.clone());
            let mut goals_in_partition = vec![];

            // put the goals with position p in the partition
            let g = position_to_goals.get(p).unwrap();
            for i in g {
                goals_in_partition.push(goals[*i].clone());
            }

            // Go over the positions until we find a position that is not comparable to p
            // Because all_positions is sorted we know that once we find a position that is not comparable
            // all subsequent positions will also not be comparable.
            // Moreover, all positions in the partition are related to p. p is the highest in the partition.
            p_index += 1;
            while p_index < all_positions.len()
                && MatchGoal::pos_comparable(p, &all_positions[p_index])
            {
                pos_in_partition.push(all_positions[p_index].clone());
                //Put the goals with position all_positions[p_index] in the partition
                let g = position_to_goals.get(&all_positions[p_index]).unwrap();
                for i in g {
                    goals_in_partition.push(goals[*i].clone());
                }
                p_index += 1;
            }
            partitions.push((goals_in_partition, pos_in_partition));
        }
        partitions
    }
}

/// Adds the position of a variable to the equivalence classes
fn update_equivalences(ve: &mut Vec<EquivalenceClass>, variable: &ATerm, pos: ExplicitPosition) {
    // Check if the variable was seen before
    if ve.iter().any(|ec| &ec.variable == variable) {
        for ec in ve.iter_mut() {
            //Find the equivalence class and add the position
            if &ec.variable == variable && !ec.positions.iter().any(|x| x == &pos) {
                ec.positions.push(pos.clone());
            }
        }
    } else {
        // If the variable was not found at another position add a new equivalence class
        ve.push(EquivalenceClass {
            variable: variable.clone(),
            positions: vec![pos],
        });
    }
}
