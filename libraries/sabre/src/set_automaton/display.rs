use std::process::Command;

use crate::set_automaton::{SetAutomaton, State, EnhancedMatchAnnouncement};
use std::fs::File;
use std::io::Write;
use core::fmt;

use super::Transition;

impl fmt::Debug for SetAutomaton 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}", self.to_string())
    }
}

/// Implement display for a transition with a term pool
impl fmt::Display for Transition 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        write!(f, "Transition {{ Symbol: {}, Outputs: [", self.symbol.name())?;
        for r in &self.announcements 
        {
            write!(f, "{}", r)?;
        }

        write!(f,"] Destinations: [")?;
        for (pos,s) in &self.destinations 
        {
            write!(f,"({},{})  ", pos.to_string(), s)?;
        }
        write!(f,"]}}")
    }
}

/// Implement display for a match announcement
impl fmt::Display for EnhancedMatchAnnouncement 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}@{}, [", &self.announcement.rule.lhs,self.announcement.position.to_string())?;

        for ec in &self.equivalence_classes 
        {
            write!(f, "{}{{", ec.variable)?;
            let mut first = true;
            for p in &ec.positions {
                if first {
                    write!(f, "{}",p.to_string())?;
                    first = false;
                } else {
                    write!(f, ",{}",p.to_string())?;
                }
            }
            write!(f,"}} ")?;
        }

        write!(f,"] [")?;
        for (i,c) in self.announcement.rule.conditions.iter().enumerate() 
        {
            if i != 0 {write!(f, ", ")?;}
            let comparison_symbol = if c.equality {
                "=="
            } else {
                "<>"
            };
            write!(f, "{} {} {}", &c.lhs, comparison_symbol, &c.rhs)?;
        }
        write!(f,"]")
    }
}

/// Implement display for a state with a term pool
impl fmt::Display for State 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        write!(f, "State {{Label: {}, \n Transitions: [", self.label.to_string())?;
        for t in &self.transitions 
        {
            write!(f, "{}\n", t)?;
        }
        write!(f, "],\n Match goals: [\n")?;
        for m in &self.match_goals 
        {
            write!(f, "\t Obligations:")?;
            for mo in &m.obligations 
            {
                write!(f, "{}@{}  ", &mo.pattern, mo.position.to_string())?;
            }

            write!(f, "Announcement: {}@{}, ", &m.announcement.rule.lhs, m.announcement.position.to_string())?;
            write!(f, "Conditions: ")?;
            for c in &m.announcement.rule.conditions 
            {
                let comparison_symbol = if c.equality {
                    "=="
                } else {
                    "<>"
                };
                write!(f, "{} {} {}", &c.lhs, comparison_symbol, &c.rhs)?;
            }
            write!(f, "\n")?;
        }
        write!(f, "]}}")
    }
}

impl fmt::Display for SetAutomaton 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "States: {{\n")?;

        let mut state_index = 0;
        for s in &self.states 
        {
            write!(f, "State {}: {} \n",state_index, s)?;
            state_index += 1;
        }
        write!(f, "}}\n")
    }
}

impl SetAutomaton 
{
    /// Create a .dot file and convert it to a .svg using the dot command
    pub fn to_dot_graph(&self, f_base_name: &str) 
    {
        let mut f_dot_name = f_base_name.to_string();
        f_dot_name += ".dot";
        let mut f = File::create(f_dot_name.clone()).unwrap();

        writeln!(&mut f, "digraph{{").unwrap();
        writeln!(&mut f, "  final[label=\"💩\"];").unwrap();
        for (i,s) in self.states.iter().enumerate() {
            let mut match_goals = "".to_string();
            for mg in &s.match_goals {
                for (i,mo) in mg.obligations.iter().enumerate() {
                    if i > 0 {
                        match_goals += ", ";
                    }
                    match_goals += &*format!("{}@{}", &mo.pattern, mo.position);
                }
                match_goals += &*format!(" 👉 {}-\\>{}@{}, {}\\l", &mg.announcement.rule.lhs, &mg.announcement.rule.rhs, mg.announcement.position,mg.announcement.symbols_seen);
            }
            writeln!(&mut f, "  s{}[shape=record label=\"{{{{s{} | {}}} | {}}}\"]",i,i,s.label.to_string(),match_goals).unwrap();
        }

        for (i,s) in self.states.iter().enumerate() 
        {
            for tr in s.transitions.iter()
            {
                let symbol = tr.symbol.name();
                let mut announcements = "".to_string();

                for a in &tr.announcements 
                {
                    announcements += &format!(", {}@{}", &a.announcement.rule.lhs, a.announcement.position);
                }

                if tr.destinations.is_empty() {
                    writeln!(&mut f, "  s{}-> final [label=\"{}{}\"]", i, symbol, announcements).unwrap();
                } else {
                    writeln!(&mut f, "  s{}{}[shape=point]",i,symbol).unwrap();
                    writeln!(&mut f, "  s{}->s{}{}[label=\"{}{}\"]",i,i,symbol,symbol,announcements).unwrap();
                    for (pos,des) in &tr.destinations {
                        writeln!(&mut f, "  s{}{}->s{} [label =\"{}\"]",i,symbol,des,pos.to_string()).unwrap();
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
        .args(["-Tsvg", &format!("{}.dot",f_name), &format!("-o {}.svg",f_name)])
        .output()
        .expect("failed to execute process");
}