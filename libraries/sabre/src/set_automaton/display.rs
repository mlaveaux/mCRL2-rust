use std::{path::Path, io};

use itertools::Itertools;

use crate::set_automaton::{EnhancedMatchAnnouncement, SetAutomaton, State};
use core::fmt;
use std::io::Write;

use super::{Transition, MatchAnnouncement, MatchGoal, MatchObligation, EquivalenceClass};

impl fmt::Debug for SetAutomaton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// Implement display for a transition with a term pool
impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transition {{ {}, announce: [{}], dest: [{}] }}",
            self.symbol,
            self.announcements.iter().format(", "),
            self.destinations.iter().format_with(", ", |element, f| {
                f(&format_args!("{} -> {}", element.0, element.1))
            })
        )
    }
}

/// Implement display for a match announcement
impl fmt::Display for MatchAnnouncement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({})@{}",
            self.rule,
            self.position
        )
    }
}

impl fmt::Display for MatchObligation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}@{}",
            self.pattern,
            self.position
        )
    }
}

impl fmt::Display for EquivalenceClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{{ {} }}", self.variable, self.positions.iter().format(", "))
    }
}

impl fmt::Display for EnhancedMatchAnnouncement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.announcement)?;
        write!(f, "{{ conditions: [{}], equivalences: [{}] }}",
        self.announcement.rule.conditions.iter().format(", "),
        self.equivalence_classes.iter().format(", "))
    }
}

/// Implement display for a state with a term pool
impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Label: {}, ",
            self.label
        )?;
        writeln!(f, "Match goals: [")?;
        for m in &self.match_goals {
            writeln!(f, "\t {}", m)?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for SetAutomaton {
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

impl SetAutomaton {
    /// Create a .dot file and convert it to a .svg using the dot command
    pub fn to_dot_graph(&self, mut f: impl Write) -> io::Result<()>{
        writeln!(&mut f, "digraph{{")?;
        writeln!(&mut f, "  final[label=\"💩\"];")?;
        for (i, s) in self.states.iter().enumerate() {
            let mut match_goals = String::new();
            for mg in &s.match_goals {

                match_goals += &format!("{}", mg.obligations.iter().format_with(", ", |mo, f| {
                    f(&format_args!("{}@{}", &mo.pattern, mo.position))
                }));

                match_goals += &format!(
                    " 👉 {}-\\>{}@{}, {}\\l",
                    &mg.announcement.rule.lhs,
                    &mg.announcement.rule.rhs,
                    mg.announcement.position,
                    mg.announcement.symbols_seen
                );
            }
            writeln!(
                &mut f,
                "  s{}[shape=record label=\"{{{{s{} | {}}} | {}}}\"]",
                i,
                i,
                s.label,
                match_goals
            )?;
        }

        for ((i, _), tr) in &self.transitions {
            let mut announcements = "".to_string();


            let announcements = tr.announcements.iter().format_with(", ", |announcement, f| {
                f(announcement)
            });

            if tr.destinations.is_empty() {
                writeln!(
                    &mut f,
                    "  s{} -> final [label=\"{}{}\"]",
                    i, tr.symbol, announcements
                )?;
            } else {
                writeln!(&mut f, "  \"s{}{}\" [shape=point]", i, tr.symbol,).unwrap();
                writeln!(
                    &mut f,
                    "  s{} -> \"s{}{}\" [label=\"{}{}\"]",
                    i, i, tr.symbol, tr.symbol, announcements
                )?;

                for (pos, des) in &tr.destinations {
                    writeln!(
                        &mut f,
                        "  \"s{}{}\" -> s{} [label = \"{}\"]",
                        i,
                        tr.symbol,
                        des,
                        pos
                    )?;
                }
            }
        }
        writeln!(&mut f, "}}")
    }
}

impl fmt::Display for MatchGoal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let mut first = true;
        for obligation in &self.obligations {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", obligation)?;
            first = false;
        }

        write!(f, " ↪ {}", self.announcement)
    }
}