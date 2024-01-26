use std::fmt;

use ahash::HashSet;
use mcrl2::{aterm::{ATermRef, ATermTrait}, data::{is_data_application, is_data_function_symbol, is_data_variable, DataExpressionRef, DataFunctionSymbolRef, DataVariableRef}};
use sabre::RewriteSpecification;


/// Finds all data symbols in the term and adds them to the symbol index.
fn find_variables(t: &DataExpressionRef<'_>, variables: &mut HashSet<String>) {
    
    let t: ATermRef<'_> = t.copy().into();
    for child in t.iter() {
        if is_data_variable(&child) {
            variables.insert(DataVariableRef::from(child.copy()).name());
        }
    }
}

pub struct SimpleTermFormatter<'a> {
    term: ATermRef<'a>,
}

impl SimpleTermFormatter<'_> {
    pub fn new<'b>(term: &'b ATermRef<'b>) -> SimpleTermFormatter<'b> {
        SimpleTermFormatter { term: term.copy() }
    }
}

impl<'a> fmt::Display for SimpleTermFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if is_data_function_symbol(&self.term) {
            write!(f, "{}", DataFunctionSymbolRef::from(self.term.copy()).name())
        } else if is_data_application(&self.term) {
            let mut args = self.term.arguments();
    
            let head = args.next().unwrap();
            write!(f, "{}", SimpleTermFormatter::new(&head))?;
    
            let mut first = true;
            for arg in args {
                if !first {
                    write!(f, ", ")?;
                } else {
                    write!(f, "(")?;
                }
    
                write!(f, "{}", SimpleTermFormatter::new(&arg))?;
                first = false;
            }
    
            if !first {
                write!(f, ")")?;
            }

            Ok(())
        } else if is_data_variable(&self.term) {
            write!(f, "{}", DataVariableRef::from(self.term.copy()).name())
        } else {
            write!(f, "{}", self.term)
        }
    }
}

pub struct TrsFormatter<'a> {
    spec: &'a RewriteSpecification
}

impl TrsFormatter<'_> {
    pub fn new(spec: &RewriteSpecification) -> TrsFormatter {
        TrsFormatter { spec }
    }
}

impl<'a> fmt::Display for TrsFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        // Find all the variables in the specification
        let variables = {
            let mut variables = HashSet::default();

            for rule in &self.spec.rewrite_rules {
                find_variables(&rule.lhs.copy(), &mut variables);
                find_variables(&rule.rhs.copy(), &mut variables);

                for cond in &rule.conditions {
                    find_variables(&cond.lhs.copy(), &mut variables);
                    find_variables(&cond.rhs.copy(), &mut variables);
                }
            }

            variables
        };

        // Print the list of variables.
        writeln!(f, "(VAR ")?;
        for var in variables {
            writeln!(f, "\t {} ", var)?;

        }
        writeln!(f, ") ")?;

        // Print the list of rules.
        writeln!(f, "(RULES ")?;
        for rule in &self.spec.rewrite_rules {
            
            let mut output = format!("\t {} -> {}", SimpleTermFormatter::new(&rule.lhs), SimpleTermFormatter::new(&rule.rhs));
            for cond in &rule.conditions {
                if cond.equality {
                    output += &format!(" COND ==({},{}) -> true", SimpleTermFormatter::new(&cond.lhs), SimpleTermFormatter::new(&cond.rhs))
                } else {                        
                    output += &format!(" COND !=({},{}) -> true", SimpleTermFormatter::new(&cond.lhs), SimpleTermFormatter::new(&cond.rhs))
                };
            }

            writeln!(f, "{}", output.replace("|", "bar").replace("=", "eq").replace("COND", "|"))?;
        }
        writeln!(f, ")")?;

        Ok(())
    }
}