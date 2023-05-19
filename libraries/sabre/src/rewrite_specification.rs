use std::fmt;

use mcrl2_rust::atermpp::{ATerm, TermFunctionSymbol};

/// A rewrite specification contains the bare info we need for rewriting (in particular no type information).
#[derive(Debug,Clone)]
pub struct RewriteSpecification 
{
    pub rewrite_rules: Vec<Rule>,
    pub symbols: Vec<TermFunctionSymbol>,
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
        writeln!(f, "Symbols: ")?;
        for symbol in &self.symbols 
        {
            writeln!(f, "{}", symbol.name())?;
        }

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
        //write!(f, "{}: {}\n", symbol, symbol.arity())?;
        write!(f, "")
    }
}