use mcrl2_syntax::{DisplayPair, Mcrl2Parser, Rule};
use pest::Parser;

fn main() {
    let input = "proc P(x: Bool) = x -> x -> x -> delta <> delta;";

    match Mcrl2Parser::parse(Rule::MCRL2Spec, input) {
        Err(y) => {
            panic!("{}", y);
        },
        Ok(mut rule) => {
            println!("{}", DisplayPair(rule.next().unwrap()));
        }
    } 
}