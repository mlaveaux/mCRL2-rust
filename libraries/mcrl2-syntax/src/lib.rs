use pest_derive::Parser;
use pest::pratt_parser::{Assoc::*, Op, PrattParser};

#[derive(Parser)]
#[grammar = "mcrl2_grammar.pest"]
struct Mcrl2Parser;

pub fn mcrl2_pratt_parser() -> PrattParser<Rule> {
    // Precedence is defined lowest to highest
    PrattParser::new()
        // Sort operators
        .op(Op::infix(Rule::SortExprProduct, Right))
        .op(Op::infix(Rule::SortExprFunction, Left))

        // DataExpression operators
        .op(Op::infix(Rule::DataExprAdd, Right))
}


#[cfg(test)]
mod tests {
    use pest::Parser;
    use indoc::indoc;
    use test_case::test_case;

    use crate::{Mcrl2Parser, Rule};

    #[test]
    fn test_parse_term() {
        let term = "f(a, b)";
        
        let result = Mcrl2Parser::parse(Rule::TermSpec, term).unwrap();
        print!("{}", result);
    }

    #[test]
    fn test_parse_ifthen() {
        let expr = "init a -> b -> c <> delta;";
        
        let result = Mcrl2Parser::parse(Rule::MCRL2Spec, expr).unwrap();
        print!("{}", result);
    }

    #[test]
    fn test_parse_keywords() {
        let expr = "map or : Boolean # Boolean -> Boolean ;";
        
        let result = Mcrl2Parser::parse(Rule::MCRL2Spec, expr).unwrap();
        print!("{}", result);
    }
    
    #[test]
    fn test_parse_sort_spec() {
        let sort_spec = indoc!{"
            sort D = Bool -> Int -> Bool;
            

            % Test
            F     = struct d1 | d2;
            Error = struct e;
        "};

        let result = Mcrl2Parser::parse(Rule::MCRL2Spec, sort_spec).unwrap();
        print!("{}", result);
    }

    #[test]
    fn test_parse_abp() {
        let abp_spec = indoc!{"
            % This file contains the alternating bit protocol, as described 
            % J.F. Groote and M.R. Mousavi. Modeling and analysis of communicating
            % systems. The MIT Press, 2014.
            %
            % The only exception is that the domain D consists of two data elements to
            % facilitate simulation.

            sort
            D     = struct d1 | d2;
            Error = struct e;

            act
            r1,s4: D;
            s2,r2,c2: D # Bool;
            s3,r3,c3: D # Bool;
            s3,r3,c3: Error;
            s5,r5,c5: Bool;
            s6,r6,c6: Bool;
            s6,r6,c6: Error;
            i;

            proc
            S(b:Bool)     = sum d:D. r1(d).T(d,b);
            T(d:D,b:Bool) = s2(d,b).(r6(b).S(!b)+(r6(!b)+r6(e)).T(d,b));

            R(b:Bool)     = sum d:D. r3(d,b).s4(d).s5(b).R(!b)+
                            (sum d:D.r3(d,!b)+r3(e)).s5(!b).R(b);

            K             = sum d:D,b:Bool. r2(d,b).(i.s3(d,b)+i.s3(e)).K;

            L             = sum b:Bool. r5(b).(i.s6(b)+i.s6(e)).L;

            init
            allow({r1,s4,c2,c3,c5,c6,i},
                comm({r2|s2->c2, r3|s3->c3, r5|s5->c5, r6|s6->c6},
                    S(true) || K || L || R(true)
                )
            );
        "};

       match Mcrl2Parser::parse(Rule::MCRL2Spec, abp_spec) {
            Ok(x) => {
                print!("{}", x);
            }, Err(y) => {
                panic!("{}", y);
            }
        }
    }


    #[test_case(include_str!("../../../3rd-party/mCRL2/examples/academic/abp/abp.mcrl2"); "abp")]
    // TODO: Fix issues with the ambiguities in this case.
    // #[test_case(include_str!("../../../3rd-party/mCRL2/examples/industrial/1394/1394-fin.mcrl2"); "1394-fin")]
    fn test_parse_mcrl2_spec(input: &str)
    {       
        if let Err(y) = Mcrl2Parser::parse(Rule::MCRL2Spec, input) {
            panic!("{}", y);
        }
    }


    #[test_case(include_str!("../../../examples/REC/mcrl2/add8.dataspec") ; "add8")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/benchexpr10.dataspec") ; "benchexpr10")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/benchsym10.dataspec") ; "benchsym10")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/benchtree10.dataspec") ; "benchtree10")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/binarysearch.dataspec") ; "binarysearch")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/bubblesort10.dataspec") ; "bubblesort10")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/calls.dataspec") ; "calls")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/check1.dataspec") ; "check1")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/check2.dataspec") ; "check2")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/closure.dataspec") ; "closure")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/confluence.dataspec") ; "confluence")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/dart.dataspec") ; "dart")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/empty.dataspec") ; "empty")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/evalexpr.dataspec") ; "evalexpr")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/evalsym.dataspec") ; "evalsym")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/evaltree.dataspec") ; "evaltree")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/factorial5.dataspec") ; "factorial5")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/fib32.dataspec") ; "fib32")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/fibfree.dataspec") ; "fibfree")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/fibonacci05.dataspec") ; "fibonacci05")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/garbagecollection.dataspec") ; "garbagecollection")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/hanoi4.dataspec") ; "hanoi4")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/intnat.dataspec") ; "intnat")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/langton6.dataspec") ; "langton6")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/logic3.dataspec") ; "logic3")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/maa.dataspec") ; "maa")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/merge.dataspec") ; "merge")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/mergesort10.dataspec") ; "mergesort10")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/missionaries2.dataspec") ; "missionaries2")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/mul8.dataspec") ; "mul8")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/natlist.dataspec") ; "natlist")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/oddeven.dataspec") ; "oddeven")]
    #[test_case(include_str!("../../../examples/REC/mcrl2/omul8.dataspec") ; "omul8")]
    fn test_parse_mcrl2_dataspec(input: &str)
    {        
        if let Err(y) = Mcrl2Parser::parse(Rule::MCRL2Spec, input) {
            panic!("{}", y);
        }
    }
}