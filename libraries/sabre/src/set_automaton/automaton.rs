use std::rc::Rc;

use mcrl2_rust::atermpp::{TermPool, Symbol};
use smallvec::SmallVec;

use crate::utilities::ExplicitPosition;

use super::EnhancedMatchAnnouncement;

// The Set Automaton used for matching based on 
pub struct SetAutomaton
{
  pub(crate) states: Vec<State>,
  pub(crate) term_pool: Rc<TermPool>
}

#[derive(Debug)]
pub(crate) struct State
{
  pub(crate) label: ExplicitPosition,

  /// Note that transitions are indexed by the index given by the OpId from a function symbol.
  transitions: Vec<Transition>,
  //match_goals: Vec<MatchGoal>
}

struct MatchGoal
{
  //obligations: Vec<Match
}

#[derive(Debug,Clone)]
struct Transition
{
  symbol: Symbol,
  announcements: SmallVec<[EnhancedMatchAnnouncement;1]>,
  destinations: SmallVec<[(ExplicitPosition, usize);1]>
}