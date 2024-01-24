use std::{cell::RefCell, rc::Rc};

use mcrl2::{data::DataSpecification, aterm::TermPool};
use sabre::InnermostRewriter;

use enumerate::Enumerator;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_target(false)
        .format_timestamp(None)
        .init();

    let text = "
        sort Peano = struct s(x : Peano) | zero;          
    ";

    let data_spec = DataSpecification::new(text).unwrap();
    let tp = Rc::new(RefCell::new(TermPool::new()));

    let rewriter = Rc::new(InnermostRewriter::new(tp.clone(), &data_spec.clone().into()));
    let enumerator = Enumerator::new(tp.clone(), data_spec.clone(), rewriter);

    let expression = data_spec.parse_variable("v: Peano");

    enumerator.enumerate(expression.unwrap().into());
}
    