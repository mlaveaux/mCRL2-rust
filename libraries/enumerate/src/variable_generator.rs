use std::{cell::RefCell, rc::Rc};

use mcrl2::{data::DataVariable, aterm::TermPool};



/// This can be used to generate new fresh variables with a given name as prefix.
pub struct VariableGenerator {
    prefix: String,
    unique_number: usize,
    tp: Rc<RefCell<TermPool>>,
}

impl VariableGenerator {
    pub fn new(tp: Rc<RefCell<TermPool>>, prefix: impl Into<String>) -> VariableGenerator {
        VariableGenerator {
            prefix: prefix.into(),
            unique_number: 0,
            tp
        }
    }


    pub fn next(&mut self) -> DataVariable {
        self.unique_number += 1;
        DataVariable::new(&mut self.tp.borrow_mut(), &format!("{}_{}", self.prefix, self.unique_number))
    }

}