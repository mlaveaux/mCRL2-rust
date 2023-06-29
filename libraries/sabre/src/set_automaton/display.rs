use std::process::Command;

use mcrl2_rust::{data::{DataFunctionSymbol, DataApplication}, atermpp::ATerm};

use crate::set_automaton::{EnhancedMatchAnnouncement, SetAutomaton, State};
use core::fmt;
use std::fs::File;
use std::io::Write;

use super::{Transition, MatchAnnouncement, MatchGoal, Derivative, MatchObligation};

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

        write!(f, "] Destinations: [")?;
        first = true;
        for (pos, s) in &self.destinations {
            if !first {
                write!(f, " ")?;
            }
            write!(f, "({},{})", pos, s)?;
            first = false;
        }
        write!(f, "]}}")
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
            )
        } else {
            write!(
                f,
                "{}@{}",
                <ATerm as Into<DataApplication>>::into(self.rule.lhs.clone()),
                self.position
            )
        }
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

        write!(f, ", [")?;
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

        write!(f, "] [")?;
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
            "State {{Label: {}, ",
            self.label
        )?;
        writeln!(f, "Transitions: [")?;
        for t in &self.transitions {
            writeln!(f, "{}", t)?;
        }
        writeln!(f, "],\n Match goals: [")?;
        for m in &self.match_goals {
            write!(f, "\t Obligations:")?;
            for mo in &m.obligations {
                write!(f, "{}@{}  ", &mo.pattern, mo.position)?;
            }

            write!(
                f,
                "Announcement: {}@{}, ",
                &m.announcement.rule.lhs,
                m.announcement.position
            )?;
            write!(f, "Conditions: ")?;
            for c in &m.announcement.rule.conditions {
                let comparison_symbol = if c.equality { "==" } else { "<>" };
                write!(f, "{} {} {}", &c.lhs, comparison_symbol, &c.rhs)?;
            }
            writeln!(f)?;
        }
        write!(f, "]}}")
    }
}

impl fmt::Display for SetAutomaton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "States: {{")?;
        for (state_index, s) in self.states.iter().enumerate() {
            writeln!(f, "State {}: {}", state_index, s)?;
        }
        writeln!(f, "}}")
    }
}

impl SetAutomaton {
    /// Create a .dot file and convert it to a .svg using the dot command
    pub fn to_dot_graph(&self, f_base_name: &str) {
        let mut f_dot_name = f_base_name.to_string();
        f_dot_name += ".dot";
        let mut f = File::create(f_dot_name.clone()).unwrap();

        writeln!(&mut f, "digraph{{").unwrap();
        writeln!(&mut f, "  final[label=\"💩\"];").unwrap();
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
            )
            .unwrap();
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
                    )
                    .unwrap();
                } else {
                    writeln!(&mut f, "  s{}{}[shape=point]", i, tr.symbol).unwrap();
                    writeln!(
                        &mut f,
                        "  s{}->s{}{}[label=\"{}{}\"]",
                        i, i, tr.symbol, tr.symbol, announcements
                    )
                    .unwrap();
                    for (pos, des) in &tr.destinations {
                        writeln!(
                            &mut f,
                            "  s{}{}->s{} [label =\"{}\"]",
                            i,
                            tr.symbol,
                            des,
                            pos
                        )
                        .unwrap();
                    }
                }
            }
        }
        writeln!(&mut f, "}}").unwrap();
        dot2svg(f_base_name);
    }
}

fn dot2svg(f_name: &str) {
    let _ = Command::new("dot")
        .args([
            "-Tsvg",
            &format!("{}.dot", f_name),
            &format!("-o {}.svg", f_name),
        ])
        .output()
        .expect("failed to execute process");
}

/// Implement display for a match announcement
impl fmt::Display for Derivative {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "completed: ")?;
        for mg in &self.completed {
            writeln!(f, "{}", mg)?;
        }

        writeln!(f, "")?;
        writeln!(f, "unchanged: ")?;
        for mg in &self.unchanged {
            writeln!(f, "{}", mg)?;
        }

        writeln!(f, "")?;
        writeln!(f, "reduced: ")?;
        for mg in &self.reduced {
            writeln!(f, "{}", mg)?;
        }

        Ok(())
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