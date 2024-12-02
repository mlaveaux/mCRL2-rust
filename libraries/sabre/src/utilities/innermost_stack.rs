use std::fmt;

use itertools::Itertools;
use mcrl2::aterm::ATermRef;
use mcrl2::aterm::Protected;
use mcrl2::aterm::Protector;
use mcrl2::data::DataExpressionRef;
use mcrl2::data::DataFunctionSymbolRef;

use crate::utilities::PositionIndexed;

use super::Config;
use super::TermStack;

use log::trace;

/// This stack is used to avoid recursion and also to keep track of terms in
/// normal forms by explicitly representing the rewrites of a right hand
/// side.
#[derive(Default)]
pub struct InnermostStack {
    pub configs: Protected<Vec<Config>>,
    pub terms: Protected<Vec<DataExpressionRef<'static>>>,
}

impl InnermostStack {
    /// Updates the InnermostStack to integrate the rhs_stack instructions.
    pub fn integrate(
        write_configs: &mut Protector<Vec<Config>>,
        write_terms: &mut Protector<Vec<DataExpressionRef<'static>>>,
        rhs_stack: &TermStack,
        term: &DataExpressionRef,
        result_index: usize,
    ) {
        // TODO: This ignores the first element of the stack, but that is kind of difficult to deal with.
        let top_of_stack = write_terms.len();
        write_terms.reserve(rhs_stack.stack_size - 1); // We already reserved space for the result.
        for _ in 0..rhs_stack.stack_size - 1 {
            write_terms.push(Default::default());
        }

        let mut first = true;
        for config in rhs_stack.innermost_stack.read().iter() {
            match config {
                Config::Construct(symbol, arity, offset) => {
                    if first {
                        // The first result must be placed on the original result index.
                        InnermostStack::add_result(write_configs, symbol.copy(), *arity, result_index);
                    } else {
                        // Otherwise, we put it on the end of the stack.
                        InnermostStack::add_result(write_configs, symbol.copy(), *arity, top_of_stack + offset - 1);
                    }
                }
                Config::Term(term, index) => {
                    let term = write_configs.protect(term);
                    write_configs.push(Config::Term(term.into(), *index));
                }
                Config::Rewrite(_) => {
                    unreachable!("This case should not happen");
                }
                Config::Return() => {
                    unreachable!("This case should not happen");
                }
            }
            first = false;
        }
        trace!(
            "\t applied stack size: {}, substitution: {}, stack: [{}]",
            rhs_stack.stack_size,
            rhs_stack.variables.iter().format_with(", ", |element, f| {
                f(&format_args!("{} -> {}", element.0, element.1))
            }),
            rhs_stack.innermost_stack.read().iter().format("\n")
        );

        debug_assert!(
            rhs_stack.stack_size != 1 || rhs_stack.variables.len() <= 1,
            "There can only be a single variable in the right hand side"
        );
        if rhs_stack.stack_size == 1 && rhs_stack.variables.len() == 1 {
            // This is a special case where we place the result on the correct position immediately.
            // The right hand side is only a variable
            let t: ATermRef<'_> = write_terms.protect(&term.get_position(&rhs_stack.variables[0].0));
            write_terms[result_index] = t.into();
        } else {
            for (position, index) in &rhs_stack.variables {
                // Add the positions to the stack.
                let t = write_terms.protect(&term.get_position(position));
                write_terms[top_of_stack + index - 1] = t.into();
            }
        }
    }

    /// Indicate that the given symbol with arity can be constructed at the given index.
    pub fn add_result(
        write_configs: &mut Protector<Vec<Config>>,
        symbol: DataFunctionSymbolRef<'_>,
        arity: usize,
        index: usize,
    ) {
        let symbol = write_configs.protect(&symbol.into());
        write_configs.push(Config::Construct(symbol.into(), arity, index));
    }

    /// Indicate that the term must be rewritten and its result must be placed at the given index.
    pub fn add_rewrite(
        write_configs: &mut Protector<Vec<Config>>,
        write_terms: &mut Protector<Vec<DataExpressionRef<'static>>>,
        term: DataExpressionRef<'_>,
        index: usize,
    ) {
        let term = write_terms.protect(&term);
        write_configs.push(Config::Rewrite(index));
        write_terms.push(term.into());
    }
}

impl fmt::Display for InnermostStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Terms: [")?;
        for (i, term) in self.terms.read().iter().enumerate() {
            if !term.is_default() {
                writeln!(f, "{}\t{}", i, term)?;
            } else {
                writeln!(f, "{}\t<default>", i)?;
            }
        }
        writeln!(f, "]")?;

        writeln!(f, "Configs: [")?;
        for config in self.configs.read().iter() {
            writeln!(f, "\t{}", config)?;
        }
        write!(f, "]")
    }
}
