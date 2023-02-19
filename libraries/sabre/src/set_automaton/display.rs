use std::process::Command;
use crate::set_automaton::{SetAutomaton, State, Transition, EnhancedMatchAnnouncement};
use std::fs::File;
use std::io::Write;
use core::fmt;
use term_pool::TermPool;

impl fmt::Debug for SetAutomaton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.to_string())
    }
}

//To print a transition we also need the term pool
impl Transition {
    fn with_term_pool<'a>(&'a self, term_pool: &'a TermPool) -> TransitionWithTermPool<'a> {
        TransitionWithTermPool {
            transition: self,
            term_pool,
        }
    }
}
struct TransitionWithTermPool<'a> {
    transition: &'a Transition,
    term_pool: &'a TermPool
}
//Implement display for a transition with a term pool
impl<'a> fmt::Display for TransitionWithTermPool<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Transition {{ Symbol: {}, Outputs: [", self.term_pool.symbol_to_string(self.transition.symbol))?;
        for r in &self.transition.announcements {
            write!(f, "{}", AnnouncementWithTermPool{announcement:r,term_pool:self.term_pool})?;
        }
        write!(f,"] Destinations: [")?;
        for (pos,s) in &self.transition.destinations {
            write!(f,"({},{})  ", pos.to_string(), s)?;
        }
        write!(f,"]}}")
    }
}

//To print a match announcement we also need the term pool
struct AnnouncementWithTermPool<'a> {
    announcement: &'a EnhancedMatchAnnouncement,
    term_pool: &'a TermPool
}
//Implement display for a match announcement with a term pool
impl<'a> fmt::Display for AnnouncementWithTermPool<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}, [",self.term_pool.get_syntax_tree(&self.announcement.announcement.rule.lhs),self.announcement.announcement.position.to_string())?;
        for ec in &self.announcement.equivalence_classes {
            write!(f, "{}{{", self.term_pool.symbol_to_string(ec.variable))?;
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
        for (i,c) in self.announcement.announcement.rule.conditions.iter().enumerate() {
            if i != 0 {write!(f, ", ")?;}
            let comparison_symbol = if c.equality {
                "=="
            } else {
                "<>"
            };
            write!(f, "{} {} {}",self.term_pool.get_syntax_tree(&c.lhs),comparison_symbol,self.term_pool.get_syntax_tree(&c.rhs))?;
        }
        write!(f,"]")
    }
}
impl EnhancedMatchAnnouncement {
    pub fn to_string(&self,tp: &TermPool) -> String {
        format!("{}",AnnouncementWithTermPool{announcement:&self,term_pool:tp})
    }
}

//We need a term pool to print a state
impl State {
    fn with_term_pool<'a>(&'a self, term_pool: &'a TermPool) -> StateWithTermPool<'a> {
        StateWithTermPool {
            state: self,
            term_pool,
        }
    }
}
struct StateWithTermPool<'a> {
    state: &'a State,
    term_pool: &'a TermPool
}
//Implement display for a state with a term pool
impl<'a> fmt::Display for StateWithTermPool<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "State {{Label: {}, \n Transitions: [", self.state.label.to_string())?;
        for t in &self.state.transitions {
            write!(f, "{}\n",t.with_term_pool(self.term_pool))?;
        }
        write!(f, "],\n Match goals: [\n")?;
        for m in &self.state.match_goals {
            write!(f, "\t Obligations:")?;
            for mo in &m.obligations {
                write!(f, "{}@{}  ", self.term_pool.get_syntax_tree(&mo.pattern), mo.position.to_string())?;
                //write!(f, "p@{}  ",mo.position.to_string())?;
            }
            write!(f, "Announcement: {}@{}, ", self.term_pool.get_syntax_tree(&m.announcement.rule.lhs), m.announcement.position.to_string())?;
            write!(f, "Conditions: ")?;
            for c in &m.announcement.rule.conditions {
                let comparison_symbol = if c.equality {
                    "=="
                } else {
                    "<>"
                };
                write!(f, "{} {} {}",self.term_pool.get_syntax_tree(&c.lhs),comparison_symbol,self.term_pool.get_syntax_tree(&c.rhs))?;
            }
            write!(f, "\n")?;
            //write!(f, "Announcement: p@{} \n", m.announcement.position.to_string())?;
        }
        write!(f, "]}}")
    }
}

impl fmt::Display for SetAutomaton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "States: {{\n")?;
        let mut state_index = 0;
        for s in &self.states {
            write!(f, "State {}: {} \n",state_index, s.with_term_pool(&self.term_pool))?;
            state_index += 1;
        }
        write!(f, "}}\n")
    }
}

impl SetAutomaton {
    //Create a .dot file and convert it to a .svg using the dot command
    pub fn to_dot_graph(&self, f_base_name: &str) {
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
                    match_goals += &*format!("{}@{}", self.term_pool.get_syntax_tree(&mo.pattern),mo.position);
                }
                match_goals += &*format!(" 👉 {}-\\>{}@{}, {}\\l", self.term_pool.get_syntax_tree(&mg.announcement.rule.lhs),self.term_pool.get_syntax_tree(&mg.announcement.rule.rhs),mg.announcement.position,mg.announcement.symbols_seen);
            }
            writeln!(&mut f, "  s{}[shape=record label=\"{{{{s{} | {}}} | {}}}\"]",i,i,s.label.to_string(),match_goals).unwrap();
        }
        for (i,s) in self.states.iter().enumerate() {
            for (symbol,tr) in s.transitions.iter().enumerate() {
                let symbol = self.term_pool.symbol_to_string(symbol);
                let mut announcements = "".to_string();
                for a in &tr.announcements {
                    announcements += &format!(", {}@{}",self.term_pool.get_syntax_tree(&a.announcement.rule.lhs),a.announcement.position);
                }
                if tr.destinations.is_empty() {
                    writeln!(&mut f, "  s{}-> final [label=\"{}{}\"]",i,symbol,announcements).unwrap();
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