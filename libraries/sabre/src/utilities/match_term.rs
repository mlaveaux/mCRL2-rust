use mcrl2::{
    aterm::{ATermRef, Protected, TermPool},
    data::{
        DataApplication, DataExpression, DataExpressionRef, DataFunctionSymbolRef
    },
};

use super::{ExplicitPosition, PositionIndexed};

/// A temporary term that can be used for matching purposes to avoid
/// constructing a potentially useless term.
pub struct MatchTerm<'a> {
    symbol: DataFunctionSymbolRef<'a>,
    arguments: &'a Protected<Vec<DataExpressionRef<'static>>>,
}

#[derive(Debug)]
pub enum MatchTermInner<'a> {
    Match(
        DataFunctionSymbolRef<'a>,
        &'a Protected<Vec<DataExpressionRef<'static>>>,
    ),
    Term(DataExpressionRef<'a>),
}

impl<'a> MatchTerm<'a> {
    /// Create a new MatchTerm from its components
    pub fn new(symbol: DataFunctionSymbolRef<'a>,
        input_arguments: &[DataExpressionRef<'static>],
        arguments: &'a mut Protected<Vec<DataExpressionRef<'static>>>,
    ) -> MatchTerm<'a> {
        {
            let mut write = arguments.write();
            write.clear();

            for arg in input_arguments {
                let arg = write.protect(arg);
                write.push(arg.into());
            }
        }

        MatchTerm { symbol, arguments }
    }

    /// Converts the match term to an actual term.
    pub fn to_term(&self, tp: &mut TermPool) -> DataExpression {
        let read = self.arguments.read();
        if read.is_empty() {
            self.symbol.protect().into()
        } else {
            DataApplication::new(tp, &self.symbol, &read).into()
        }
    }

    /// Returns the function symbol of the term without constructing it.
    pub fn data_function_symbol(&self, position: &ExplicitPosition) -> DataFunctionSymbolRef<'_> {
        match self.get_position(position) {
            MatchTermInner::Match(symbol, _) => {
                unsafe {
                    std::mem::transmute(symbol.copy())
                }
            },
            MatchTermInner::Term(t) => {
                unsafe {
                    std::mem::transmute(DataExpressionRef::from(t.copy()).data_function_symbol())
                }
            }
        }
    }
}

impl PositionIndexed for MatchTerm<'_> {
    type Target<'a> = MatchTermInner<'a> where Self: 'a;

    fn get_position<'a>(&'a self, position: &ExplicitPosition) -> Self::Target<'a> {

        // Loop through the position
        let mut it = position.indices.iter();
        match it.next() {
            None => {
                MatchTermInner::Match(self.symbol.copy(), &self.arguments)
            }
            Some(index) => {
                let read = self.arguments.read();

                // Take into account that [symbol, t1, ..., tn]
                let root = if *index == 1 {
                    let t: ATermRef = self.symbol.copy().into();
                    let t: DataExpressionRef = t.into();
                    t
                } else {
                    read[*index - 2].copy()
                };

                let mut result = root.copy();

                for index in it {
                    result = result.arg(index - 1).upgrade(&root).into(); // Note that positions are 1 indexed.
                }
        
                unsafe { MatchTermInner::Term(std::mem::transmute(result)) }
            }
        }
    }
}

impl<'a> PartialEq for MatchTermInner<'a> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            MatchTermInner::Match(symbol, arguments) => match other {
                MatchTermInner::Match(other_symbol, other_arguments) => {
                    symbol == other_symbol && arguments == other_arguments
                }
                MatchTermInner::Term(other_t) => {

                    *symbol == other_t.data_function_symbol()
                        && arguments
                            .read()
                            .iter()
                            .map(|t| t.copy())
                            .eq(other_t.data_arguments().map(|t| DataExpressionRef::from(t)))
                }
            },
            MatchTermInner::Term(t) => match other {
                MatchTermInner::Match(other_symbol, other_arguments) => {
                    
                    t.data_function_symbol() == *other_symbol
                        && t.data_arguments()
                            .map(|t| DataExpressionRef::from(t))
                            .eq(other_arguments.read().iter().map(|t| t.copy()))
                }

                MatchTermInner::Term(other_t) => t == other_t,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use ahash::AHashSet;
    use mcrl2::{aterm::{random_term, ATermRef}, data::is_data_expression};

    use rand::{rngs::StdRng, Rng, SeedableRng};
    use test_log::test;

    use crate::utilities::{to_untyped_data_expression, PositionIterator};

    use super::*;

    #[test]
    fn test_match_term() {
        let mut tp = TermPool::new();
        
        let seed: u64 =  rand::thread_rng().gen();
        println!("seed: {}", seed);
        let mut rng = StdRng::seed_from_u64(seed);

        for _ in 0..100 {
            let t = random_term(&mut tp, 
                &mut rng, 
                &[("f".to_string(), 2)],
                &["a".to_string(), "b".to_string()],
                10);
            let t = to_untyped_data_expression(&mut tp, &t, &AHashSet::new());

            let mut args = Protected::new(Vec::<DataExpressionRef<'static>>::new());
            let mut write = args.write();
            for arg in t.data_arguments() {
                let arg = write.protect(&arg);
                write.push(arg.into());
            }

            let mut args2 = Protected::new(Vec::<DataExpressionRef<'static>>::new());
            let match_term = MatchTerm::new(t.data_function_symbol(), &write, &mut args2);

            // Check that the match term is equal to the term from which it was constructed
            println!("Testing {:?}", t);
            assert_eq!(match_term.get_position(&ExplicitPosition::default()), MatchTermInner::Term(t.copy()));
            assert_eq!(MatchTermInner::Term(t.copy()), match_term.get_position(&ExplicitPosition::default()));

            let t: &ATermRef = &t;
            for (subterm, position) in PositionIterator::new(t.copy()) {
                if is_data_expression(&subterm) {
                    println!("Position {}", position);
                    assert_eq!(match_term.get_position(&position), MatchTermInner::Term(subterm.copy().into()));
                    assert_eq!(MatchTermInner::Term(subterm.copy().into()), match_term.get_position(&position));
                }
            }

            //assert_eq!(&match_term.to_term(&mut tp), t, "The to_term function should return the original");
        }

    }
}