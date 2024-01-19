use std::cell::RefCell;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::rc::Rc;
use std::time::Instant;
use std::fmt::Debug;

use ahash::AHashSet;
use anyhow::{Result as AnyResult, bail};
use clap::ValueEnum;
use mcrl2::aterm::TermPool;
use mcrl2::data::{DataSpecification, JittyRewriter, DataExpression};
use rec_tests::load_REC_from_file;
use sabre::utilities::to_untyped_data_expression;
use sabre::{InnermostRewriter, RewriteEngine, SabreRewriter, RewriteSpecification};

#[derive(ValueEnum, Debug, Clone)]
pub enum Rewriter {
    Jitty,
    Innermost,
    Sabre
}

/// Performs state space exploration of the given model and returns the number of states.
pub fn rewrite_data_spec(tp: Rc<RefCell<TermPool>>, rewriter: Rewriter, filename_dataspec: &str, filename_terms: &str, output: bool) -> AnyResult<()> {
    
    // Read the data specification
    let data_spec_text = fs::read_to_string(filename_dataspec)?;
    let data_spec = DataSpecification::new(&data_spec_text)?;

    // Open the file in read-only mode.
    let file = File::open(filename_terms).unwrap();

    // Read and convert the terms
    let terms: Vec::<DataExpression> = BufReader::new(file).lines().map(|x| {
        data_spec.parse(&x.unwrap())
    }).collect();

    match rewriter {
        Rewriter::Jitty => {
            // Create a jitty rewriter;
            let mut jitty_rewriter = JittyRewriter::new(&data_spec);

            // Read the file line by line, and return an iterator of the lines of the file.
            let now = Instant::now();
            for term in &terms {
                let result = jitty_rewriter.rewrite(term);
                if output {
                    println!("{}", result)
                }
            }
            println!("Jitty rewrite took {} ms", now.elapsed().as_millis());
        },
        Rewriter::Innermost => {    
            let rewrite_spec = RewriteSpecification::from(data_spec.clone());
            let mut inner_rewriter = InnermostRewriter::new(tp.clone(), &rewrite_spec);  
        
            // Read the file line by line, and return an iterator of the lines of the file.
            let now = Instant::now();
            for term in &terms {
                let result = inner_rewriter.rewrite(term.clone());
                if output {
                    println!("{}", result)
                }
            }
            println!("Innermost rewrite took {} ms", now.elapsed().as_millis());
        },
        Rewriter::Sabre => {                
            let rewrite_spec = RewriteSpecification::from(data_spec.clone());
            let mut sabre_rewriter = SabreRewriter::new(tp.clone(), &rewrite_spec);  
        
            let now = Instant::now();
            for term in &terms {                
                let result = sabre_rewriter.rewrite(term.clone());
                if output {
                    println!("{}", result)
                }
            }
            println!("Sabre rewrite took {} ms", now.elapsed().as_millis());
        }
    }

    Ok(())
}

pub fn rewrite_rec(rewriter: Rewriter, filename_specification: &str, output: bool) -> AnyResult<()> {
    let tp = Rc::new(RefCell::new(TermPool::new()));

    let (syntax_spec, syntax_terms) =
        load_REC_from_file(&mut tp.borrow_mut(), filename_specification.into()).unwrap();

    let spec = syntax_spec.to_rewrite_spec(&mut tp.borrow_mut());

    match rewriter {
        Rewriter::Innermost => {
            let mut inner = InnermostRewriter::new(tp.clone(), &spec);

            let now = Instant::now();
            for term in &syntax_terms {
                let term = to_untyped_data_expression(&mut tp.borrow_mut(), term, &AHashSet::new());
                let result = inner.rewrite(term);
                if output {
                    println!("{}", result)
                }
            }
            println!("Innermost rewrite took {} ms", now.elapsed().as_millis());
        },
        Rewriter::Sabre => {
            let mut sa = SabreRewriter::new(tp.clone(), &spec);        
        
            let now = Instant::now();
            for term in &syntax_terms {
                let term = to_untyped_data_expression(&mut tp.borrow_mut(), term, &AHashSet::new());
                let result = sa.rewrite(term);
                if output {
                    println!("{}", result)
                }
            }
            println!("Sabre rewrite took {} ms", now.elapsed().as_millis());
        },
        Rewriter::Jitty => {
            bail!("Cannot use REC specifications with mCRL2's jitty rewriter");
        }
    }

    Ok(())
}

