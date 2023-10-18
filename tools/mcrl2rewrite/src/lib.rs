use std::cell::RefCell;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::rc::Rc;
use std::time::Instant;

use ahash::AHashSet;
use anyhow::Result as AnyResult;
use mcrl2::atermpp::TermPool;
use mcrl2::data::{DataSpecification, JittyRewriter};
use rec_tests::load_REC_from_file;
use sabre::utilities::to_data_expression;
use sabre::{InnermostRewriter, RewriteEngine, SabreRewriter, RewriteSpecification};

/// Performs state space exploration of the given model and returns the number of states.
pub fn rewrite_data_spec(tp: Rc<RefCell<TermPool>>, filename_dataspec: &str, filename_expressions: &str) -> AnyResult<()> {
    // Read the data specification
    let data_spec_text = fs::read_to_string(filename_dataspec)?;
    let data_spec = DataSpecification::new(&data_spec_text);

    // Create a jitty rewriter;
    let mut jitty_rewriter = JittyRewriter::new(&data_spec);

    // Convert to the rewrite rules that sabre expects.
    let rewrite_spec = RewriteSpecification::from(data_spec.clone());
    let mut inner_rewriter = InnermostRewriter::new(tp, &rewrite_spec);
    let mut sabre_rewriter = SabreRewriter::new(tp, &rewrite_spec);

    // Open the file in read-only mode.
    let file = File::open(filename_expressions).unwrap();

    // Read the file line by line, and return an iterator of the lines of the file.
    for line in BufReader::new(file).lines().map(|x| x.unwrap()) {
        println!("{}", &line);

        let term = data_spec.parse(&line);

        let now = Instant::now();
        jitty_rewriter.rewrite(&term);
        println!("jitty rewrite took {} ms", now.elapsed().as_millis());

        let now = Instant::now();
        inner_rewriter.rewrite(term.clone());
        println!("innermost rewrite took {} ms", now.elapsed().as_millis());
    
        let now = Instant::now();
        let result = sabre_rewriter.rewrite(term.clone());
        println!("sabre rewrite took {} ms", now.elapsed().as_millis());
    }

    Ok(())
}

pub fn rewrite_rec(specification: &str, text: &str) -> AnyResult<()> {
    let tp = Rc::new(RefCell::new(TermPool::new()));

    let (syntax_spec, _) =
        load_REC_from_file(&mut tp.borrow_mut(), specification.into());
    let spec = syntax_spec.to_rewrite_spec(&mut tp.borrow_mut());
    let term_str = tp.borrow_mut().from_string(text)?;
    let term = to_data_expression(&mut tp.borrow_mut(), &term_str, &AHashSet::new());

    let mut sa = SabreRewriter::new(tp.clone(), &spec);
    let mut inner = InnermostRewriter::new(tp.clone(), &spec);

    let now = Instant::now();
    inner.rewrite(term.clone());
    println!("innermost rewrite took {} ms", now.elapsed().as_millis());

    let now = Instant::now();
    let result = sa.rewrite(term.clone());
    println!("sabre rewrite took {} ms", now.elapsed().as_millis());

    println!("{}", result);
    Ok(())
}

