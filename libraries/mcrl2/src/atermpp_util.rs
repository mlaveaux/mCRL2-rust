use std::fmt;

use anyhow::Result as AnyResult;
use ahash::AHashSet;
use mcrl2_sys::atermpp::{ATerm, Symbol, TermPool};

enum Config<I> {
    Apply(I, usize),
    Construct(Symbol, usize),
}

pub struct TermArgs<'a, I> {
    terms: &'a mut Vec<ATerm>,
    tmp: &'a mut Vec<Config<I>>,
    top_of_stack: usize,
}

impl<'a, I> TermArgs<'a, I> {
    pub fn push(&mut self, input: I) {
        self.terms.push(ATerm::default());    
        self.tmp.push(Config::Apply(input, self.top_of_stack + self.tmp.len()));
    }
}

/// Can be used to construct a term bottom up using an iterative approach
pub struct TermBuilder<I>
{
    // The stack of terms
    terms: Vec<ATerm>,
    configs: Vec<Config<I>>,
    tmp: Vec<Config<I>>,
}

impl<I> fmt::Display for TermBuilder<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Terms: [")?;
        for (i, term) in self.terms.iter().enumerate() {
            writeln!(f, "{}\t{}", i, term)?;
        }
        writeln!(f, "]")?;
        
        writeln!(f, "Tmp: [")?;
        for config in &self.tmp {
            writeln!(f, "\t{}", config)?;
        }
        writeln!(f, "]")?;

        writeln!(f, "Configs: [")?;
        for config in &self.configs {
            writeln!(f, "\t{}", config)?;
        }
        write!(f, "]")
    }
}

impl<I> fmt::Display for Config<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Config::Apply(_, result) => write!(f, "Apply({})", result),
            Config::Construct(symbol, result) => {
                write!(f, "Construct({}, {})", symbol, result)
            }
        }
    }
}

impl<I> TermBuilder<I> {

    pub fn new() -> TermBuilder<I> {
        TermBuilder {
            terms: vec![],
            configs: vec![],
            tmp: vec![],
        }
    }

    pub fn evaluate<F>(&mut self, tp: &mut TermPool, input: I, function: F) -> AnyResult<ATerm>
        where F: Fn(&mut TermPool, &mut TermArgs<I>, I) -> AnyResult<Symbol>  {

        self.configs.push(Config::Apply(input, 0));
        
        loop {
            //println!("{}", self);

            match self.configs.pop() {
                Some(config) => {
                    match config {
                        Config::Apply(input, result) => {
                            // Applies the given function to this input, and obtain a number of symbol and arguments.
                            let top_of_stack = self.terms.len();
                            let mut args = TermArgs {
                                terms: &mut self.terms,
                                tmp: &mut self.tmp,
                                top_of_stack,
                            };

                            let symbol = function(tp, &mut args, input)?;

                            let arity =  symbol.arity();
                            self.configs.push(Config::Construct(symbol, result));
                            self.configs.append(&mut self.tmp);

                            assert_eq!(top_of_stack, self.terms.len() - arity, "Function should have added {arity} arguments");

                        },
                        Config::Construct(symbol, result) => {
                            let arguments = &self.terms[self.terms.len() - symbol.arity()..];                    

                            let t = tp.create(&symbol, arguments);

                            // Remove elements from the stack.
                            self.terms.drain(self.terms.len() - symbol.arity()..);

                            if result == self.terms.len() {
                                // Special case where the result is placed on the first argument.
                                self.terms.push(ATerm::default());
                            } else if result > self.terms.len() {
                                panic!("The result can only replace the first argument.")
                            }

                            self.terms[result] = t;
                        }
                    }
                },
                None => {
                    break;
                }
            }
        }

        assert!(
            self.terms.len() == 1,
            "Expect exactly one term on the result stack"
        );

        return Ok(self
            .terms
            .pop().unwrap())

    }
}

/// Create a random term consisting of the given symbol and constants. Performs
/// iterations number of constructions, and uses chance_duplicates to choose the
/// amount of subterms that are duplicated.
pub fn random_term(
    tp: &mut TermPool,
    symbols: &[(String, usize)],
    constants: &[String],
    iterations: usize,
) -> ATerm {
    use rand::prelude::IteratorRandom;

    assert!(
        !constants.is_empty(),
        "We need constants to be able to create a term"
    );

    let mut subterms = AHashSet::<ATerm>::from_iter(constants.iter().map(|name| {
        let symbol = tp.create_symbol(name, 0);
        tp.create(&symbol, &[])
    }));

    let mut rng = rand::thread_rng();
    let mut result = ATerm::default();
    for _ in 0..iterations {
        let (symbol, arity) = symbols.iter().choose(&mut rng).unwrap();

        let mut arguments = vec![];
        for _ in 0..*arity {
            arguments.push(subterms.iter().choose(&mut rng).unwrap().clone());
        }

        let symbol = tp.create_symbol(symbol, *arity);
        result = tp.create(&symbol, &arguments);

        // Make this term available as another subterm that can be used.
        subterms.insert(result.clone());
    }

    result
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    fn verify_term(term: &ATerm) {
        for subterm in term.iter() {
            assert_eq!(subterm.get_head_symbol().arity(), subterm.arguments().len(), "The arity matches the number of arguments.")
        }        
    }

    #[test]
    fn test_term_iterator() {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(g(a),b)").unwrap();

        let mut result = t.iter();
        assert_eq!(result.next().unwrap(), tp.from_string("f(g(a),b)").unwrap());
        assert_eq!(result.next().unwrap(), tp.from_string("g(a)").unwrap());
        assert_eq!(result.next().unwrap(), tp.from_string("a").unwrap());
        assert_eq!(result.next().unwrap(), tp.from_string("b").unwrap());
    }

    #[test]
    fn test_thread_aterm_pool() {
        let mut threads = vec![];

        for _ in 0..100 {
            threads.push(thread::spawn(
                || {
                    let mut tp = TermPool::new();

                    let terms : Vec::<ATerm> = 
                        (0..100)
                        .map(|_| {
                            random_term(&mut tp,
                                &[("f".to_string(), 2)],
                                &["a".to_string(), "b".to_string()],
                                10)
                        }).collect();

                    tp.collect();

                    for term in &terms {
                        verify_term(term);
                    }
                },
            ));
        }
    }
}
