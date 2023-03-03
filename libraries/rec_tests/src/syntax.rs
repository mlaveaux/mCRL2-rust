//! This crate offers several features to handle terms.
//!
//! REC, short for Rewriting Engine Competition, is a format for specifying rewrite systems.
//! The parser module contains functions for loading a REC file.
//!
//! 
use core::fmt;
use std::hash::Hash;

use ahash::AHashMap as HashMap;
use mcrl2_rust::atermpp::ATerm;
use sabre::utilities::ExplicitPosition;

/// A rewrite spec contains all the bare info we need for rewriting (so no type information).
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
    pub fn new() -> Self {
        RewriteSpecificationSyntax {
            rewrite_rules: vec![],
            symbols: vec![],
            arity_per_symbol: HashMap::default()
        }
    }
}

impl fmt::Display for RewriteSpecificationSyntax 
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
    ///Get the subtree at a given position. Panics if that subtree does not exists.
    pub fn get_position(&self, p: &ExplicitPosition) -> &TermSyntaxTree {
        let mut sub_term = self;
        for x in &p.indices {
            sub_term = sub_term.sub_terms.get(*x as usize -1).unwrap();
        }
        sub_term
    }
}
//Pretty prints TermSyntaxTrees. Sample output: and(true, false).
impl fmt::Display for TermSyntaxTree 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
pub struct RewriteRuleSyntax {
    pub lhs: TermSyntaxTree,
    pub rhs: TermSyntaxTree,
    pub conditions: Vec<ConditionSyntax>
}
impl fmt::Display for RewriteRuleSyntax {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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