use std::fmt;
use std::ops::Deref;

use crate::set_automaton::{EnhancedMatchAnnouncement, SetAutomaton};
use crate::utilities::ExplicitPosition;
use mcrl2::aterm::{Protected, TermPool};
use mcrl2::data::{DataExpression, DataExpressionRef};

use super::{get_position, substitute};

/// A Configuration is part of the configuration stack/tree
/// It contains:
///     1. the index of a state
///     2. The subterm at the position of the configuration.
///     3. The difference of position compared to the parent configuration (None for the root).
///         Note that it stores a reference to a position. It references the position listed on
///         a transition of the set automaton.
#[derive(Debug)]
pub(crate) struct Configuration<'a> {
    pub state: usize,
    pub position: Option<&'a ExplicitPosition>,
}

/// SideInfo stores additional information of a configuration.
/// It stores an index of the corresponding configuration on the configuration stack.
#[derive(Debug)]
pub(crate) struct SideInfo<'a> {
    pub corresponding_configuration: usize,
    pub info: SideInfoType<'a>,
}

/// Three types of side information. See the stack rewriter on how they are used.
#[derive(Debug)]
pub(crate) enum SideInfoType<'a> {
    SideBranch(&'a [(ExplicitPosition, usize)]),
    DelayedRewriteRule(&'a EnhancedMatchAnnouncement),
    EquivalenceAndConditionCheck(&'a EnhancedMatchAnnouncement),
}

/// A configuration stack. The first element is the root of the configuration tree.
#[derive(Debug)]
pub(crate) struct ConfigurationStack<'a> {
    pub stack: Vec<Configuration<'a>>,
    pub terms: Protected<Vec<DataExpressionRef<'static>>>,

    /// Separate stack with extra information on some configurations
    pub side_branch_stack: Vec<SideInfo<'a>>,

    /// Current node. Becomes None when the configuration tree is completed
    pub current_node: Option<usize>,

    /// Upon applying a rewrite rule we do not immediately update the subterm stored in every configuration on the stack.
    /// That would be very expensive. Instead we ensure that the subterm in the current_node is always up to date.
    /// oldest_reliable_subterm is an index to the highest configuration in the tree that is up to date.
    pub oldest_reliable_subterm: usize,
}

impl<'a> ConfigurationStack<'a> {
    /// Initialise the stack with one Configuration containing 'term' and the initial state of the set automaton
    pub fn new(state: usize, term: DataExpression) -> ConfigurationStack<'a> {
        let mut conf_list = ConfigurationStack {
            stack: Vec::with_capacity(8),
            side_branch_stack: vec![],
            terms: Protected::new(vec![]),
            current_node: Some(0),
            oldest_reliable_subterm: 0,
        };
        conf_list.stack.push(Configuration {
            state,
            position: None,
        });

        let mut write_conf_list = conf_list.terms.write();
        let term = write_conf_list.protect(&term.copy().into());
        write_conf_list.push(term.into());
        drop(write_conf_list);

        conf_list
    }

    /// Obtain the first unexplored node of the stack, which is just the top of the stack.
    pub(crate) fn get_unexplored_leaf(&self) -> Option<usize> {
        self.current_node
    }

    /// Returns the lowest configuration in the tree with SideInfo
    pub(crate) fn get_prev_with_side_info(&self) -> Option<usize> {
        self.side_branch_stack
            .last()
            .map(|si| si.corresponding_configuration)
    }

    /// Grow a Configuration with index c. tr_slice contains the hypertransition to possibly multiple states
    pub fn grow(&mut self, c: usize, tr_slice: &'a [(ExplicitPosition, usize)]) {
        // Pick the first transition to grow the stack
        let (pos, des) = tr_slice.first().unwrap();

        // If there are more transitions store the remaining on the side stack
        let tr_slice: &[(ExplicitPosition, usize)] = &(tr_slice)[1..];
        if !tr_slice.is_empty() {
            self.side_branch_stack.push(SideInfo {
                corresponding_configuration: c,
                info: SideInfoType::SideBranch(tr_slice),
            })
        }

        // Create a new configuration and push it onto the stack
        let new_leaf = Configuration {
            state: *des,
            position: Some(pos),
        };
        self.stack.push(new_leaf);

        // Push the term belonging to the leaf.
        let mut write_terms = self.terms.write();
        let t = write_terms.protect(&get_position(write_terms[c].deref(), pos).into());
        write_terms.push(t.into());

        self.current_node = Some(c + 1);
    }

    /// When rewriting prune "prunes" the configuration stack to the place where the first symbol
    /// of the matching rewrite rule was observed (at index 'depth').
    pub fn prune(
        &mut self,
        tp: &mut TermPool,
        automaton: &SetAutomaton,
        depth: usize,
        new_subterm: DataExpression,
    ) {
        self.current_node = Some(depth);

        // Reroll the configuration stack by truncating the Vec (which is a constant time operation)
        self.stack.truncate(depth + 1);
        self.terms.write().truncate(depth + 1);

        // Remove side info for the deleted configurations
        self.roll_back_side_info_stack(depth, true);

        // Update the subterm stored at the prune point.
        // Note that the subterm stored earlier may not have been up to date. We replace it with a term that is up to date
        let mut write_terms = self.terms.write();
        let subterm = write_terms.protect(
            substitute(
                tp,
                write_terms[depth].deref(),
                new_subterm.into(),
                &automaton.states[self.stack[depth].state].label.indices,
            )
            .deref(),
        );
        write_terms[depth] = subterm.into();

        self.oldest_reliable_subterm = depth;
    }

    /// Removes side info for configurations beyond configuration index 'end'.
    /// If 'including' is true the side info for the configuration with index 'end' is also deleted.
    pub fn roll_back_side_info_stack(&mut self, end: usize, including: bool) {
        loop {
            match self.side_branch_stack.last() {
                None => {
                    break;
                }
                Some(sbi) => {
                    if sbi.corresponding_configuration < end
                        || (sbi.corresponding_configuration <= end && !including)
                    {
                        break;
                    } else {
                        self.side_branch_stack.pop();
                    }
                }
            }
        }
    }

    /// Roll back the configuration stack to level 'depth'.
    /// This function is used exclusively when a subtree has been explored and no matches have been found.
    pub fn jump_back(&mut self, depth: usize, tp: &mut TermPool) {
        // Updated subterms may have to be propagated up the configuration tree
        self.integrate_updated_subterms(depth, tp, true);
        self.current_node = Some(depth);
        self.stack.truncate(depth + 1);
        self.terms.write().truncate(depth + 1);

        self.roll_back_side_info_stack(depth, false);
    }

    /// When going back up the configuration tree the subterms stored in the configuration tree must be updated
    /// This function ensures that the Configuration at depth 'end' is made up to date.
    /// If store_intermediate is true, all configurations below 'end' are also up to date.
    pub fn integrate_updated_subterms(
        &mut self,
        end: usize,
        tp: &mut TermPool,
        store_intermediate: bool,
    ) {
        // Check if there is anything to do. Start updating from self.oldest_reliable_subterm
        let mut up_to_date = self.oldest_reliable_subterm;
        if up_to_date == 0 || end >= up_to_date {
            return;
        }

        let mut write_terms = self.terms.write();
        let mut subterm = write_terms.protect(&write_terms[up_to_date]);

        // Go over the configurations one by one until we reach 'end'
        while up_to_date > end {
            // If the position is not deepened nothing needs to be done, otherwise substitute on the position stored in the configuration.
            subterm = match self.stack[up_to_date].position {
                None => subterm,
                Some(p) => {
                    let t = substitute(
                        tp,
                        write_terms[up_to_date - 1].deref(),
                        subterm.protect().into(),
                        &p.indices,
                    );
                    write_terms.protect(&t).into()
                }
            };
            up_to_date -= 1;

            if store_intermediate {
                let subterm = write_terms.protect(&subterm);
                write_terms[up_to_date] = subterm.into();
            }
        }

        self.oldest_reliable_subterm = up_to_date;

        let subterm = write_terms.protect(&subterm);
        write_terms[up_to_date] = subterm.into();
    }

    /// Final term computed by integrating all subterms up to the root configuration
    pub fn compute_final_term(&mut self, tp: &mut TermPool) -> DataExpression {
        self.jump_back(0, tp);
        self.terms.read()[0].protect()
    }

    /// Returns a SideInfoType object if there is side info for the configuration with index 'leaf_index'
    pub fn pop_side_branch_leaf(
        stack: &mut Vec<SideInfo<'a>>,
        leaf_index: usize,
    ) -> Option<SideInfoType<'a>> {
        let should_pop = match stack.last() {
            None => false,
            Some(si) => si.corresponding_configuration == leaf_index,
        };
        if should_pop {
            Some(stack.pop().unwrap().info)
        } else {
            None
        }
    }
}

impl<'a> fmt::Display for ConfigurationStack<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Current node: {:?}", self.current_node)?;
        for (i, c) in self.stack.iter().enumerate() {
            writeln!(f, "Configuration {} ", i)?;
            writeln!(f, "    State: {:?}", c.state)?;
            writeln!(
                f,
                "    Position: {}",
                match c.position {
                    Some(x) => x.to_string(),
                    None => "None".to_string(),
                }
            )?;
            writeln!(f, "    Subterm: {}", self.terms.read()[i])?;

            for side_branch in &self.side_branch_stack {
                if i == side_branch.corresponding_configuration {
                    writeln!(f, "    Side branch: {} ", side_branch.info)?;
                }
            }
        }

        Ok(())
    }
}

impl<'a> fmt::Display for SideInfoType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SideInfoType::SideBranch(tr_slice) => {
                let mut first = true;
                for (position, index) in tr_slice.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} {}", position, *index)?;
                    first = false;
                }
            }
            SideInfoType::DelayedRewriteRule(ema) => {
                write!(f, "delayed rule: {}", ema)?;
            }
            SideInfoType::EquivalenceAndConditionCheck(ema) => {
                write!(f, "equivalence {}", ema)?;
            }
        }

        Ok(())
    }
}
