use std::{rc::Rc, cell::RefCell};

use mcrl2::{aterm::TermPool, data::{DataSpecification, DataExpression, DataApplication}};
use sabre::{RewriteEngine, utilities::create_var_map};

use crate::variable_generator::VariableGenerator;


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
    tp: Rc<RefCell<TermPool>>,

    rewriter: Rc<R>,

    specification: DataSpecification,

}

impl<R: RewriteEngine> Enumerator<R> {

    /// Creates an enumerator for the given data specification, with a preconstructed rewriter.
    /// Assumes that R is a rewriter for RewriteSpecification::from(specification).
    pub fn new(tp: Rc<RefCell<TermPool>>, specification: DataSpecification, rewriter: Rc<R>) -> Enumerator<R> {
        Enumerator {
            tp,
            specification,
            rewriter
        }
    }

    pub fn enumerate(&self, expression: DataExpression) {

        // Typically we have some static expression c that we want to enumerate and subsitute certain variable for.
        let variables = create_var_map(&expression);

        // A variable generator with fresh names
        let mut variable_generator = VariableGenerator::new(self.tp.clone(), "x");

        let mut arguments = vec![];

       // let mut queue = vec![];

        for (var, position) in &variables {

            for cons in self.specification.constructors(&var.sort()) {

                println!("{}", cons.sort());

                for _ in 0..1 {
                    arguments.push(variable_generator.next());
                }

                //let t = DataApplication::from_refs(&mut self.tp.borrow_mut(), cons, arguments);




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

        let rewriter = Rc::new(InnermostRewriter::new(tp.clone(), &data_spec.clone().into()));
        let enumerator = Enumerator::new(tp.clone(), data_spec.clone(), rewriter);

        let expression = data_spec.parse("x");

        enumerator.enumerate(expression);

    }
}