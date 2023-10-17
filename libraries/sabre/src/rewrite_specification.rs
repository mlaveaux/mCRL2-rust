use std::fmt;

use itertools::Itertools;
use mcrl2::{
    atermpp::ATerm,
    data::{DataFunctionSymbol, DataSpecification, BoolSort}
};

/// A rewrite specification contains the bare info we need for rewriting (in particular no type information).
#[derive(Debug,Clone)]
pub struct RewriteSpecification 
{
    pub rewrite_rules: Vec<Rule>,
    pub constructors: Vec<(DataFunctionSymbol, usize)>,
}

/// Either lhs == rhs or lhs != rhs depending on equality being true.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Condition
{
    pub lhs: ATerm,
    pub rhs: ATerm,
    pub equality: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Rule
{
    /// A conjunction of clauses
    pub conditions: Vec<Condition>, 
    pub lhs: ATerm,
    pub rhs: ATerm
}

impl From<DataSpecification> for RewriteSpecification {
    fn from(value: DataSpecification) -> Self {
        let equations = value.equations();

        // Convert the equations.
        let mut rewrite_rules = vec![];
        for equation in equations {
            rewrite_rules.push(Rule {
                conditions: vec![
                    Condition {
                        lhs: equation.condition,
                        rhs: BoolSort::true_term().into(),
                        equality: true
                    }
                ],
                lhs: equation.lhs,
                rhs: equation.rhs
            })
        }
        
        RewriteSpecification { rewrite_rules, constructors: vec![] }
    }
}

impl fmt::Display for RewriteSpecification 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        writeln!(f, "Rewrite rules: ")?;
        for rule in &self.rewrite_rules 
        {
            writeln!(f, "{}", rule)?;
        }
        writeln!(f)
    }
}

impl fmt::Display for Rule 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        if self.conditions.is_empty() {
            write!(f, "{} = {}", self.lhs, self.rhs)
        } else {
            write!(f, "{} -> {} = {}", self.conditions.iter().format(", "), self.lhs, self.rhs)
        }
    }
}

impl fmt::Display for Condition
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        if self.equality {
            write!(f, "{} == {}", self.lhs, self.rhs)
        } else {
            write!(f, "{} <> {}", self.lhs, self.rhs)
        }
    }
}