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

    // This command can be used to generate the tests cases, but duplicate names have to be removed
    // find ./examples/ -name *.mcf -exec echo "#[test_case(include_str"'!'"(\"../../{}\") ; \"$(basename {})\")]" \;

    #[test_case(include_str!("../../../3rd-party/mCRL2/examples/academic/abp/abp.mcrl2"); "abp")]
    // TODO: Fix issues with the ambiguities in this case.
    // #[test_case(include_str!("../../../3rd-party/mCRL2/examples/industrial/1394/1394-fin.mcrl2"); "1394-fin")]
    fn test_parse_mcrl2_spec(input: &str)
    {       
        if let Err(y) = Mcrl2Parser::parse(Rule::MCRL2Spec, input) {
            panic!("{}", y);
        }
    }

  
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/abp/infinitely_often_enabled_then_infinitely_often_taken.mcf") ; "infinitely_often_enabled_then_infinitely_often_taken.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/abp/infinitely_often_lost.mcf") ; "infinitely_often_lost.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/abp/infinitely_often_receive_d1.mcf") ; "infinitely_often_receive_d1.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/abp/infinitely_often_receive_for_all_d.mcf") ; "infinitely_often_receive_for_all_d.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/abp/nodeadlock.mcf") ; "nodeadlock.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/abp/no_duplication_of_messages.mcf") ; "no_duplication_of_messages.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/abp/no_generation_of_messages.mcf") ; "no_generation_of_messages.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/abp/read_then_eventually_send.mcf") ; "read_then_eventually_send.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/abp/read_then_eventually_send_if_fair.mcf") ; "read_then_eventually_send_if_fair.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/bakery/always_can_get_number.mcf") ; "always_can_get_number.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/bakery/get_at_least_number_circulating.mcf") ; "get_at_least_number_circulating.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/bakery/request_can_eventually_enter.mcf") ; "request_can_eventually_enter.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/bakery/request_must_eventually_enter.mcf") ; "request_must_eventually_enter.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/bke/secret_not_leaked.mcf") ; "secret_not_leaked.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/bounded_ricart-agrawala/RA_fixed/properties/deadlock freedom.mcf") ; "deadlock freedom.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/bounded_ricart-agrawala/RA_fixed/properties/no deadlock in model.mcf") ; "no deadlock in model.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/bounded_ricart-agrawala/RA_fixed/properties/starvation freedom.mcf") ; "starvation freedom.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/dining/nostarvation.mcf") ; "nostarvation.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/dining/nostuffing.mcf") ; "nostuffing.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/food_distribution/sustained_delivery.mcf") ; "sustained_delivery.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/leader/at_most_one_leader.mcf") ; "at_most_one_leader.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/leader/leader_always_elected.mcf") ; "leader_always_elected.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/minepump_product_line/family_based_experiments/formula1/prop1.mcf") ; "prop1.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/minepump_product_line/family_based_experiments/formula10/prop10.mcf") ; "prop10.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/minepump_product_line/family_based_experiments/formula11/prop11.mcf") ; "prop11.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/minepump_product_line/family_based_experiments/formula12/prop12.mcf") ; "prop12.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/minepump_product_line/family_based_experiments/formula2/prop2.mcf") ; "prop2.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/minepump_product_line/family_based_experiments/formula3/prop3.mcf") ; "prop3.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/minepump_product_line/family_based_experiments/formula4/prop4.mcf") ; "prop4.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/minepump_product_line/family_based_experiments/formula5/prop5.mcf") ; "prop5.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/minepump_product_line/family_based_experiments/formula6/prop6.mcf") ; "prop6.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/minepump_product_line/family_based_experiments/formula7/prop7.mcf") ; "prop7.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/minepump_product_line/family_based_experiments/formula8/prop8.mcf") ; "prop8.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/minepump_product_line/family_based_experiments/formula9/prop9.mcf") ; "prop9.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/mpsu/mpsu1.mcf") ; "mpsu1.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/mpsu/mpsu2.mcf") ; "mpsu2.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/mpsu/mpsu3.mcf") ; "mpsu3.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/mpsu/mpsu4.mcf") ; "mpsu4.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/mpsu/mpsu5.mcf") ; "mpsu5.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/mpsu/mpsu6.mcf") ; "mpsu6.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/mutex_models/Dekker/properties/Always eventually request.mcf") ; "Always eventually request.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/mutex_models/Dekker/properties/Eventual access if fair.mcf") ; "Eventual access if fair.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/mutex_models/Dekker/properties/Eventual access.mcf") ; "Eventual access.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/mutex_models/Dekker/properties/Mutual exclusion.mcf") ; "Mutual exclusion.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/parallel_proc_with_global_var/parallel_counting.mcf") ; "parallel_counting.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/peterson_justness/justlive.mcf") ; "justlive.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/scheduler/infinitely_often_enabled_then_infinitely_often_taken_a.mcf") ; "infinitely_often_enabled_then_infinitely_often_taken_a.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/academic/trains/infinitely_often_enabled_then_infinitely_often_taken_enter.mcf") ; "infinitely_often_enabled_then_infinitely_often_taken_enter.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/domineering/eventually_player1_or_player2_wins.mcf") ; "eventually_player1_or_player2_wins.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/domineering/player1_can_win.mcf") ; "player1_can_win.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/domineering/player2_can_win.mcf") ; "player2_can_win.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/four_in_a_row/red_wins.mcf") ; "red_wins.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/open_field_tic_tac_toe/red_has_a_winning_strategy.mcf") ; "red_has_a_winning_strategy.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/open_field_tic_tac_toe/yellow_has_a_winning_strategy.mcf") ; "yellow_has_a_winning_strategy.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/othello/exists_draw.mcf") ; "exists_draw.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/othello/red_can_win.mcf") ; "red_can_win.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/othello/red_wins_always.mcf") ; "red_wins_always.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/othello/white_can_win.mcf") ; "white_can_win.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/othello/white_wins_always.mcf") ; "white_wins_always.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/rule1.mcf") ; "rule1.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/rule11.mcf") ; "rule11.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/rule12.mcf") ; "rule12.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/rule14.mcf") ; "rule14.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/rule15.mcf") ; "rule15.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/rule3.mcf") ; "rule3.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/rule4.mcf") ; "rule4.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/rule5.mcf") ; "rule5.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/rule6a.mcf") ; "rule6a.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/rule6b.mcf") ; "rule6b.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/rule789.mcf") ; "rule789.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/winning_strategy_player_1.mcf") ; "winning_strategy_player_1.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/quoridor/properties/winning_strategy_player_2.mcf") ; "winning_strategy_player_2.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/snake/black_has_winning_strategy.mcf") ; "black_has_winning_strategy.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/snake/eventually_white_or_black_wins.mcf") ; "eventually_white_or_black_wins.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/snake/white_has_winning_strategy.mcf") ; "white_has_winning_strategy.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/tictactoe/has_player_cross_a_winning_strategy.mcf") ; "has_player_cross_a_winning_strategy.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/games/tictactoe/one_wrong_move.mcf") ; "one_wrong_move.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/DIRAC/properties_SMS/eventuallyDeleted.mcf") ; "eventuallyDeleted.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/DIRAC/properties_SMS/noTransitFromDeleted.mcf") ; "noTransitFromDeleted.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/DIRAC/properties_WMS/jobFailedToDone.mcf") ; "jobFailedToDone.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/DIRAC/properties_WMS/noZombieJobs.mcf") ; "noZombieJobs.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/ERTMS/version1A/section_I/IU/deterministic_stabilisation.mcf") ; "deterministic_stabilisation.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/ERTMS/version1A/section_I/IU/no_collision.mcf") ; "no_collision.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/ERTMS/version1A/section_I/IU/strong_determinacy.mcf") ; "strong_determinacy.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/ERTMS/version1A/section_I/IU/termination.mcf") ; "termination.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/flexray/mucalc/eventually_comm.mcf") ; "eventually_comm.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/flexray/mucalc/eventually_startup.mcf") ; "eventually_startup.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/ieee-11073/data_can_be_communicated.mcf") ; "data_can_be_communicated.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/ieee-11073/infinite_data_communication_is_possible.mcf") ; "infinite_data_communication_is_possible.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/ieee-11073/no_inconsistent_operating_states.mcf") ; "no_inconsistent_operating_states.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/industrial/ieee-11073/no_successful_transmission_in_inconsistent_operating_states.mcf") ; "no_successful_transmission_in_inconsistent_operating_states.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/probabilistic/coin_tossing/formula1.mcf") ; "formula1.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/probabilistic/coin_tossing/formula2.mcf") ; "formula2.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/probabilistic/sultan_of_persia/best_spouse.mcf") ; "best_spouse.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Knuths_dancing_links/Dancing_links/properties/Correctness.mcf") ; "Correctness.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Lamport_queue/properties/Head get set value.mcf") ; "Head get set value.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Lamport_queue/properties/Head not set out of bounds.mcf") ; "Head not set out of bounds.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Lamport_queue/properties/No out of bounds read.mcf") ; "No out of bounds read.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Lamport_queue/properties/No out of bounds write.mcf") ; "No out of bounds write.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Lamport_queue/properties/Pop always terminates.mcf") ; "Pop always terminates.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Lamport_queue/properties/Pop can terminate.mcf") ; "Pop can terminate.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Lamport_queue/properties/Push always terminates.mcf") ; "Push always terminates.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Lamport_queue/properties/Push can terminate.mcf") ; "Push can terminate.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Lamport_queue/properties/Queue behaviour.mcf") ; "Queue behaviour.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Lamport_queue/properties/Tail get set value.mcf") ; "Tail get set value.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Lamport_queue/properties/Tail not read out of bounds.mcf") ; "Tail not read out of bounds.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Lamport_queue/properties/Tail not set out of bounds.mcf") ; "Tail not set out of bounds.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Petersons_mutex/Petersons_F_F/properties/Bounded overtaking.mcf") ; "Bounded overtaking.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Treiber_stack/Treiber_CAS/properties/Correct release implies correct retrieve.mcf") ; "Correct release implies correct retrieve.mcf")]
    #[test_case(include_str!("../../../3rd-party/mCRL2/./examples/software_models/Treiber_stack/Treiber_CAS/properties/Inevitably retrieve when stacksize is 2.mcf") ; "Inevitably retrieve when stacksize is 2.mcf")]
    fn test_parse_mcrl2_modal_formula(input: &str)
    {
        if let Err(y) = Mcrl2Parser::parse(Rule::StateFrmSpec, input) {
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