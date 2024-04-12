use mcrl2::{
    aterm::{ATermRef, ATermTrait, Protected, TermPool},
    data::{
        DataApplication, DataApplicationRef, DataExpression, DataExpressionRef, DataFunctionSymbol, DataFunctionSymbolRef
    },
};

use super::{ExplicitPosition, PositionIndexed};

/// A temporary term that can be used for matching purposes to avoid
/// constructing a potentially useless term.
pub struct MatchTerm<'a> {
    symbol: DataFunctionSymbolRef<'a>,
    arguments: Protected<Vec<DataExpressionRef<'static>>>,
}

pub enum MatchTermInner<'a> {
    Match(
        DataFunctionSymbolRef<'a>,
        &'a Protected<Vec<DataExpressionRef<'static>>>,
    ),
    Term(ATermRef<'a>),
}

impl<'a> MatchTerm<'a> {
    /// Create a new MatchTerm from its components
    pub fn new(
        tp: &mut TermPool,
        symbol: DataFunctionSymbolRef<'a>,
        input_arguments: &[DataExpressionRef<'static>],
    ) -> MatchTerm<'a> {
        let mut arguments = Protected::new(vec![]);

        {
            let mut write = arguments.write();
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
            DataApplication::from_refs(tp, &self.symbol, &read).into()
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

                let root = read[*index - 1].copy();
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
                    let other_t = DataApplicationRef::from(other_t.copy());

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
                    let t = DataApplicationRef::from(t.copy());
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
