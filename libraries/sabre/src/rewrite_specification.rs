use std::fmt;

use mcrl2_rust::{
    atermpp::ATerm,
    data::DataFunctionSymbol
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
        write!(f, "{:?} -> {} = {}\n", self.conditions, self.lhs, self.rhs)
    }
}