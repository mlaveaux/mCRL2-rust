use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use ahash::AHashMap as HashMap;

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::syntax::{
    ConditionSyntax, RewriteRuleSyntax, RewriteSpecificationSyntax, TermSyntaxTree,
};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct TermParser;

/// Parses a REC specification. REC files can import other REC files.
/// Returns a RewriteSpec containing all the rewrite rules and a list of terms that need to be rewritten.
///
/// Arguments
/// `contents` - A REC specification as string
/// `path` - An optional path to a folder in which other importable REC files can be found.
#[allow(non_snake_case)]
fn parse_REC(
    contents: &str,
    path: Option<PathBuf>,
) -> (RewriteSpecificationSyntax, Vec<TermSyntaxTree>) {
    // Initialise return result
    let mut rewrite_spec = RewriteSpecificationSyntax::default();
    let mut terms = vec![];

    // Use Pest parser (generated automatically from the grammar.pest file)
    match TermParser::parse(Rule::rec_spec, contents) {
        Ok(mut pairs) => {
            // Get relevant sections from the REC file
            let pair = pairs.next().unwrap();
            let mut inner = pair.into_inner();
            let header = inner.next().unwrap();
            let _sorts = inner.next().unwrap();
            let cons = inner.next().unwrap();
            let opns = inner.next().unwrap();
            let vars = inner.next().unwrap();
            let rules = inner.next().unwrap();
            let eval = inner.next().unwrap();
            let (_name, include_files) = parse_header(header);

            // REC files can import other REC files. Import all referenced by the header.
            for file in include_files {
                if let Some(p) = &path {
                    let include_path = p.parent().unwrap();
                    let file_name = PathBuf::from_str(&(file.to_lowercase() + ".rec")).unwrap();
                    let load_file = include_path.join(file_name);
                    let contents = fs::read_to_string(load_file).unwrap();
                    let (include_spec, include_terms) = parse_REC(&contents, path.clone());

                    // Add rewrite rules and terms to the result.
                    terms.extend_from_slice(&include_terms);
                    rewrite_spec
                        .arity_per_symbol
                        .extend(include_spec.arity_per_symbol);
                    rewrite_spec
                        .rewrite_rules
                        .extend_from_slice(&include_spec.rewrite_rules);
                    for s in include_spec.variables {
                        if !rewrite_spec.variables.contains(&s) {
                            rewrite_spec.variables.push(s);
                        }
                    }
                }
            }
            let arity_per_symbol_cons = parse_constructors(cons);
            let arity_per_symbol_opns = parse_operations(opns);
            rewrite_spec
                .rewrite_rules
                .extend_from_slice(&parse_rewrite_rules(rules));
            if eval.as_rule() == Rule::eval {
                terms.extend_from_slice(&parse_eval(eval));
            }

            rewrite_spec.variables = parse_variables(vars);

            rewrite_spec.arity_per_symbol.extend(arity_per_symbol_cons);
            rewrite_spec.arity_per_symbol.extend(arity_per_symbol_opns);
        }
        Err(e) => {
            // TODO: Panic when a parse error occurs. Should probably return an error.
            println!("{}", e);
            panic!("Failed to load rewrite system");
        }
    }
    (rewrite_spec, terms)
}

/// Load a REC specification from a specified file.
#[allow(non_snake_case, dead_code)]
pub fn load_REC_from_file(file: PathBuf) -> (RewriteSpecificationSyntax, Vec<TermSyntaxTree>) {
    let contents = fs::read_to_string(file.clone()).unwrap();
    parse_REC(&contents, Some(file))
}

/// Load and join multiple REC specifications
#[allow(non_snake_case)]
pub fn load_REC_from_strings(
    specs: Vec<&str>,
) -> (RewriteSpecificationSyntax, Vec<TermSyntaxTree>) {
    let mut rewrite_spec = RewriteSpecificationSyntax::default();
    let mut terms = vec![];

    for spec in specs {
        let (include_spec, include_terms) = parse_REC(spec, None);

        terms.extend_from_slice(&include_terms);
        rewrite_spec
            .arity_per_symbol
            .extend(include_spec.arity_per_symbol);
        rewrite_spec
            .rewrite_rules
            .extend_from_slice(&include_spec.rewrite_rules);

        for s in include_spec.variables {
            if !rewrite_spec.variables.contains(&s) {
                rewrite_spec.variables.push(s);
            }
        }
    }

    (rewrite_spec, terms)
}

/// Extracts data from parsed header of REC spec. Returns name and include files.
fn parse_header(pair: Pair<Rule>) -> (String, Vec<String>) {
    assert_eq!(pair.as_rule(), Rule::header);

    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let mut include_files = vec![];

    for f in inner {
        include_files.push(f.as_str().to_string());
    }

    (name, include_files)
}

/// Extracts data from parsed constructor section, derives the arity of symbols. Types are ignored.
fn parse_constructors(pair: Pair<Rule>) -> HashMap<String, usize> {
    assert_eq!(pair.as_rule(), Rule::cons);
    let mut arity_per_symbol = HashMap::new();
    for decl in pair.into_inner() {
        assert_eq!(decl.as_rule(), Rule::cons_decl);
        let mut inner = decl.into_inner();
        let symbol = inner.next().unwrap().as_str().to_string();
        let arity = inner.count() - 1;
        arity_per_symbol.insert(symbol, arity);
    }
    arity_per_symbol
}

/// Extracts data from parsed operations section, derives additional arity of symbols. Types are ignored.
fn parse_operations(pair: Pair<Rule>) -> HashMap<String, usize> {
    assert_eq!(pair.as_rule(), Rule::opns);
    let mut arity_per_symbol = HashMap::new();
    for decl in pair.into_inner() {
        assert_eq!(decl.as_rule(), Rule::opn_decl);
        let mut inner = decl.into_inner();
        let symbol = inner.next().unwrap().as_str().to_string();
        let arity = inner.count() - 1;
        arity_per_symbol.insert(symbol, arity);
    }
    arity_per_symbol
}

/// Extracts data from parsed rewrite rules. Returns list of rewrite rules
fn parse_rewrite_rules(pair: Pair<Rule>) -> Vec<RewriteRuleSyntax> {
    assert_eq!(pair.as_rule(), Rule::rules);
    let mut rules = vec![];
    let inner = pair.into_inner();
    for p in inner {
        let rule = parse_rewrite_rule(p);
        rules.push(rule);
    }
    rules
}

// Extracts data from the variable VARS block. Types are ignored.
fn parse_variables(pair: Pair<Rule>) -> Vec<String> {
    assert_eq!(pair.as_rule(), Rule::vars);

    let mut variables = vec![];
    let inner = pair.into_inner();
    for v in inner {
        assert_eq!(v.as_rule(), Rule::var_decl);

        variables.push(String::from(v.into_inner().as_str()));
    }

    variables
}

/// Extracts data from parsed EVAL section, returns a list of terms that need to be rewritten.
fn parse_eval(pair: Pair<Rule>) -> Vec<TermSyntaxTree> {
    assert_eq!(pair.as_rule(), Rule::eval);
    let mut terms = vec![];
    let inner = pair.into_inner();
    for p in inner {
        let term = parse_term(p);
        terms.push(term);
    }
    terms
}

/// Constructs a TermSyntaxTree from a string.
impl TermSyntaxTree {
    pub fn from_string(str: &str) -> Result<TermSyntaxTree, pest::error::Error<Rule>> {
        let mut pairs = TermParser::parse(Rule::single_term, str)?;
        Ok(parse_term(
            pairs.next().unwrap().into_inner().next().unwrap(),
        ))
    }
}

/// Extracts data from parsed term.
fn parse_term(pair: Pair<Rule>) -> TermSyntaxTree {
    assert_eq!(pair.as_rule(), Rule::term);
    match pair.as_rule() {
        Rule::term => {
            let mut inner = pair.into_inner();
            let head_symbol = inner.next().unwrap().as_str().to_string().replace(' ', "");
            let mut sub_terms = vec![];
            if let Some(args) = inner.next() {
                for arg in args.into_inner() {
                    sub_terms.push(parse_term(arg));
                }
            }
            TermSyntaxTree {
                head_symbol,
                sub_terms,
            }
        }
        _ => {
            panic!("Should be unreachable!")
        }
    }
}

// /Extracts data from parsed rewrite rule
fn parse_rewrite_rule(pair: Pair<Rule>) -> RewriteRuleSyntax {
    assert!(pair.as_rule() == Rule::single_rewrite_rule || pair.as_rule() == Rule::rewrite_rule);

    let mut inner = match pair.as_rule() {
        Rule::single_rewrite_rule => pair.into_inner().next().unwrap().into_inner(),
        Rule::rewrite_rule => pair.into_inner(),
        _ => {
            panic!("Unreachable");
        }
    };
    let lhs = parse_term(inner.next().unwrap());
    let rhs = parse_term(inner.next().unwrap());

    // Extract conditions
    let mut conditions = vec![];
    for c in inner {
        assert_eq!(c.as_rule(), Rule::condition);
        let mut c_inner = c.into_inner();
        let lhs_cond = parse_term(c_inner.next().unwrap());
        let equality = match c_inner.next().unwrap().as_str() {
            "=" => true,
            "<>" => false,
            _ => {
                panic!("Unknown comparison operator");
            }
        };
        let rhs_cond = parse_term(c_inner.next().unwrap());

        let condition = ConditionSyntax {
            lhs: lhs_cond,
            rhs: rhs_cond,
            equality,
        };
        conditions.push(condition);
    }
    RewriteRuleSyntax {
        lhs,
        rhs,
        conditions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_parsing() {
        assert!(TermParser::parse(Rule::single_term, "f(a").is_err());
        assert!(TermParser::parse(Rule::single_term, "f()").is_err());
        assert!(TermParser::parse(Rule::single_term, "f(a,)").is_err());
        assert!(TermParser::parse(Rule::single_term, "f").is_ok());
        assert!(TermParser::parse(Rule::single_term, "f(a)").is_ok());
        assert!(TermParser::parse(Rule::single_term, "f(a,b)").is_ok());
        assert!(TermParser::parse(Rule::single_rewrite_rule, "f(a,b) = g(x)").is_ok());
        assert!(TermParser::parse(Rule::single_rewrite_rule, "f(a,b) = g(x) if x = a").is_ok());
        assert!(TermParser::parse(Rule::single_rewrite_rule, "f(a,b) = g(x) if x<> a").is_ok());
        assert!(TermParser::parse(Rule::single_rewrite_rule, "f(a,b) = g(x) if x <= a").is_err());
        assert!(TermParser::parse(Rule::single_rewrite_rule, "f(a,b) = ").is_err());
    }

    #[test]
    fn test_parsing_rewrite_rule() {
        let expected = RewriteRuleSyntax {
            lhs: TermSyntaxTree::from_string("f(x,b)").unwrap(),
            rhs: TermSyntaxTree::from_string("g(x)").unwrap(),
            conditions: vec![
                ConditionSyntax {
                    lhs: TermSyntaxTree::from_string("x").unwrap(),
                    rhs: TermSyntaxTree::from_string("a").unwrap(),
                    equality: true,
                },
                ConditionSyntax {
                    lhs: TermSyntaxTree::from_string("b").unwrap(),
                    rhs: TermSyntaxTree::from_string("b").unwrap(),
                    equality: true,
                },
            ],
        };
        let actual = parse_rewrite_rule(
            TermParser::parse(
                Rule::single_rewrite_rule,
                "f(x,b) = g(x) if x = a and-if b = b",
            )
            .unwrap()
            .next()
            .unwrap(),
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parsing_rec() {
        assert!(TermParser::parse(Rule::rec_spec, include_str!("missionaries.rec")).is_ok());
    }

    #[test]
    fn loading_rec() {
        let _ = parse_REC(include_str!("missionaries.rec"), None);
    }
}
