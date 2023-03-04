use mcrl2_rust::atermpp::{ATerm, Symbol};

/// A rewrite specification contains the bare info we need for rewriting (in particular no type information).
#[derive(Debug,Clone)]
pub struct RewriteSpecification 
{
    pub rewrite_rules: Vec<Rule>,
    pub symbols: Vec<Symbol>,
}

#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
/// Either lhs == rhs or lhs != rhs.
pub struct Condition
{
    pub lhs: ATerm,
    pub rhs: ATerm,
    pub equality: bool,
}

#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Rule
{
    pub conditions: Vec<Condition>, // A conjunction of clauses.
    pub lhs: ATerm,
    pub rhs: ATerm
}

/*
impl fmt::Display for RewriteSpec 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbols: \n")?;
        for (symbol, arity) in &self.arity_per_symbol {
            write!(f, "{}: {}\n",symbol,arity)?;
        }
        write!(f, "Rewrite rules: \n")?;
        for rule in &self.rewrite_rules {
            write!(f, "{}\n", rule)?;
        }
        write!(f, "\n")
    }
}
*/