use mcrl2::data::DataVariable;



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

        DataVariable::new(&mut self.tp.borrow_mut(), &format!("{}_{}", prefix, self.unique_number))
    }

}