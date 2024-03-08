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

    use crate::{Mcrl2Parser, Rule};

    #[test]
    fn test_parse_term() {
        let term = "f(a, b)";
        
        let result = Mcrl2Parser::parse(Rule::TermSpec, term).unwrap();
        print!("{}", result);
    }

    #[test]
    fn test_parse_ifthen() {
        let term = "init a -> b -> c <> delta;";
        
        let result = Mcrl2Parser::parse(Rule::MCRL2Spec, term).unwrap();
        print!("{}", result);
    }

    
    #[test]
    fn test_parse_sort_spec() {
        let abp_spec = indoc!{"
            sort D = Bool -> Int -> Bool;
            

            % Test
            F     = struct d1 | d2;
            Error = struct e;
        "};

        let result = Mcrl2Parser::parse(Rule::MCRL2Spec, abp_spec).unwrap();
        print!("{}", result);
    }

    #[test]
    fn test_parse_mcrl2() {
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

        let result = Mcrl2Parser::parse(Rule::MCRL2Spec, abp_spec).unwrap();
        print!("{}", result);
    }



    // #[test_case(include_str("../../../3rd-party/mCRL2/examples"))]
    // fn test_parse_mcrl2_spec(input: str)
    // {        
    //     let result = Mcrl2Parser::parse(Rule::MCRL2Spec, &input).unwrap();
    //     print!("{}", result);
    // }
}