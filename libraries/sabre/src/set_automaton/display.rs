use std::{path::Path, io};

use mcrl2_sys::{data::{DataFunctionSymbol, DataApplication}, atermpp::ATerm};

use crate::set_automaton::{EnhancedMatchAnnouncement, SetAutomaton, State};
use core::fmt;
use std::fs::File;
use std::io::Write;

use super::{Transition, MatchAnnouncement, MatchGoal, MatchObligation};

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
            "Transition {{ Symbol: {}, Outputs: [",
            self.symbol
        )?;

        let mut first = true;
        for r in &self.announcements {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", r)?;
            first = false;
        }

        write!(f, "], Destinations: [")?;
        first = true;
        for (pos, s) in &self.destinations {
            if !first {
                write!(f, " ")?;
            }
            write!(f, "({},{})", pos, s)?;
            first = false;
        }
        write!(f, "] }}")
    }
}

/// Implement display for a match announcement
impl fmt::Display for MatchAnnouncement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.rule.lhs.is_data_function_symbol() {
            write!(
                f,
                "{}@{}",
                <ATerm as Into<DataFunctionSymbol>>::into(self.rule.lhs.clone()),
                self.position
            )?;
        } else {
            write!(
                f,
                "{}@{}",
                <ATerm as Into<DataApplication>>::into(self.rule.lhs.clone()),
                self.position
            )?;
        }

        if !self.rule.conditions.is_empty() {   
            write!(f, "[")?;

            let mut first = true;
            for c in &self.rule.conditions {
                if !first { 
                    write!(f, ", ")?;
                }

                let comparison_symbol = if c.equality { "==" } else { "<>" };
                write!(f, "{} {} {}", &c.lhs, comparison_symbol, &c.rhs)?;

                first = false;
            } 

            write!(f, "]")?;
        }

        Ok(())
    }
}

impl fmt::Display for MatchObligation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.pattern.is_data_function_symbol() {
            write!(
                f,
                "{}@{}",
                <ATerm as Into<DataFunctionSymbol>>::into(self.pattern.clone()),
                self.position
            )
        } else {
            write!(
                f,
                "{}@{}",
                <ATerm as Into<DataApplication>>::into(self.pattern.clone()),
                self.position
            )
        }

    }
}

impl fmt::Display for EnhancedMatchAnnouncement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.announcement)?;

        write!(f, ", equivalences: [")?;
        for ec in &self.equivalence_classes {
            write!(f, "{}{{", ec.variable)?;
            let mut first = true;
            for p in &ec.positions {
                if !first {
                    write!(f, ", ")?;
                } 
                write!(f, "{}", p)?;
                first = false;
            }
            write!(f, "}} ")?;
        }

        write!(f, "], conditions: [")?;
        for (i, c) in self.announcement.rule.conditions.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            let comparison_symbol = if c.equality { "==" } else { "<>" };
            write!(f, "{} {} {}", &c.lhs, comparison_symbol, &c.rhs)?;
        }
        write!(f, "]")
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
        writeln!(f, "Transitions: [")?;
        for t in &self.transitions {
            writeln!(f, "\t {}", t)?;
        }
        writeln!(f, "],")?;
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
            writeln!(f, "State {}\n{{\n{}\n}}", state_index, s)?;
        }
        writeln!(f, "}}")
    }
}

impl SetAutomaton {
    /// Create a .dot file and convert it to a .svg using the dot command
    pub fn to_dot_graph(&self, filename: &Path) -> io::Result<()>{
        let mut f = File::create(filename)?;

        writeln!(&mut f, "digraph{{")?;
        writeln!(&mut f, "  final[label=\"💩\"];")?;
        for (i, s) in self.states.iter().enumerate() {
            let mut match_goals = "".to_string();
            for mg in &s.match_goals {
                for (i, mo) in mg.obligations.iter().enumerate() {
                    if i > 0 {
                        match_goals += ", ";
                    }
                    match_goals += &*format!("{}@{}", &mo.pattern, mo.position);
                }
                match_goals += &*format!(
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

        for (i, s) in self.states.iter().enumerate() {
            for tr in s.transitions.iter() {
                let mut announcements = "".to_string();

                for a in &tr.announcements {
                    announcements +=
                        &format!(", {}@{}", &a.announcement.rule.lhs, a.announcement.position);
                }

                if tr.destinations.is_empty() {
                    writeln!(
                        &mut f,
                        "  s{}-> final [label=\"{}{}\"]",
                        i, tr.symbol, announcements
                    )?;
                } else {
                    writeln!(&mut f, "  s{}{}[shape=point]", i, tr.symbol).unwrap();
                    writeln!(
                        &mut f,
                        "  s{}->s{}{}[label=\"{}{}\"]",
                        i, i, tr.symbol, tr.symbol, announcements
                    )?;

                    for (pos, des) in &tr.destinations {
                        writeln!(
                            &mut f,
                            "  s{}{}->s{} [label =\"{}\"]",
                            i,
                            tr.symbol,
                            des,
                            pos
                        )?;
                    }
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