use mcrl2::data::{DataApplication, DataSpecification, self};
use std::{cell::RefCell, rc::Rc};
use test_case::test_case;

use mcrl2::aterm::{ATerm, TermPool};
use sabre::{InnermostRewriter, RewriteEngine, SabreRewriter};

// #[test_case(include_str!("../../../examples/REC/mcrl2/add8.dataspec"), include_str!("../../../examples/REC/mcrl2/add8.expressions"), include_str!("snapshot/result_add8.txt") ; "add8")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/add16.dataspec"), include_str!("../../../examples/REC/mcrl2/add16.expressions"), include_str!("snapshot/result_add16.txt") ; "add16")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/add32.dataspec"), include_str!("../../../examples/REC/mcrl2/add32.expressions"), include_str!("snapshot/result_add32.txt") ; "add32")]
#[test_case(include_str!("../../../examples/REC/mcrl2/benchexpr10.dataspec"), include_str!("../../../examples/REC/mcrl2/benchexpr10.expressions"), include_str!("snapshot/result_benchexpr10.txt") ; "benchexpr10")]
#[test_case(include_str!("../../../examples/REC/mcrl2/benchsym10.dataspec"), include_str!("../../../examples/REC/mcrl2/benchsym10.expressions"), include_str!("snapshot/result_benchsym10.txt") ; "benchsym10")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/bubblesort10.dataspec"), include_str!("../../../examples/REC/mcrl2/bubblesort10.expressions"), include_str!("snapshot/result_bubblesort10.txt") ; "bubblesort10")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/bubblesort20.dataspec"), include_str!("../../../examples/REC/mcrl2/bubblesort20.expressions"), include_str!("snapshot/result_bubblesort20.txt") ; "bubblesort20")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/bubblesort100.dataspec"), include_str!("../../../examples/REC/mcrl2/bubblesort100.expressions"), include_str!("snapshot/result_bubblesort100.txt") ; "bubblesort100")]
#[test_case(include_str!("../../../examples/REC/mcrl2/calls.dataspec"), include_str!("../../../examples/REC/mcrl2/calls.expressions"), include_str!("snapshot/result_calls.txt") ; "calls")]
#[test_case(include_str!("../../../examples/REC/mcrl2/check1.dataspec"), include_str!("../../../examples/REC/mcrl2/check1.expressions"), include_str!("snapshot/result_check1.txt") ; "check1")]
#[test_case(include_str!("../../../examples/REC/mcrl2/check2.dataspec"), include_str!("../../../examples/REC/mcrl2/check2.expressions"), include_str!("snapshot/result_check2.txt") ; "check2")]
#[test_case(include_str!("../../../examples/REC/mcrl2/confluence.dataspec"), include_str!("../../../examples/REC/mcrl2/confluence.expressions"), include_str!("snapshot/result_confluence.txt") ; "confluence")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/factorial5.dataspec"), include_str!("../../../examples/REC/mcrl2/factorial5.expressions"), include_str!("snapshot/result_factorial5.txt") ; "factorial5")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/factorial6.dataspec"), include_str!("../../../examples/REC/mcrl2/factorial6.expressions"), include_str!("snapshot/result_factorial6.txt") ; "factorial6")]
#[test_case(include_str!("../../../examples/REC/mcrl2/fibonacci05.dataspec"), include_str!("../../../examples/REC/mcrl2/fibonacci05.expressions"), include_str!("snapshot/result_fibonacci05.txt") ; "fibonacci05")]
#[test_case(include_str!("../../../examples/REC/mcrl2/garbagecollection.dataspec"), include_str!("../../../examples/REC/mcrl2/garbagecollection.expressions"), include_str!("snapshot/result_garbagecollection.txt") ; "garbagecollection")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/hanoi4.dataspec"), include_str!("../../../examples/REC/mcrl2/hanoi4.expressions"), include_str!("snapshot/result_hanoi4.txt") ; "hanoi4")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/hanoi8.dataspec"), include_str!("../../../examples/REC/mcrl2/hanoi8.expressions"), include_str!("snapshot/result_hanoi8.txt") ; "hanoi8")]
#[test_case(include_str!("../../../examples/REC/mcrl2/logic3.dataspec"), include_str!("../../../examples/REC/mcrl2/logic3.expressions"), include_str!("snapshot/result_logic3.txt") ; "logic3")]
#[test_case(include_str!("../../../examples/REC/mcrl2/merge.dataspec"), include_str!("../../../examples/REC/mcrl2/merge.expressions"), include_str!("snapshot/result_merge.txt") ; "merge")]
#[test_case(include_str!("../../../examples/REC/mcrl2/mergesort10.dataspec"), include_str!("../../../examples/REC/mcrl2/mergesort10.expressions"), include_str!("snapshot/result_mergesort10.txt") ; "mergesort10")]
#[test_case(include_str!("../../../examples/REC/mcrl2/missionaries2.dataspec"), include_str!("../../../examples/REC/mcrl2/missionaries2.expressions"), include_str!("snapshot/result_missionaries2.txt") ; "missionaries2")]
#[test_case(include_str!("../../../examples/REC/mcrl2/missionaries3.dataspec"), include_str!("../../../examples/REC/mcrl2/missionaries3.expressions"), include_str!("snapshot/result_missionaries3.txt") ; "missionaries3")]
#[test_case(include_str!("../../../examples/REC/mcrl2/quicksort10.dataspec"), include_str!("../../../examples/REC/mcrl2/quicksort10.expressions"), include_str!("snapshot/result_quicksort10.txt") ; "quicksort10")]
#[test_case(include_str!("../../../examples/REC/mcrl2/revelt.dataspec"), include_str!("../../../examples/REC/mcrl2/revelt.expressions"), include_str!("snapshot/result_revelt.txt") ; "revelt")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/revnat100.dataspec"), include_str!("../../../examples/REC/mcrl2/revnat100.expressions"), include_str!("snapshot/result_revnat100.txt") ; "revnat100")]
#[test_case(include_str!("../../../examples/REC/mcrl2/searchinconditions.dataspec"), include_str!("../../../examples/REC/mcrl2/searchinconditions.expressions"), include_str!("snapshot/result_searchinconditions.txt") ; "searchinconditions")]
// #[test_case(include_str!("../../../examples/REC/mcrl2/sieve20.dataspec"), include_str!("../../../examples/REC/mcrl2/sieve20.expressions"), include_str!("snapshot/result_sieve20.txt") ; "sieve20")]
#[test_case(include_str!("../../../examples/REC/mcrl2/sieve100.dataspec"), include_str!("../../../examples/REC/mcrl2/sieve100.expressions"), include_str!("snapshot/result_sieve100.txt") ; "sieve100")]
#[test_case(include_str!("../../../examples/REC/mcrl2/soundnessofparallelengines.dataspec"), include_str!("../../../examples/REC/mcrl2/soundnessofparallelengines.expressions"), include_str!("snapshot/result_soundnessofparallelengines.txt") ; "soundnessofparallelengines")]
#[test_case(include_str!("../../../examples/REC/mcrl2/tak18.dataspec"), include_str!("../../../examples/REC/mcrl2/tak18.expressions"), include_str!("snapshot/result_tak18.txt") ; "tak18")]
#[test_case(include_str!("../../../examples/REC/mcrl2/tautologyhard.dataspec"), include_str!("../../../examples/REC/mcrl2/tautologyhard.expressions"), include_str!("snapshot/result_tautologyhard.txt") ; "tautologyhard")]
#[test_case(include_str!("../../../examples/REC/mcrl2/tricky.dataspec"), include_str!("../../../examples/REC/mcrl2/tricky.expressions"), include_str!("snapshot/result_tricky.txt") ; "tricky")]

fn rewriter_test(data_spec: &str, expressions: &str, expected_result: &str) {
    let tp = Rc::new(RefCell::new(TermPool::new()));
    let spec = DataSpecification::new(data_spec).unwrap();
    let terms: Vec<ATerm> = expressions.lines().map(|text| spec.parse(text)).collect();

    // Test Sabre rewriter
    // let mut sa = SabreRewriter::new(tp.clone(), &spec.into());
    let mut inner = InnermostRewriter::new(tp.clone(), &spec.clone().into());

    let mut expected = expected_result.split('\n');

    for term in &terms {
        let expected_result = spec.parse(expected.next().unwrap());

        let result = inner.rewrite(term.clone());
        assert_eq!(
            result.clone(),
            expected_result.clone(),
            "The inner rewrite result doesn't match the expected result"
        );

        // let result = sa.rewrite(term.clone());
        // assert_eq!(DataApplication::from(result), DataApplication::from(expected_result), "The sabre rewrite result doesn't match the expected result");
    }
}
