use std::io;

use itertools::Itertools;

use crate::set_automaton::{SetAutomaton, State};
use core::fmt;
use std::io::Write;

use super::{MatchAnnouncement, MatchObligation, Transition};

impl<M> fmt::Debug for SetAutomaton<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// Implement display for a transition with a term pool
impl<M> fmt::Display for Transition<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transition {{ {}, announce: [{}], dest: [{}] }}",
            self.symbol,
            self.announcements.iter().map(|(x, _)| { x }).format(", "),
            self.destinations.iter().format_with(", ", |element, f| {
                f(&format_args!("{} -> {}", element.0, element.1))
            })
        )
    }
}

/// Implement display for a match announcement
impl fmt::Display for MatchAnnouncement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})@{}", self.rule, self.position)
    }
}

impl fmt::Display for MatchObligation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.pattern, self.position)
    }
}

/// Implement display for a state with a term pool
impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Label: {}, ", self.label)?;
        writeln!(f, "Match goals: [")?;
        for m in &self.match_goals {
            writeln!(f, "\t {}", m)?;
        }
        write!(f, "]")
    }
}

impl<M> fmt::Display for SetAutomaton<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "States: {{")?;

        for (state_index, s) in self.states.iter().enumerate() {
            writeln!(f, "State {} {{\n{}", state_index, s)?;

            writeln!(f, "Transitions: {{")?;
            for ((from, _), tr) in self.transitions.iter() {
                if state_index == *from {
                    writeln!(f, "\t {}", tr)?;
                }
            }
            writeln!(f, "}}")?;
        }

        writeln!(f, "}}")
    }
}

impl<M> SetAutomaton<M> {
    /// Create a .dot file and convert it to a .svg using the dot command
    pub fn to_dot_graph(
        &self,
        mut f: impl Write,
        show_backtransitions: bool,
        show_final: bool,
    ) -> io::Result<()> {
        // Write the header anf final states.
        writeln!(&mut f, "digraph{{")?;

        if show_final {
            writeln!(&mut f, "  final[label=\"💩\"];")?;
        }

        for (i, s) in self.states.iter().enumerate() {
            let match_goals = s
                .match_goals
                .iter()
                .format_with("\\n", |goal, f| f(&format_args!("{}", html_escape::encode_safe(&format!("{}", goal)))));

            writeln!(
                &mut f,
                "  s{}[shape=record label=\"{{{{s{} | {}}} | {}}}\"]",
                i, i, s.label, match_goals
            )?;
        }

        for ((i, _), tr) in &self.transitions {
            let announcements = tr
                .announcements
                .iter()
                .format_with(", ", |(announcement, _), f| {
                    f(&format_args!(
                        "{}@{}",
                        announcement.rule.rhs, announcement.position
                    ))
                });

            if tr.destinations.is_empty() {
                if show_final {
                    writeln!(
                        &mut f,
                        "  s{} -> final [label=\"{} \\[{}\\]\"]",
                        i, tr.symbol, announcements
                    )?;
                }
            } else {
                writeln!(&mut f, "  \"s{}{}\" [shape=point]", i, tr.symbol,).unwrap();
                writeln!(
                    &mut f,
                    "  s{} -> \"s{}{}\" [label=\"{} \\[{}\\]\"]",
                    i, i, tr.symbol, tr.symbol, announcements
                )?;

                for (pos, des) in &tr.destinations {
                    if show_backtransitions || *des != 0 {
                        // Hide backpointers to the initial state.
                        writeln!(
                            &mut f,
                            "  \"s{}{}\" -> s{} [label = \"{}\"]",
                            i, tr.symbol, des, pos
                        )?;
                    }
                }
            }
        }
        writeln!(&mut f, "}}")
    }
}