use core::fmt;
use std::hash::Hash;

use ahash::AHashMap as HashMap;
use mcrl2_rust::atermpp::{ATerm, TermPool};
use sabre::{utilities::ExplicitPosition, rewrite_specification::{RewriteSpecification, Rule, Condition}};

/// A rewrite specification contains all the bare info we need for rewriting (in particular no type information) as a syntax tree.
/// Parsing a REC file results in a RewriteSpecificationSyntax.
#[derive(Debug,Clone)]
pub struct RewriteSpecificationSyntax 
{
    pub rewrite_rules: Vec<RewriteRuleSyntax>,
    pub symbols: Vec<String>,
    pub arity_per_symbol: HashMap<String,usize>
}

impl RewriteSpecificationSyntax 
{
    pub fn new() -> Self 
    {
        RewriteSpecificationSyntax {
            rewrite_rules: vec![],
            symbols: vec![],
            arity_per_symbol: HashMap::default()
        }
    }

    pub fn to_rewrite_spec(&self, tp: &mut TermPool) -> RewriteSpecification
    {        
      // Store the rewrite rules in the maximally shared term storage
      let mut rewrite_rules = Vec::new();
      for rr in &self.rewrite_rules 
      {
          let lhs = rr.lhs.to_term(tp);
          let rhs  = rr.rhs.to_term(tp);

          // Convert the conditions.
          let mut conditions = vec![];
          for c in &rr.conditions 
          {
              let lhs_cond = c.lhs.to_term(tp);
              let rhs_cond = c.rhs.to_term(tp);
              let condition = Condition {
                  lhs: lhs_cond,
                  rhs: rhs_cond,
                  equality: c.equality
              };
              conditions.push(condition);
          }
          rewrite_rules.push(Rule { lhs, rhs, conditions });
      }

      RewriteSpecification { rewrite_rules, symbols: vec![] } 
    }
}

impl fmt::Display for RewriteSpecificationSyntax 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
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

/// A TermSyntaxTrees stores a term in a tree structure. They are not used in expensive computations.
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(C)]
pub struct TermSyntaxTree 
{
    pub head_symbol: String,
    pub sub_terms: Vec<TermSyntaxTree>
}

impl TermSyntaxTree 
{
    /// Get the subtree at a given position. Panics if that subtree does not exists.
    pub fn get_position(&self, p: &ExplicitPosition) -> &TermSyntaxTree 
    {
        // Start with the root
        let mut sub_term = self;

        for x in &p.indices 
        {
            sub_term = sub_term.sub_terms.get(*x as usize - 1).unwrap();
        }

        sub_term
    }

    /// Converts the syntax tree into a maximally shared [ATerm].
    pub fn to_term(&self, tp: &mut TermPool) -> ATerm
    {        
        // Create an ATerm with as arguments all the evaluated semi compressed term trees.              
        let mut subterms = Vec::with_capacity(self.sub_terms.len());

        for argument in self.sub_terms.iter()
        {
            subterms.push(argument.to_term(tp));
        }

        let head = tp.create_symbol(&self.head_symbol, self.sub_terms.len());
        tp.create(&head, &subterms)
    }
}

/// Pretty prints TermSyntaxTrees. Sample output: and(true, false).
impl fmt::Display for TermSyntaxTree 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        write!(f, "{}", self.head_symbol.clone())?;
        if !self.sub_terms.is_empty() {write!(f, "(")?;}
        let mut first = true;
        for sub in &self.sub_terms {
            if first{
                sub.fmt(f)?;
                first = false;
            } else {
                write!(f, ",")?;
                sub.fmt(f)?;
            }
        }
        if !self.sub_terms.is_empty() {write!(f, ")")?;}
        Ok(())
    }
}

/// Syntax tree for rewrite rules
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RewriteRuleSyntax 
{
    pub lhs: TermSyntaxTree,
    pub rhs: TermSyntaxTree,
    pub conditions: Vec<ConditionSyntax>
}

impl fmt::Display for RewriteRuleSyntax 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        write!(f, "{} -> {}", self.lhs, self.rhs)
    }
}

/// Syntax tree for conditional part of a rewrite rule
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConditionSyntax 
{
    pub lhs: TermSyntaxTree,
    pub rhs: TermSyntaxTree,
    pub equality: bool // The condition either specifies that lhs and rhs are equal or different
}