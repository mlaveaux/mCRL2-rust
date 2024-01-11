use std::rc::Rc;

use mcrl2::{aterm::ATerm, data::{DataSpecification, DataExpression}};
use sabre::{RewriteEngine, utilities::create_var_map};


/// This object implements the enumeration of open terms. This is also called
/// "narrowing" in other contexts. 
/// 
/// # Narrowing
/// 
/// Given an open term, i.e. one that contains free variables, such as "x < 5".
/// Pick a variable, in this case "x", and instantiate it with a constructor of
/// the data specification. This depends on the type of the variable, for
/// example natural numbers in Peano notation have "0" and "s(y)" as
/// constructors. In that case we can instantiate it as "0 < 5" and "s(x') < 5",
/// where "x'" is a new natural number variable.
/// 
/// 

struct Enumerator<R : RewriteEngine> {

    rewriter: Rc<R>,

    specification: DataSpecification,

}

impl<R: RewriteEngine> Enumerator<R> {

    /// Creates an enumerator for the given data specification, with a preconstructed rewriter.
    /// Assumes that R is a rewriter for RewriteSpecification::from(specification).
    pub fn new(specification: DataSpecification, rewriter: Rc<R>) -> Enumerator<R> {
        Enumerator {
            specification,
            rewriter
        }
    }

    pub fn enumerate(&self, expression: DataExpression) {

        // Typically we have some static expression c that we want to enumerate and subsitute certain variable for.
        let variables = create_var_map(&expression);

        for (var, position) in &variables {

            for cons in self.specification.constructors(&var.sort()) {
                
            }
        }
    }
}




#[cfg(test)]
mod tests {
    use std::{rc::Rc, cell::RefCell};

    use mcrl2::{data::DataSpecification, aterm::TermPool};
    use sabre::InnermostRewriter;

    use super::Enumerator;


    #[test]
    fn test_enumerator() {

        let text = "
            sort Peano = struct s(x : Peano) | zero;

                    
        ";

        let data_spec = DataSpecification::new(text).unwrap();
        let tp = Rc::new(RefCell::new(TermPool::new()));

        let rewriter = Rc::new(InnermostRewriter::new(tp, &data_spec.clone().into()));
        let enumerator = Enumerator::new(data_spec.clone(), rewriter);

        let expression = data_spec.parse("x");

        enumerator.enumerate(expression);

    }
}