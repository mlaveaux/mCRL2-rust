use std::fmt;

use mcrl2_rust::atermpp::{ATerm, Symbol};

/// A rewrite specification contains the bare info we need for rewriting (in particular no type information).
#[derive(Debug,Clone)]
pub struct RewriteSpecification 
{
    pub rewrite_rules: Vec<Rule>,
    pub symbols: Vec<Symbol>,
}

/// Either lhs == rhs or lhs != rhs depending on equality being true.
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Condition
{
    pub lhs: ATerm,
    pub rhs: ATerm,
    pub equality: bool,
}

#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
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
        write!(f, "Symbols: \n")?;
        for symbol in &self.symbols 
        {
            write!(f, "{}: {}\n", symbol, symbol.arity())?;
        }

        write!(f, "Rewrite rules: \n")?;
        for rule in &self.rewrite_rules 
        {
            write!(f, "{}\n", rule)?;
        }
        write!(f, "\n")
    }
}

impl fmt::Display for Rule 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        //write!(f, "{}: {}\n", symbol, symbol.arity())?;
        write!(f, "")
    }
}