use std::{rc::Rc, collections::VecDeque};

use ahash::HashMap;
use mcrl2_rust::atermpp::{TermPool, Symbol, ATerm};
use smallvec::{SmallVec, smallvec};

use crate::{utilities::ExplicitPosition, rewrite_specification::{Rule, RewriteSpecification}};

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
  match_goals: Vec<MatchGoal>
}

#[derive(Debug,Clone)]
struct Transition
{
  symbol: Symbol,
  announcements: SmallVec<[EnhancedMatchAnnouncement;1]>,
  destinations: SmallVec<[(ExplicitPosition, usize);1]>
}

#[derive(Clone,Debug)]
pub(crate) struct MatchGoal
{
  obligations: Vec<MatchObligation>,
  announcement: MatchAnnouncement,
}

#[derive(Clone,Debug)]
pub(crate) struct MatchObligation
{
  pattern: ATerm,
  position: ExplicitPosition
}

#[derive(Clone,Debug)]
pub(crate) struct MatchAnnouncement
{
  pub rule: Rule,
  pub position: ExplicitPosition,
  symbols_seen: usize
}

impl SetAutomaton 
{
  /// Construct a set automaton. If 'apma' is true construct an APMA instead.
  /// An APMA is just a set automaton that does not partition the match goals on a transition
  /// and does not add fresh goals.
  pub(crate) fn construct(tp: Rc<TermPool>, spec: RewriteSpecification, apma: bool) -> SetAutomaton 
  {
      // States are labelled s0, s1, s2, etcetera. state_counter keeps track of count.
      let mut state_counter:usize = 1;

      // The initial state has a match goals for each pattern. For each pattern l there is a match goal
      // with one obligation l@ε and announcement l@ε.
      let mut match_goals = Vec::<MatchGoal>::new();
      for rr in &spec.rewrite_rules 
      {
          match_goals.push(MatchGoal{
              obligations: vec![MatchObligation{pattern: rr.lhs.clone(), position: ExplicitPosition::empty_pos()}],
              announcement: MatchAnnouncement {rule: (*rr).clone(), position: ExplicitPosition::empty_pos(), symbols_seen: 0}
          });
      }

      // Match goals need to be sorted so that we can easily check whether a state with a certain
      // set of match goals already exists.
      match_goals.sort();

      // Create the initial state
      let initial_state = State {
        label: ExplicitPosition::empty_pos(),
        transitions: Vec::with_capacity(spec.symbols.len()),
        match_goals: match_goals
      };

      // HashMap from goals to state number
      let mut map_goals_state = HashMap::default();

      //Queue of states that still need to be explored
      let mut queue = VecDeque::new();
      queue.push_back(0);

      map_goals_state.insert(match_goals.clone(), 0);
      
      let mut states = vec![initial_state];

      while !queue.is_empty() 
      {
          // Pick a state to explore
          let s_index = queue.pop_front().unwrap();

          // Compute the transitions from the state in parallel using rayon
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

      SetAutomaton{
          states,
          term_pool
      }
  }
}