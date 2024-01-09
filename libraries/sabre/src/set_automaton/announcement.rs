use std::cmp::min;

use crate::{
    rewrite_specification::Rule,
    utilities::{create_var_map, get_position, ExplicitPosition, SemiCompressedTermTree, PositionIterator}, Config,
};
use ahash::{HashMap, HashMapExt};
use mcrl2::{aterm::{ATerm, ATermTrait, ATermRef}, data::{is_data_variable, is_data_expression}};
use smallvec::SmallVec;

use super::{MatchObligation, get_data_function_symbol, get_data_arguments};

/// An equivalence class is a variable with (multiple) positions. This is
/// necessary for non-linear patterns. It is used by EnhancedMatchAnnouncement
/// to store what positions need to be compared.
///
/// # Example
/// Suppose we have a pattern f(x,x), where x is a variable. Then it will have
/// one equivalence class storing "x" and the positions 1 and 2. The function
/// equivalences_hold checks whether the term has the same term on those
/// positions. For example, it will returns false on the term f(a, b) and true
/// on the term f(a, a).
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct EquivalenceClass {
    pub(crate) variable: ATerm,
    pub(crate) positions: Vec<ExplicitPosition>,
}

/// Checks if the equivalence classes hold for the given term.
pub fn check_equivalence_classes(term: &ATerm, eqs: &[EquivalenceClass]) -> bool {
    eqs.iter().all(|ec| {
        debug_assert!(ec.positions.len() >= 2, "An equivalence class must contain at least two positions");

        // The term at the first position must be equivalent to all other positions.
        let mut iter_pos = ec.positions.iter();
        let first = iter_pos.next().unwrap();
        iter_pos.all(|other_pos| get_position(term, first) == get_position(term, other_pos))
    })
}

/// Adds the position of a variable to the equivalence classes
fn update_equivalences(ve: &mut Vec<EquivalenceClass>, variable: ATermRef, pos: ExplicitPosition) {
    // Check if the variable was seen before
    if ve.iter().any(|ec| ec.variable.copy() == variable) {
        for ec in ve.iter_mut() {
            // Find the equivalence class and add the position
            if ec.variable.copy() == variable && !ec.positions.iter().any(|x| x == &pos) {
                ec.positions.push(pos);
                break;
            }
        }
    } else {
        // If the variable was not found at another position add a new equivalence class
        ve.push(EquivalenceClass {
            variable: variable.protect(),
            positions: vec![pos],
        });
    }
}

/// Derives the positions in a pattern with same variable (for non-linear patters)
fn derive_equivalence_classes(announcement: &MatchAnnouncement) -> Vec<EquivalenceClass> {
    let mut var_equivalences = vec![];

    for (term, pos) in PositionIterator::new(announcement.rule.lhs.copy()) {
        if is_data_variable(&term) {
            // Register the position of the variable
            update_equivalences(&mut var_equivalences, term, pos);
        }
    }

    // Discard variables that only occur once
    var_equivalences.retain(|x| x.positions.len() > 1);
    var_equivalences
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
    /// Conditions for the left hand side.
    pub conditions: Vec<EMACondition>,

    // TODO: This information is rewriter specific and should be delegated.

    /// Right hand side is stored in the term pool as much as possible with a SemiCompressedTermTree
    pub semi_compressed_rhs: SemiCompressedTermTree,
    /// Whether the rewrite rule duplicates subterms, e.g. times(s(x), y) = plus(y, times(x, y))
    pub is_duplicating: bool,

    /// The innermost rewrite stack for the right hand side and the positions that must be added to the stack.
    pub innermost_stack: Vec<Config>,
    pub variables: Vec<(ExplicitPosition, usize)>,
    pub stack_size: usize,
}

impl EnhancedMatchAnnouncement {
    
    /// For a match announcement derives an EnhancedMatchAnnouncement, which precompiles some information
    /// for faster rewriting.
    pub(crate) fn new(announcement: MatchAnnouncement) -> EnhancedMatchAnnouncement {
        
        // Compute the extra information for the InnermostRewriter.
        // Create a mapping of where the variables are and derive SemiCompressedTermTrees for the
        // rhs of the rewrite rule and for lhs and rhs of each condition.
        // Also see the documentation of SemiCompressedTermTree
        let var_map = create_var_map(&announcement.rule.lhs);
        let sctt_rhs = SemiCompressedTermTree::from_term(&announcement.rule.rhs, &var_map);
        let mut conditions = vec![];

        for c in &announcement.rule.conditions {
            let ema_condition = EMACondition {
                semi_compressed_lhs: SemiCompressedTermTree::from_term(&c.lhs, &var_map),
                semi_compressed_rhs: SemiCompressedTermTree::from_term(&c.rhs, &var_map),
                equality: c.equality,
            };
            conditions.push(ema_condition);
        }

        let is_duplicating = sctt_rhs.contains_duplicate_var_references();
        let equivalence_classes = derive_equivalence_classes(&announcement);

        // Compute the extra information for the InnermostRewriter.
        let mut innermost_stack = vec![];
        let mut positions = vec![];
        let mut stack_size = 0;

        for (term, position) in PositionIterator::new(announcement.rule.rhs.copy()) {
            if let Some(index) = position.indices.last() {
                if *index == 1 {
                    continue; // Skip the function symbol.
                }
            }

            if is_data_variable(&term) {
                positions.push((var_map.get(&term.protect().into()).expect("All variables in the right hand side must occur in the left hand side").clone(), stack_size));
                stack_size += 1;
            } else if is_data_expression(&term) {
                let arity = get_data_arguments(&term).len();
                innermost_stack.push(Config::Construct(get_data_function_symbol(&term).protect(), arity, stack_size));
                stack_size += 1;
            } else {
                // Skip intermediate terms such as UntypeSortUnknown and SortId(@NoValue)
            }
        }

        EnhancedMatchAnnouncement {
            announcement,
            equivalence_classes,
            semi_compressed_rhs: sctt_rhs,
            conditions,
            is_duplicating,
            variables: positions,
            innermost_stack,
            stack_size,
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
        // gcp is empty if there are no match goals
        if goals.is_empty() {
            return ExplicitPosition::empty_pos();
        }

        // Initialise the prefix with the first match goal, can only shrink afterwards
        let first_match_pos = &goals.first().unwrap().announcement.position;
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
                // Compare up to gcp_length or the length of the match obligation position
                let compare_length = min(gcp_length, mo.position.len());
                // gcp_length shrinks if they are not the same up to compare_length
                gcp_length = MatchGoal::common_prefix_length(
                    &prefix.indices[0..compare_length],
                    &mo.position.indices[0..compare_length],
                );
            }
        }
        // The gcp is constructed by taking the first gcp_length indices of the first match goal prefix
        let greatest_common_prefix = SmallVec::from_slice(&prefix.indices[0..gcp_length]);
        ExplicitPosition {
            indices: greatest_common_prefix,
        }
    }

    // Assumes two slices are of the same length and computes to what length they are equal
    fn common_prefix_length(pos1: &[usize], pos2: &[usize]) -> usize {
        debug_assert_eq!(
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

        // If one of the goals has a root position all goals are related.
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

        // Sort the positions. They are now in depth first order.
        all_positions.sort_unstable();

        // Compute the partitions, finished when all positions are processed
        let mut p_index = 0; // position index
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
                // Put the goals with position all_positions[p_index] in the partition
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

#[cfg(test)]
mod tests {
    use ahash::AHashSet;
    use mcrl2::aterm::TermPool;

    use crate::{utilities::to_untyped_data_expression, test_utility::create_rewrite_rule};

    use super::*;

    #[test]
    fn test_derive_equivalence_classes()
    {                
        let mut tp = TermPool::new();
        let announcement = MatchAnnouncement {
            rule: create_rewrite_rule(&mut tp, "f(x, h(x))", "result", &["x"]),
            position: ExplicitPosition::default(),
            symbols_seen: 0,
        };

        let eq: Vec<EquivalenceClass> = derive_equivalence_classes(&announcement);

        assert_eq!(eq,
            vec![
                EquivalenceClass {
                    variable: tp.create_variable("x").into(),
                    positions: vec![ExplicitPosition::new(&[2]), ExplicitPosition::new(&[3, 2])]
                },
            ], "The resulting config stack is not as expected");

        // Check the equivalence class for an example
        let term = tp.from_string("f(a(b), h(a(b)))").unwrap();
        let expression = to_untyped_data_expression(&mut tp, &term, &AHashSet::new());

        assert!(check_equivalence_classes(&expression, &eq), "The equivalence classes are not checked correctly, equivalences: {:?} and term {}", &eq, &expression);
    }
    
    #[test]
    fn test_enhanced_match_announcement() {
        let mut tp = TermPool::new();

        let announcement = MatchAnnouncement {
            rule: create_rewrite_rule(&mut tp, "fact(s(N))", "times(s(N), fact(N))", &["N"]),
            position: ExplicitPosition::default(),
            symbols_seen: 0,
        };

        let ema = EnhancedMatchAnnouncement::new(announcement);

        // Check if the resulting construction succeeded.
        assert_eq!(ema.innermost_stack, 
            vec![
                Config::Construct(tp.create_data_function_symbol("times"), 2, 0),
                Config::Construct(tp.create_data_function_symbol("s"), 1, 1),
                Config::Construct(tp.create_data_function_symbol("fact"), 1, 2),
            ]
        , "The resulting config stack is not as expected");

        assert_eq!(ema.stack_size, 5, "The stack size does not match");
    }

    #[test]
    fn test_enhanced_match_announcement_variable() {
        let mut tp = TermPool::new();

        let announcement = MatchAnnouncement {
            rule: create_rewrite_rule(&mut tp, "f(x)", "x", &["x"]),
            position: ExplicitPosition::default(),
            symbols_seen: 0,
        };

        let ema = EnhancedMatchAnnouncement::new(announcement);

        // Check if the resulting construction succeeded.
        assert!(ema.innermost_stack.is_empty(), "The resulting config stack is not as expected");

        assert_eq!(ema.stack_size, 1, "The stack size does not match");
    }


}