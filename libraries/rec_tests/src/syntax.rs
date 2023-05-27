use core::fmt;
use std::hash::Hash;

use ahash::{AHashMap as HashMap, AHashSet};
use mcrl2_rust::atermpp::{ATerm, TermFunctionSymbol, TermPool};
use sabre::{
    rewrite_specification::{Condition, RewriteSpecification, Rule},
    utilities::{to_data_expression, ExplicitPosition},
};

/// A rewrite specification contains all the bare info we need for rewriting (in particular no type information) as a syntax tree.
/// Parsing a REC file results in a RewriteSpecificationSyntax.
#[derive(Clone, Default, Debug)]
pub struct RewriteSpecificationSyntax {
    pub rewrite_rules: Vec<RewriteRuleSyntax>,
    pub variables: Vec<String>,
    pub arity_per_symbol: HashMap<String, usize>,
}

impl RewriteSpecificationSyntax {

    pub fn to_rewrite_spec(&self, tp: &mut TermPool) -> RewriteSpecification {
        println!("specification: {}", self);

        // The names for all variables
        let variables = AHashSet::from_iter(self.variables.clone());

        // Store the rewrite rules in the maximally shared term storage
        let mut rewrite_rules = Vec::new();
        for rr in &self.rewrite_rules {

            // Convert the conditions.
            let mut conditions = vec![];
            for c in &rr.conditions {
                let lhs_term = c.lhs.to_term(tp);
                let rhs_term = c.rhs.to_term(tp);

                let lhs_cond = to_data_expression(tp, &lhs_term, &variables);
                let rhs_cond = to_data_expression(tp, &rhs_term, &variables);
                let condition = Condition {
                    lhs: to_data_expression(tp, &lhs_cond, &variables),
                    rhs: to_data_expression(tp, &rhs_cond, &variables),
                    equality: c.equality,
                };
                conditions.push(condition);
            }
            
            let lhs = rr.lhs.to_term(tp);
            let rhs = rr.rhs.to_term(tp);

            rewrite_rules.push(Rule {
                lhs: to_data_expression(tp, &lhs, &variables),
                rhs: to_data_expression(tp, &rhs, &variables),
                conditions,
            });
        }

        // Find the indices of all the function symbols.
        let mut symbols = vec![];

        for rule in &rewrite_rules {

            let mut iter: Box<dyn Iterator<Item = ATerm>> =
                Box::new(rule.lhs.iter().chain(rule.rhs.iter()));
            for cond in &rule.conditions {
                iter = Box::new(iter.chain(cond.rhs.iter().chain(cond.lhs.iter())));
            }

            for subterm in iter {
                if subterm.is_function_symbol() {
                    let index = subterm.operation_id();
                    symbols.resize(index + 1, TermFunctionSymbol::default());
                    symbols[index] = subterm.into();
                }
            }
        }

        println!("symbols: {:?}", symbols);

        RewriteSpecification {
            rewrite_rules,
            symbols,
        }
    }
}

impl fmt::Display for RewriteSpecificationSyntax {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Symbols: ")?;
        for (symbol, arity) in &self.arity_per_symbol {
            writeln!(f, "{}: {}", symbol, arity)?;
        }
        writeln!(f, "Rewrite rules: ")?;
        for rule in &self.rewrite_rules {
            writeln!(f, "{}", rule)?;
        }
        writeln!(f)
    }
}

/// A TermSyntaxTrees stores a term in a tree structure. They are not used in expensive computations.
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(C)]
pub struct TermSyntaxTree {
    pub head_symbol: String,
    pub sub_terms: Vec<TermSyntaxTree>,
}

impl TermSyntaxTree {
    /// Get the subtree at a given position. Panics if that subtree does not exists.
    pub fn get_position(&self, p: &ExplicitPosition) -> &TermSyntaxTree {
        // Start with the root
        let mut sub_term = self;

        for x in &p.indices {
            sub_term = sub_term.sub_terms.get(*x - 1).unwrap();
        }

        sub_term
    }

    /// Converts the syntax tree into a maximally shared [ATerm].
    pub fn to_term(&self, tp: &mut TermPool) -> ATerm {
        // Create an ATerm with as arguments all the evaluated semi compressed term trees.
        let mut subterms = Vec::with_capacity(self.sub_terms.len());

        for argument in self.sub_terms.iter() {
            subterms.push(argument.to_term(tp));
        }

        let head = tp.create_symbol(&self.head_symbol, self.sub_terms.len());
        tp.create(&head, &subterms)
    }
}

/// Pretty prints TermSyntaxTrees. Sample output: and(true, false).
impl fmt::Display for TermSyntaxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.head_symbol.clone())?;
        if !self.sub_terms.is_empty() {
            write!(f, "(")?;
        }
        let mut first = true;
        for sub in &self.sub_terms {
            if first {
                sub.fmt(f)?;
                first = false;
            } else {
                write!(f, ",")?;
                sub.fmt(f)?;
            }
        }
        if !self.sub_terms.is_empty() {
            write!(f, ")")?;
        }
        Ok(())
    }
}

/// Syntax tree for rewrite rules
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RewriteRuleSyntax {
    pub lhs: TermSyntaxTree,
    pub rhs: TermSyntaxTree,
    pub conditions: Vec<ConditionSyntax>,
}

impl fmt::Display for RewriteRuleSyntax {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.lhs, self.rhs)
    }
}

/// Syntax tree for conditional part of a rewrite rule
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConditionSyntax {
    pub lhs: TermSyntaxTree,
    pub rhs: TermSyntaxTree,
    pub equality: bool, // The condition either specifies that lhs and rhs are equal or different
}
