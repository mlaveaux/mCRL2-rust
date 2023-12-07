#[cfg(test)]
mod tests
{
    use std::time::Instant;
    use std::{cell::RefCell, rc::Rc};
    use test_case::test_case;

    use ahash::AHashSet;
    use mcrl2::atermpp::{ATerm, TermPool};
    use sabre::{SabreRewriter, RewriteEngine, RewriteSpecification, InnermostRewriter};
    use sabre::utilities::to_data_expression;
    use rec_tests::load_REC_from_strings;

    #[test_case(vec![include_str!("../REC_files/benchexpr10.rec"), include_str!("../REC_files/asfsdfbenchmark.rec")], include_str!("../validated_results/result_benchexpr10.txt") ; "benchexpr10")]
    #[test_case(vec![include_str!("../REC_files/benchsym10.rec"), include_str!("../REC_files/asfsdfbenchmark.rec")], include_str!("../validated_results/result_benchsym10.txt") ; "benchsym10")]
    #[test_case(vec![include_str!("../REC_files/bubblesort10.rec"), include_str!("../REC_files/bubblesort.rec")], include_str!("../validated_results/result_bubblesort10.txt") ; "bubblesort10")]
    #[test_case(vec![include_str!("../REC_files/bubblesort20.rec"), include_str!("../REC_files/bubblesort.rec")], include_str!("../validated_results/result_bubblesort20.txt") ; "bubblesort20")]
    #[test_case(vec![include_str!("../REC_files/bubblesort100.rec"), include_str!("../REC_files/bubblesort.rec")], include_str!("../validated_results/result_bubblesort100.txt") ; "bubblesort100")]
    #[test_case(vec![include_str!("../REC_files/calls.rec")], include_str!("../validated_results/result_calls.txt") ; "calls")]
    #[test_case(vec![include_str!("../REC_files/check1.rec")], include_str!("../validated_results/result_check1.txt") ; "check1")]
    #[test_case(vec![include_str!("../REC_files/check2.rec")], include_str!("../validated_results/result_check2.txt") ; "check2")]
    #[test_case(vec![include_str!("../REC_files/confluence.rec")], include_str!("../validated_results/result_confluence.txt") ; "confluence")]
    #[test_case(vec![include_str!("../REC_files/factorial5.rec"), include_str!("../REC_files/factorial.rec")], include_str!("../validated_results/result_factorial5.txt") ; "factorial5")]
    #[test_case(vec![include_str!("../REC_files/factorial6.rec"), include_str!("../REC_files/factorial.rec")], include_str!("../validated_results/result_factorial6.txt") ; "factorial6")]
    #[test_case(vec![include_str!("../REC_files/fibonacci05.rec"), include_str!("../REC_files/fibonacci.rec")], include_str!("../validated_results/result_fibonacci05.txt") ; "fibonacci05")]
    #[test_case(vec![include_str!("../REC_files/fibonacci18.rec"), include_str!("../REC_files/fibonacci.rec")], include_str!("../validated_results/result_fibonacci18.txt") ; "fibonacci18")]
    #[test_case(vec![include_str!("../REC_files/garbagecollection.rec")], include_str!("../validated_results/result_garbagecollection.txt") ; "garbagecollection")]
    #[test_case(vec![include_str!("../REC_files/hanoi4.rec"), include_str!("../REC_files/hanoi.rec")], include_str!("../validated_results/result_hanoi4.txt") ; "hanoi4")]
    #[test_case(vec![include_str!("../REC_files/hanoi8.rec"), include_str!("../REC_files/hanoi.rec")], include_str!("../validated_results/result_hanoi8.txt") ; "hanoi8")]
    #[test_case(vec![include_str!("../REC_files/logic3.rec")], include_str!("../validated_results/result_logic3.txt") ; "logic3")]
    #[test_case(vec![include_str!("../REC_files/merge.rec")], include_str!("../validated_results/result_merge.txt") ; "merge")]
    #[test_case(vec![include_str!("../REC_files/mergesort10.rec"), include_str!("../REC_files/mergesort.rec")], include_str!("../validated_results/result_mergesort10.txt") ; "mergesort10")]
    #[test_case(vec![include_str!("../REC_files/missionaries2.rec"), include_str!("../REC_files/missionaries.rec")], include_str!("../validated_results/result_missionaries2.txt") ; "missionaries2")]
    #[test_case(vec![include_str!("../REC_files/missionaries3.rec"), include_str!("../REC_files/missionaries.rec")], include_str!("../validated_results/result_missionaries3.txt") ; "missionaries3")]
    #[test_case(vec![include_str!("../REC_files/quicksort10.rec"), include_str!("../REC_files/quicksort.rec")], include_str!("../validated_results/result_quicksort10.txt") ; "quicksort10")]
    #[test_case(vec![include_str!("../REC_files/revelt.rec")], include_str!("../validated_results/result_revelt.txt") ; "revelt")]
    #[test_case(vec![include_str!("../REC_files/revnat100.rec"), include_str!("../REC_files/revnat.rec")], include_str!("../validated_results/result_revnat100.txt") ; "revnat100")]
    #[test_case(vec![include_str!("../REC_files/searchinconditions.rec")], include_str!("../validated_results/result_searchinconditions.txt") ; "searchinconditions")]
    #[test_case(vec![include_str!("../REC_files/sieve20.rec"), include_str!("../REC_files/sieve.rec")], include_str!("../validated_results/result_sieve20.txt") ; "sieve20")]
    #[test_case(vec![include_str!("../REC_files/sieve100.rec"), include_str!("../REC_files/sieve.rec")], include_str!("../validated_results/result_sieve100.txt") ; "sieve100")]
    #[test_case(vec![include_str!("../REC_files/soundnessofparallelengines.rec")], include_str!("../validated_results/result_soundnessofparallelengines.txt") ; "soundnessofparallelengines")]
    #[test_case(vec![include_str!("../REC_files/tak18.rec"), include_str!("../REC_files/tak.rec")], include_str!("../validated_results/result_tak18.txt") ; "tak18")]
    #[test_case(vec![include_str!("../REC_files/tautologyhard.rec")], include_str!("../validated_results/result_tautologyhard.txt") ; "tautologyhard")]
    #[test_case(vec![include_str!("../REC_files/tricky.rec")], include_str!("../validated_results/result_tricky.txt") ; "tricky")]
    fn rec_test(rec_files: Vec<&str>, expected_result: &str) 
    {
        let tp = Rc::new(RefCell::new(TermPool::new()));
        let (spec, terms): (RewriteSpecification, Vec<ATerm>) = { 
            let (syntax_spec, syntax_terms) = load_REC_from_strings(&mut tp.borrow_mut(), &rec_files);
            let result = syntax_spec.to_rewrite_spec(&mut tp.borrow_mut());
            (result, syntax_terms.iter().map(|t| { 
                to_data_expression(&mut tp.borrow_mut(), t, &AHashSet::new())
            }).collect())
        };

        // Test Sabre rewriter
        let mut sa = SabreRewriter::new(tp.clone(), &spec);
        let mut inner = InnermostRewriter::new(tp.clone(), &spec);
        
        let mut expected = expected_result.split('\n');

        for term in &terms {
            let expected_term = tp.borrow_mut().from_string(expected.next().unwrap()).unwrap();
            let expected_result = to_data_expression(&mut tp.borrow_mut(), &expected_term, &AHashSet::new());

            let now = Instant::now();

            let result = inner.rewrite(term.clone());
            assert_eq!(result, expected_result, "The inner rewrite result doesn't match the expected result");

            println!("innermost rewrite took {} ms", now.elapsed().as_millis());
            let now = Instant::now();

            let result = sa.rewrite(term.clone());
            assert_eq!(result, expected_result, "The sabre rewrite result doesn't match the expected result");
            
            println!("sabre rewrite took {} ms", now.elapsed().as_millis());
        }
    }

    #[cfg(not(debug_assertions))]
    #[test_case(vec![include_str!("../REC_files/benchexpr20.rec"), include_str!("../REC_files/asfsdfbenchmark.rec")], include_str!("../validated_results/result_benchexpr20.txt") ; "benchexpr20")]
    #[test_case(vec![include_str!("../REC_files/benchsym20.rec"), include_str!("../REC_files/asfsdfbenchmark.rec")], include_str!("../validated_results/result_benchsym20.txt") ; "benchsym20")]
    #[test_case(vec![include_str!("../REC_files/closure.rec")], include_str!("../validated_results/result_closure.txt") ; "closure")]
    // #[test_case(vec![include_str!("../REC_files/dart.rec")], include_str!("../validated_results/result_dart.txt") ; "dart")]
    #[test_case(vec![include_str!("../REC_files/empty.rec")], include_str!("../validated_results/result_empty.txt") ; "empty")]
    #[test_case(vec![include_str!("../REC_files/evalexpr.rec")], include_str!("../validated_results/result_evalexpr.txt") ; "evalexpr")]
    #[test_case(vec![include_str!("../REC_files/evaltree.rec")], include_str!("../validated_results/result_evaltree.txt") ; "evaltree")]
    //#[test_case(vec![include_str!("../REC_files/factorial7.rec"), include_str!("../REC_files/factorial.rec")], include_str!("../validated_results/result_factorial7.txt") ; "factorial7")]
    //#[test_case(vec![include_str!("../REC_files/factorial8.rec"), include_str!("../REC_files/factorial.rec")], include_str!("../validated_results/result_factorial8.txt") ; "factorial8")]
    //#[test_case(vec![include_str!("../REC_files/factorial9.rec"), include_str!("../REC_files/factorial.rec")], include_str!("../validated_results/result_factorial9.txt") ; "factorial9")]
    //#[test_case(vec![include_str!("../REC_files/fibonacci19.rec"), include_str!("../REC_files/fibonacci.rec")], include_str!("../validated_results/result_fibonacci19.txt") ; "fibonacci19")]
    //#[test_case(vec![include_str!("../REC_files/fibonacci20.rec"), include_str!("../REC_files/fibonacci.rec")], include_str!("../validated_results/result_fibonacci20.txt") ; "fibonacci20")]
    //#[test_case(vec![include_str!("../REC_files/fibonacci21.rec"), include_str!("../REC_files/fibonacci.rec")], include_str!("../validated_results/result_fibonacci21.txt") ; "fibonacci21")]
    #[test_case(vec![include_str!("../REC_files/natlist.rec")], include_str!("../validated_results/result_natlist.txt") ; "natlist")]
    #[test_case(vec![include_str!("../REC_files/oddeven.rec")], include_str!("../validated_results/result_oddeven.txt") ; "oddeven")]
    #[test_case(vec![include_str!("../REC_files/order.rec")], include_str!("../validated_results/result_order.txt") ; "order")]
    //#[test_case(vec![include_str!("../REC_files/hanoi12.rec"), include_str!("../REC_files/hanoi.rec")], include_str!("../validated_results/result_hanoi12.txt") ; "hanoi12")]   
    #[test_case(vec![include_str!("../REC_files/permutations6.rec"), include_str!("../REC_files/permutations.rec")], include_str!("../validated_results/result_permutations6.txt") ; "permutations6")]
    //#[test_case(vec![include_str!("../REC_files/permutations7.rec"), include_str!("../REC_files/permutations.rec")], include_str!("../validated_results/result_permutations7.txt") ; "permutations7")]
    #[test_case(vec![include_str!("../REC_files/revnat1000.rec"), include_str!("../REC_files/revnat.rec")], include_str!("../validated_results/result_revnat1000.txt") ; "revnat1000")]
    #[test_case(vec![include_str!("../REC_files/sieve1000.rec"), include_str!("../REC_files/sieve.rec")], include_str!("../validated_results/result_sieve1000.txt") ; "sieve1000")]
    fn rec_test_release(REC_files: Vec<&str>, expected_result: &str) {
        rec_test(REC_files, expected_result);
    }
}