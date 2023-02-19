use core::fmt;
use std::borrow::BorrowMut;
use std::hash::Hash;

use ahash::AHashMap as HashMap;

use crate::utilities::position::ExplicitPosition;
use mcrl2_rust::atermpp::ATerm;

/// A pattern is simply an aterm of the shape f(...)
pub type Pattern = ATerm;

/// A condition stored in the term pool
#[derive(Hash, Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Condition 
{
    pub lhs: Pattern,
    pub rhs: Pattern,
    pub equality: bool
}

/// Rewrite rule \bigwedge condition -> lhs = rhs in mCRL2 syntax.
#[derive(Hash, Clone, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct RewriteRule {
    pub lhs: Pattern,
    pub rhs: Pattern,
    pub conditions: Vec<Condition>
}

/// A rewrite specification contains all the info we need for rewriting.
#[derive(Debug, Clone)]
pub struct RewriteSpecification
{
    pub rewrite_rules: Vec<RewriteRule>,
    pub symbols: Vec<String>,
    pub arity_per_symbol: HashMap<String,usize>
}

impl RewriteSpecification 
{
    pub fn new() -> Self {
        RewriteSpecification {
            rewrite_rules: vec![],
            symbols: vec![],
            arity_per_symbol: HashMap::default()
        }
    }
}

impl fmt::Display for RewriteSpecification 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbols: \n")
        /*for (symbol, arity) in &self.arity_per_symbol {
            write!(f, "{}: {}\n",symbol,arity)?;
        }
        write!(f, "Rewrite rules: \n")?;
        for rule in &self.rewrite_rules {
            write!(f, "{}\n", rule)?;
        }
        write!(f, "\n")*/
    }
}


#[cfg(test)]
mod tests
{
    use super::*;    
    
    // Test the iterator implementation.
    #[test]
    fn parse_mcrl2_data_specification()
    {

        
    }
}
