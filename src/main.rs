use pest::{Parser};
use pest_derive::Parser;


#[derive(Parser)]
#[grammar = "star.pest"]
pub struct StarParser;

fn main() {
    let test_string = "loop_                                  \
                                 _atom_name                         \
                                 _atomic_mass_ratio                 \
                                     1H	    1.007825031898(14)      \
                                     2H	    1.0070508889220(75)     \
                                     3H	    1.005349760440(27)      \
                                     3He	1.005343107322(20)      \
                             stop_                                  ";


    let successful_parse = StarParser::parse(Rule::data_loop, test_string);
    // println!("{:?}", successful_parse);
    println!("{}", successful_parse.unwrap());
    // println!("{:?}", successful_parse);

}

#[macro_use]
extern crate pest;


#[cfg(test)]
mod tests {
    use pest::parses_to;
    use super::*;

    // data_name
    #[test]
    fn data_name() {
        parses_to! {
            parser: StarParser,
            input:  "_ABC",
            rule:   Rule::data_name,
            tokens: [
                data_name(0, 4)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "__ABC",
            rule:   Rule::data_name,
            tokens: [
                data_name(0, 5)
            ]
        }


        parses_to! {
            parser: StarParser,
            input:  "_'ABC",
            rule:   Rule::data_name,
            tokens: [
                data_name(0, 5)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "_\"ABC",
            rule:   Rule::data_name,
            tokens: [
                data_name(0, 5)
            ]
        }

        fails_with! {
            parser: StarParser,
            input: "ABC",
            rule: Rule::data_name,
            positives: vec![Rule::data_name],
            negatives: vec![],
            pos: 0
        }
    }

    // data_value
    #[test]
    fn data_value() {
        parses_to! {
            parser: StarParser,
            input:  "ABC",
            rule:   Rule::non_quoted_text_string,
            tokens: [
                non_quoted_text_string(0, 3)
            ]
        }
        

        parses_to! {
            parser: StarParser,
            input:  "A_BC",
            rule:   Rule::non_quoted_text_string,
            tokens: [
                non_quoted_text_string(0, 4)
            ]
        }
        

        fails_with! {
            parser: StarParser,
            input: "_ABC",
            rule: Rule::non_quoted_text_string,
            positives: vec![Rule::non_quoted_text_string],
            negatives: vec![],
            pos: 0
        }
    }

    #[test]
    fn data() {
        parses_to! {
            parser: StarParser,
            input:  "_test 123",
            rule:   Rule::data,
            tokens: [
                data(0, 9, [
                    data_name(0, 5),
                    non_quoted_text_string(6, 9)
                ])
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "_test \"123\"",
            rule:   Rule::data,
            tokens: [
                data(0, 11, [
                    data_name(0, 5),
                    double_quote_string(6, 11),
                ])
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "_test 'a'",
            rule:   Rule::data,
            tokens: [
                data(0, 9, [
                    data_name(0, 5),
                    single_quote_string(6, 9),
                ])
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "_test $test",
            rule:   Rule::data,
            tokens: [
                data(0, 11, [
                    data_name(0, 5),
                    frame_code(6, 11),
                ])
            ]
        }

        fails_with! {
            parser: StarParser,
            input: "test",
            rule: Rule::data,
            positives: vec![Rule::data],
            negatives: vec![],
            pos: 0
        }

        fails_with! {
            parser: StarParser,
            input: "_test",
            rule: Rule::data,
            positives: vec![
                Rule::new_line_semi_colon,
                Rule::non_quoted_text_string,
                Rule::double_quote_string,
                Rule::single_quote_string,
                Rule::frame_code
            ],
            negatives: vec![],
            pos: 5
        }

        fails_with! {
            parser: StarParser,
            input: "_test _test",
            rule: Rule::data,
            positives: vec![
                Rule::new_line_semi_colon,
                Rule::non_quoted_text_string,
                Rule::double_quote_string,
                Rule::single_quote_string,
                Rule::frame_code
            ],
            negatives: vec![],
            pos: 6
        }
    }


    // single_quoted_string
    #[test]
    fn single_quoted_string() {
        parses_to! {
            parser: StarParser,
            input:  "''",
            rule:   Rule::single_quote_string,
            tokens: [
                single_quote_string(0, 2)
            ]
        }

         parses_to! {
            parser: StarParser,
            input:  "''a'",
            rule:   Rule::single_quote_string,
            tokens: [
                single_quote_string(0, 4)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "'\"'",
            rule:   Rule::single_quote_string,
            tokens: [
                single_quote_string(0, 3)
            ]
        }


        parses_to! {
            parser: StarParser,
            input:  "' \t '",
            rule:   Rule::single_quote_string,
            tokens: [
                single_quote_string(0, 5)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "'ABC'",
            rule:   Rule::single_quote_string,
            tokens: [
                single_quote_string(0, 5)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "'A BC'",
            rule:   Rule::single_quote_string,
            tokens: [
                single_quote_string(0, 6)
            ]
        }

        fails_with! {
            parser: StarParser,
            input: "ABC\n",
            rule: Rule::single_quote_string,
            positives: vec![Rule::single_quote_string],
            negatives: vec![],
            pos: 0
        }

        fails_with! {
            parser: StarParser,
            input: "\rABC",
            rule: Rule::single_quote_string,
            positives: vec![Rule::single_quote_string],
            negatives: vec![],
            pos: 0
        }

        fails_with! {
            parser: StarParser,
            input: "'ABC",
            rule: Rule::single_quote_string,
            positives: vec![Rule::single_quote_string],
            negatives: vec![],
            pos: 0
        }

        fails_with! {
            parser: StarParser,
            input: "ABC'",
            rule: Rule::single_quote_string,
            positives: vec![Rule::single_quote_string],
            negatives: vec![],
            pos: 0
        }
    }

    // double_quoted_string
    #[test]
    fn double_quoted_string() {
        parses_to! {
            parser: StarParser,
            input:  "\"\"",
            rule:   Rule::double_quote_string,
            tokens: [
                double_quote_string(0, 2)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "\"\"a\"",
            rule:   Rule::double_quote_string,
            tokens: [
                double_quote_string(0, 4)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "\"ABC\"",
            rule:   Rule::double_quote_string,
            tokens: [
                double_quote_string(0, 5)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "\"A BC\"",
            rule:   Rule::double_quote_string,
            tokens: [
                double_quote_string(0, 6)
            ]
        }

        fails_with! {
            parser: StarParser,
            input: "ABC\n",
            rule: Rule::double_quote_string,
            positives: vec![Rule::double_quote_string],
            negatives: vec![],
            pos: 0
        }

        fails_with! {
            parser: StarParser,
            input: "\rABC",
            rule: Rule::double_quote_string,
            positives: vec![Rule::double_quote_string],
            negatives: vec![],
            pos: 0
        }

        fails_with! {
            parser: StarParser,
            input: "\"ABC",
            rule: Rule::double_quote_string,
            positives: vec![Rule::double_quote_string],
            negatives: vec![],
            pos: 0
        }

        fails_with! {
            parser: StarParser,
            input: "ABC\"",
            rule: Rule::double_quote_string,
            positives: vec![Rule::double_quote_string],
            negatives: vec![],
            pos: 0
        }

        // TODO add more tests of patalogical strings
        //      unify single and double quote string tests..
    }

    // frame_code
    #[test]
    fn frame_code() {
        parses_to! {
            parser: StarParser,
            input:  "$frame_code",
            rule:   Rule::frame_code,
            tokens: [
                frame_code(0, 11)
            ]
        }

        fails_with! {
            parser: StarParser,
            input: "$ ",
            rule: Rule::frame_code,
            positives: vec![Rule::frame_code],
            negatives: vec![],
            pos: 0
        }
    }

    // data_heading
    #[test]
    fn data_heading() {
        parses_to! {
            parser: StarParser,
            input:  "data_ABC",
            rule:   Rule::data_heading,
            tokens: [
                data_heading(0, 8)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "DaTa_ABC",
            rule:   Rule::data_heading,
            tokens: [
                data_heading(0, 8)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "DaTa_\"ABC",
            rule:   Rule::data_heading,
            tokens: [
                data_heading(0, 9)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "DaTa_'ABC",
            rule:   Rule::data_heading,
            tokens: [
                data_heading(0, 9)
            ]
        }


        fails_with! {
            parser: StarParser,
            input: "data_",
            rule: Rule::data_heading,
            positives: vec![Rule::data_heading],
            negatives: vec![],
            pos: 0
        }
    }

    #[test]
    fn basic_data_block() {
        let test_string = "data_1              \
                                 _test     123       \
                                 _test_1   \"123\"   \
                                 _test_3   ' a b '   \
                                 _test_4   $test_1   \
                                                     ";
        parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::data_block,
            tokens: [
                data_block(0, 95, [
                    data_heading(0, 6),
                    data(20, 33, [data_name(20, 25), non_quoted_text_string(30, 33)]),
                    data(40, 55, [data_name(40, 47), double_quote_string(50, 55)]),
                    data(58, 75, [data_name(58, 65), single_quote_string(68, 75)]),
                    data(78, 95, [data_name(78, 85), frame_code(88, 95)])
                ])
            ]

        }

        let test_string = "data_1              \
                                 test     123        ";

        fails_with! {
            parser: StarParser,
            input: test_string,
            rule: Rule::data_block,
            positives: vec![Rule::data, Rule::save_heading],
            negatives: vec![],
            pos: 20
        }

        let test_string = "data_1              \
                                 _test     _123      ";

        fails_with! {
            parser: StarParser,
            input: test_string,
            rule: Rule::data_block,
            positives: vec![
                Rule::new_line_semi_colon,
                Rule::non_quoted_text_string,
                Rule::double_quote_string,
                Rule::single_quote_string,
                Rule::frame_code
            ],
            negatives: vec![],
            pos: 30
        }
    }
    
    // save_heading
    #[test]
    fn save_heading() {
        parses_to! {
            parser: StarParser,
            input:  "save_ABC",
            rule:   Rule::save_heading,
            tokens: [
                save_heading(0, 8)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "save_ABC",
            rule:   Rule::save_heading,
            tokens: [
                save_heading(0, 8)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "save_\"ABC",
            rule:   Rule::save_heading,
            tokens: [
                save_heading(0, 9)
            ]
        }

        parses_to! {
            parser: StarParser,
            input:  "save_'ABC",
            rule:   Rule::save_heading,
            tokens: [
                save_heading(0, 9)
            ]
        }

        
        fails_with! {
            parser: StarParser,
            input: "save_",
            rule: Rule::save_heading,
            positives: vec![Rule::save_heading],
            negatives: vec![],
            pos: 0
        }
            
    }

    #[test]
    fn data_loop() {
        let test_string = "loop_                                  \
                                     _atom_name                         \
                                     _atomic_mass_ratio                 \
                                         1H	    1.007825031898(14)      \
                                         2H	    1.0070508889220(75)     \
                                         3H	    1.005349760440(27)      \
                                         3He	1.005343107322(20)      \
                                stop_                                  ";

        parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::data_loop,
            tokens: [
                data_loop(0, 235, [
                    loop_keyword(0, 5),
                    data_loop_definition(39, 92, [
                        data_name(39, 49),
                        data_name(74, 92)
                    ]
                ),
                data_loop_values(109, 235, [
                        non_quoted_text_string(109, 111), non_quoted_text_string(116, 134),
                        non_quoted_text_string(140, 142), non_quoted_text_string(147, 166),
                        non_quoted_text_string(171, 173), non_quoted_text_string(178, 196),
                        non_quoted_text_string(202, 205), non_quoted_text_string(206, 224),
                        stop_keyword(230, 235)
                    ])
                ])
            ]

        }

        let test_string =  "loop_                                        \
                                _atomic_name                                   \
                                    loop_                                      \
                                        _level_scheme                          \
                                        _level_energy                          \
                                            loop_                              \
                                               _function_exponent              \
                                               _function_coefficient           \
                                    hydrogen                                   \
                                      \"(2)->[2] \" -0.485813                  \
                                         1.3324838E+01    1.0                  \
                                         2.0152720-01     1.0 stop_            \
                                      \"(2)->[2]\"  -0.485813                  \
                                         1.3326990E+01    1.0                  \
                                         2.0154600E-01    1.0 stop_            \
                                      \"(2)->[1]\"  -0.485813                  \
                                         1.3324800E-01    2.7440850-01         \
                                         2.0152870E-01    8.2122540-01 stop_   \
                                      \"(3)->[2]\"  -0.496979                  \
                                         4.5018000+00    1.5628500E-01         \
                                         6.8144400E-01   9.0469100E-01         \
                                         1.5139800E-01   1.0000000E+01 stop_   \
                                   stop_                                       ";

        parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::data_loop,
            tokens: [
                data_loop(0, 858, [
                    loop_keyword(0, 5),
                    data_loop_definition(45, 312, [
                        data_name(45, 57),
                        nested_loop(92, 312, [
                            loop_keyword(92, 97),
                            data_name(135, 148),
                            data_name(174, 187),
                            nested_loop(213, 312, [
                                loop_keyword(213, 218),
                                data_name(248, 266),
                                data_name(280, 301)])
                        ])
                    ]),
                    data_loop_values(312, 858, [
                        non_quoted_text_string(312, 320),
                          double_quote_string(355, 366),      non_quoted_text_string(367, 376),
                            non_quoted_text_string(394, 407), non_quoted_text_string(411, 414),
                            non_quoted_text_string(432, 444), non_quoted_text_string(449, 452), stop_keyword(453, 458),

                          double_quote_string(470, 480),      non_quoted_text_string(482, 491),
                            non_quoted_text_string(509, 522), non_quoted_text_string(526, 529),
                            non_quoted_text_string(547, 560), non_quoted_text_string(564, 567), stop_keyword(568, 573),

                          double_quote_string(585, 595),      non_quoted_text_string(597, 606),
                            non_quoted_text_string(624, 637), non_quoted_text_string(641, 653),
                            non_quoted_text_string(662, 675), non_quoted_text_string(679, 691), stop_keyword(692, 697),

                          double_quote_string(700, 710),      non_quoted_text_string(712, 721),
                            non_quoted_text_string(739, 751), non_quoted_text_string(755, 768),
                            non_quoted_text_string(777, 790), non_quoted_text_string(793, 806),
                            non_quoted_text_string(815, 828), non_quoted_text_string(831, 844), stop_keyword(845, 850),
                        stop_keyword(853, 858)])
                ])
            ]

        }
    }

    #[test]
    fn data_block_with_save_frame() {

        // data_frame with save_frames from
        // Extensions to the STAR File Syntax Nick Spadaccini* and Sydney R. Hall
        // dx.doi.org/10.1021/ci300074v | J. Chem. Inf. Model. 2012, 52, 1901−1906
        let test_string = "data_experiment                    \
                                    _images.collected 1289         \
                                    _images_refined   894          \
                                save_fragment_1                    \
                                    _molecular_weight  234         \
                                    _max_bond_length   2.7         \
                                save_                              \
                                save_fragment_2                    \
                                    _molecular_weight  23          \
                                    _max_bond_length   1.1         \
                                    _fragment_parent   $fragment_1 \
                                save_                              ";


        parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::data_block,
            tokens: [
                data_block(0, 362, [
                    data_heading(0, 15),
                    data(35, 57, [data_name(35, 52), non_quoted_text_string(53, 57)]),
                    data(66, 87, [data_name(66, 81), non_quoted_text_string(84, 87)]),
                    save_frame(97, 199, [
                        save_heading(97, 112),
                        data(132, 154, [data_name(132, 149), non_quoted_text_string(151, 154)]),
                        data(163, 185, [data_name(163, 179), non_quoted_text_string(182, 185)]),
                    save_keyword(194, 199)]),
                    save_frame(229, 362, [
                        save_heading(229, 244),
                        data(264, 285, [data_name(264, 281), non_quoted_text_string(283, 285)]),
                        data(295, 317, [data_name(295, 311), non_quoted_text_string(314, 317)]),
                        data(326, 356, [data_name(326, 342), frame_code(345, 356)]),
                    save_keyword(357, 362)])
                ])
            ]

        }

        // save frames and data items can be freely interspersed [true in pynmrstar?]
        // <data_block> ::= <data_heading> <data_block_body>+
        // <data_block_body> ::= {<data> | <save_frame> }+
        // note: is the double repetition really neccessary ie <data_block_body>+  and
        //      {<data> | <save_frame> }+
        let test_string = "data_experiment                    \
                                                                   \
                                save_fragment_1                    \
                                    _molecular_weight  234         \
                                    _max_bond_length   2.7         \
                                save_                              \
                                                                   \
                                _images.collected 1289             \
                                _images_refined   894              \
                                                                   \
                                save_fragment_2                    \
                                    _molecular_weight  23          \
                                    _max_bond_length   1.1         \
                                    _fragment_parent   $fragment_1 \
                                save_                              ";


        parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::data_block,
            tokens: [
                data_block(0, 370, [
                    data_heading(0, 15),

                    save_frame(35, 137, [
                        save_heading(35, 50),
                        data(70, 92, [data_name(70, 87), non_quoted_text_string(89, 92)]),
                        data(101, 123, [data_name(101, 117), non_quoted_text_string(120, 123)]),
                    save_keyword(132, 137)]),

                    data(167, 189, [data_name(167, 184), non_quoted_text_string(185, 189)]),
                    data(202, 223, [data_name(202, 217), non_quoted_text_string(220, 223)]),

                    save_frame(237, 370, [
                        save_heading(237, 252),
                        data(272, 293, [data_name(272, 289), non_quoted_text_string(291, 293)]),
                        data(303, 325, [data_name(303, 319), non_quoted_text_string(322, 325)]),
                        data(334, 364, [data_name(334, 350), frame_code(353, 364)]),
                    save_keyword(365, 370)])
                ])
            ]
        }

        // just a test with only save frames
        let test_string = "data_experiment                    \
                                                                   \
                                save_fragment_1                    \
                                    _molecular_weight  234         \
                                    _max_bond_length   2.7         \
                                save_                              \
                                                                   \
                                save_fragment_2                    \
                                    _molecular_weight  23          \
                                    _max_bond_length   1.1         \
                                    _fragment_parent   $fragment_1 \
                                save_                              ";


        parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::data_block,
            tokens: [data_block(0, 300, [
                data_heading(0, 15),
                save_frame(35, 137, [
                    save_heading(35, 50),
                    data(70, 92, [data_name(70, 87), non_quoted_text_string(89, 92)]),
                    data(101, 123, [data_name(101, 117), non_quoted_text_string(120, 123)]),
                    save_keyword(132, 137)
                ]),
                save_frame(167, 300, [
                    save_heading(167, 182),
                    data(202, 223, [data_name(202, 219), non_quoted_text_string(221, 223)]),
                    data(233, 255, [data_name(233, 249), non_quoted_text_string(252, 255)]),
                    data(264, 294, [data_name(264, 280), frame_code(283, 294)]),
                    save_keyword(295, 300)])
                ])
            ]

        }

        // a test with only save frames and interleaved data
        let test_string = "data_experiment                    \
                                                                   \
                                save_fragment_1                    \
                                    _molecular_weight  234         \
                                    _max_bond_length   2.7         \
                                    loop_                          \
                                        _atom_identity_node        \
                                        _atom_identity_symbol      \
                                        1 C                        \
                                        2 C                        \
                                        3 C                        \
                                    stop_                          \
                                    loop_                          \
                                        _atom_identity_node        \
                                        _atom_identity_symbol      \
                                        1 C                        \
                                        2 C                        \
                                        3 C                        \
                                   stop_                           \
                                    _molecular_weight  234         \
                                    _max_bond_length   2.7         \
                                save_                              \
                                                                   ";

        parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::data_block,
            tokens: [data_block(0, 594, [
                data_heading(0, 15),
                save_frame(35, 594, [
                    save_heading(35, 50),
                    data(70, 92, [data_name(70, 87), non_quoted_text_string(89, 92)]),
                    data(101, 123, [data_name(101, 117), non_quoted_text_string(120, 123)]),
                    data(132, 303, [data_loop(132, 303, [
                        loop_keyword(132, 137),
                        data_loop_definition(163, 211, [
                            data_name(163, 182),
                            data_name(190, 211)
                        ]),
                        data_loop_values(217, 303, [
                            non_quoted_text_string(217, 218), non_quoted_text_string(219, 220),
                            non_quoted_text_string(244, 245), non_quoted_text_string(246, 247),
                            non_quoted_text_string(271, 272), non_quoted_text_string(273, 274),
                            stop_keyword(298, 303)]
                        )]
                    )]
                ),
                data(329, 500, [
                    data_loop(329, 500, [
                        loop_keyword(329, 334),
                        data_loop_definition(360, 408, [
                            data_name(360, 379),
                            data_name(387, 408)
                        ]),
                    data_loop_values(414, 500, [
                            non_quoted_text_string(414, 415), non_quoted_text_string(416, 417),
                            non_quoted_text_string(441, 442), non_quoted_text_string(443, 444),
                            non_quoted_text_string(468, 469), non_quoted_text_string(470, 471),
                            stop_keyword(495, 500)
                        ])
                    ])
                ]),
                data(527, 549, [data_name(527, 544), non_quoted_text_string(546, 549)]),
                data(558, 580, [data_name(558, 574), non_quoted_text_string(577, 580)]),
                save_keyword(589, 594)])])]

        }


        // a test with only save frames and interleaved data
        let test_string = "data_experiment                    \
                                                                   \
                                save_fragment_1                    \
                                    _molecular_weight  234         \
                                    _max_bond_length   2.7         \
                                    loop_                          \
                                        _atom_identity_node        \
                                        _atom_identity_symbol      \
                                        1 C                        \
                                        2 C                        \
                                        3 C                        \
                                    stop_                          \
                                    _molecular_weight  456         \
                                    _max_bond_length   3.2         \
                                    loop_                          \
                                        _atom_identity_node        \
                                        _atom_identity_symbol      \
                                        4 N                        \
                                        5 N                        \
                                        6 N                        \
                                   stop_                           \
                                save_                              \
                                                                   ";



        parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::data_block,
            tokens: [
                data_block(0, 594, [
                    data_heading(0, 15),
                    save_frame(35, 594, [
                        save_heading(35, 50),
                        data(70, 92, [data_name(70, 87), non_quoted_text_string(89, 92)]),
                        data(101, 123, [data_name(101, 117), non_quoted_text_string(120, 123)]),
                        data(132, 303, [
                        data_loop(132, 303, [
                            loop_keyword(132, 137),
                            data_loop_definition(163, 211, [
                                data_name(163, 182),
                                data_name(190, 211)
                            ]),
                            data_loop_values(217, 303, [
                                non_quoted_text_string(217, 218), non_quoted_text_string(219, 220),
                                non_quoted_text_string(244, 245), non_quoted_text_string(246, 247),
                                non_quoted_text_string(271, 272), non_quoted_text_string(273, 274),
                                stop_keyword(298, 303)])
                            ])
                        ]),
                        data(329, 351, [data_name(329, 346), non_quoted_text_string(348, 351)]),
                        data(360, 382, [data_name(360, 376), non_quoted_text_string(379, 382)]),
                        data(391, 562, [data_loop(391, 562, [
                            loop_keyword(391, 396),
                            data_loop_definition(422, 470, [
                                data_name(422, 441),
                                data_name(449, 470)
                            ]),
                            data_loop_values(476, 562, [
                                non_quoted_text_string(476, 477), non_quoted_text_string(478, 479),
                                non_quoted_text_string(503, 504), non_quoted_text_string(505, 506),
                                non_quoted_text_string(530, 531), non_quoted_text_string(532, 533),
                                stop_keyword(557, 562)
                            ])
                        ])
                    ]),
                    save_keyword(589, 594)])
                ]
            )]
        }
    }

    #[test]
    fn global_block() {

        let test_string = "global_                  \
                                     _compound.trial 4    \
                                     _compound.source FDA ";



        parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::global_block,
            tokens: [
                global_block(0, 66, [
                    global_keyword(0, 7),
                    data(25, 42, [data_name(25, 40), non_quoted_text_string(41, 42)]),
                    data(46, 66, [data_name(46, 62), non_quoted_text_string(63, 66)])
                ])
            ]

        }

        // a global block cannot cointain a save frame only a datablock can
        let test_string = "global_                          \
                                     save_fragment_1              \
                                         _molecular_weight  234   \
                                         _max_bond_length   2.7   \
                                     save_                        \
                                     _compound.trial 4            \
                                     _compound.source FDA"        ;

        fails_with! {
            parser: StarParser,
            input: test_string,
            rule: Rule::global_block,
            positives: vec![Rule::data],
            negatives: vec![],
            pos: 33
        }


        let test_string = "global_                    \
                                     _compound.trial 4     \
                                     _compound.source FDA  \
                                     loop_                 \
                                         _atom_name        \
                                         hydrogen          \
                                         oxygen           ";


        parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::global_block,
            tokens: [
                global_block(0, 135, [
                    global_keyword(0, 7),
                    data(27, 44, [data_name(27, 42), non_quoted_text_string(43, 44)]),
                    data(49, 69, [data_name(49, 65), non_quoted_text_string(66, 69)]),
                    data(71, 135, [
                        data_loop(71, 135, [
                            loop_keyword(71, 76),
                            data_loop_definition(93, 103,[
                                data_name(93, 103)
                            ]),
                            data_loop_values(111, 135, [
                                non_quoted_text_string(111, 119),
                                non_quoted_text_string(129, 135)
                            ])
                        ])
                    ])
                ])
            ]
        }
    }

    #[test]
    fn semi_colon_bounded_string() {
        let test_string = "\n;a string \n another \n;";


        parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::semi_colon_bounded_text_string,
            tokens: [
                semi_colon_bounded_text_string(0, 23, [
                    new_line_semi_colon(0, 2),
                    line_of_text_newline(2, 12),
                    line_of_text_new_line_semi_colon(13, 23)
                ])
            ]

        }

        let test_string = "\n;a string ;\n another \n;";


         parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::semi_colon_bounded_text_string,
            tokens: [
                 semi_colon_bounded_text_string(0, 24, [
                     new_line_semi_colon(0, 2),
                     line_of_text_newline(2, 13),
                     line_of_text_new_line_semi_colon(14, 24)]
                 )
             ]

        }

        let test_string = "\n ;a string ;\n another \n;";

         fails_with! {
            parser: StarParser,
            input: test_string,
            rule: Rule::semi_colon_bounded_text_string,
            positives: vec![Rule::new_line_semi_colon],
            negatives: vec![],
            pos: 0
        }

        let test_string = "\n;a string ;\n another \n ;";

         parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::semi_colon_bounded_text_string,
            tokens: [
                 semi_colon_bounded_text_string(0, 23, [
                     new_line_semi_colon(0, 2),
                     line_of_text_newline(2, 13),
                     line_of_text_newline(14, 23)
                 ])
             ]

        }

    }

    #[test]
    fn star_document() {
        let test_string = "                                                            \
                                global_                                                      \
                                    _compound.trial             4                            \
                                    _compound.source            FDA                          \
                                data_synthesis                                               \
                                    _sample.length              5.84                         \
                                    _sample.shape               'needle'                     \
                                    _solvent.base               Methanol                     \
                                    _sample.orientation         '[1 0 2]'                    \
                                global_                                                      \
                                    _experimental.source        'ConvBeamEl'                 \
                                    _experimental.date          2011-06-09                   \
                                data_experiment                                              \
                                     _images.collected          1289                         \
                                     _images_refined            894                          \
                                save_fragment_1                                              \
                                     _molecular_weight          234                          \
                                     _max_bond_length           2.7                          \
                                save_                                                        \
                                save_fragment_2                                              \
                                     _molecular_weight          23                           \
                                     _max_bond_length           1.1                          \
                                     _fragment_parent           $fragment_1                  \
                                save_                                                        \
                                data_publication                                             \
                                     _author.details            'A.B.Smith'                  \
                                     _author.laboratory         'LLNL'                       \
                                     _journal.page              1901-1906                    \
                                     _abstract                 'the experimental results'    \
                                save_fragment_3                                              \
                                     _transition_count         3                             \
                                     loop_                                                   \
                                         _atomic_name                                        \
                                             loop_                                           \
                                                 _level_scheme                               \
                                                 _level_energy                               \
                                                     loop_                                   \
                                                        _function_exponent                   \
                                                        _function_coefficient                \
                                             hydrogen                                        \
                                                 \"(2)->[2] \" -0.485813                     \
                                                    1.3324838E+01    1.0                     \
                                                    2.0152720-01     1.0 stop_               \
                                                 \"(2)->[2]\"  -0.485813                     \
                                                    1.3326990E+01    1.0                     \
                                                    2.0154600E-01    1.0 stop_               \
                                                 \"(2)->[1]\"  -0.485813                     \
                                                    1.3324800E-01    2.7440850-01            \
                                                    2.0152870E-01    8.2122540-01 stop_      \
                                                 \"(3)->[2]\"  -0.496979                     \
                                                    4.5018000+00    1.5628500E-01            \
                                                    6.8144400E-01   9.0469100E-01            \
                                                    1.5139800E-01   1.0000000E+01 stop_      \
                                            stop_                                            \
                                         save_                                               \
                                ";

        parses_to! {
            parser: StarParser,
            input:  test_string,
            rule:   Rule::star_file,
            tokens: [
                star_file(0, 2842, [
                    global_block(60, 209, [
                        global_keyword(60, 67),
                        data(121, 150, [data_name(121, 136), non_quoted_text_string(149, 150)]),
                        data(178, 209, [data_name(178, 194), non_quoted_text_string(206, 209)])
                    ]),
                    data_block(235, 504, [
                        data_heading(235, 249),
                        data(296, 328, [data_name(296, 310), non_quoted_text_string(324, 328)]),
                        data(353, 389, [data_name(353, 366), single_quote_string(381, 389)]),
                        data(410, 446, [data_name(410, 423), non_quoted_text_string(438, 446)]),
                        data(467, 504, [data_name(467, 486), single_quote_string(495, 504)])
                    ]),
                    global_block(524, 680, [
                        global_keyword(524, 531),
                        data(585, 625, [data_name(585, 605), single_quote_string(613, 625)]),
                        data(642, 680, [data_name(642, 660), non_quoted_text_string(670, 680)])
                    ]),
                    data_block(699, 1340, [
                        data_heading(699, 714),
                        data(760, 791, [data_name(760, 777), non_quoted_text_string(787, 791)]),
                        data(816, 846, [data_name(816, 831), non_quoted_text_string(843, 846)]),
                        save_frame(872, 1050, [
                            save_heading(872, 887),
                            data(933, 963, [data_name(933, 950), non_quoted_text_string(960, 963)]),
                            data(989, 1019, [data_name(989, 1005), non_quoted_text_string(1016, 1019)]),
                            save_keyword(1045, 1050)
                        ]),
                        save_frame(1106, 1340, [
                            save_heading(1106, 1121),
                            data(1167, 1196, [data_name(1167, 1184), non_quoted_text_string(1194, 1196)]),
                            data(1223, 1253, [data_name(1223, 1239), non_quoted_text_string(1250, 1253)]),
                            data(1279, 1317, [data_name(1279, 1295), frame_code(1306, 1317)]),
                            save_keyword(1335, 1340)
                        ])
                    ]),
                    data_block(1396, 2795, [
                        data_heading(1396, 1412),
                        data(1457, 1495, [data_name(1457, 1472), single_quote_string(1484, 1495)]),
                        data(1513, 1546, [data_name(1513, 1531), single_quote_string(1540, 1546)]),
                        data(1569, 1605, [data_name(1569, 1582), non_quoted_text_string(1596, 1605)]),
                        data(1625, 1677, [data_name(1625, 1634), single_quote_string(1651, 1677)]),
                        save_frame(1681, 2795, [
                            save_heading(1681, 1696),
                            data(1742, 1769, [data_name(1742, 1759), non_quoted_text_string(1768, 1769)]),
                            data(1798, 2746, [
                                data_loop(1798, 2746, [
                                    loop_keyword(1798, 1803),
                                    data_loop_definition(1854, 2156, [
                                        data_name(1854, 1866),
                                        nested_loop(1906, 2156, [
                                            loop_keyword(1906, 1911),
                                            data_name(1954, 1967),
                                            data_name(1998, 2011),
                                            nested_loop(2042, 2156, [
                                                loop_keyword(2042, 2047),
                                                data_name(2082, 2100),
                                                data_name(2119, 2140)
                                            ])
                                        ])
                                    ]),
                                    data_loop_values(2156, 2746, [
                                        non_quoted_text_string(2156, 2164),
                                            double_quote_string(2204, 2215), non_quoted_text_string(2216, 2225),
                                                non_quoted_text_string(2246, 2259), non_quoted_text_string(2263, 2266),
                                                non_quoted_text_string(2287, 2299), non_quoted_text_string(2304, 2307),
                                            stop_keyword(2308, 2313),
                                            double_quote_string(2328, 2338), non_quoted_text_string(2340, 2349),
                                                non_quoted_text_string(2370, 2383), non_quoted_text_string(2387, 2390),
                                                non_quoted_text_string(2411, 2424), non_quoted_text_string(2428, 2431),
                                            stop_keyword(2432, 2437),
                                            double_quote_string(2452, 2462), non_quoted_text_string(2464, 2473),
                                                non_quoted_text_string(2494, 2507),  non_quoted_text_string(2511, 2523),
                                                non_quoted_text_string(2535, 2548), non_quoted_text_string(2552, 2564),
                                            stop_keyword(2565, 2570),
                                            double_quote_string(2576, 2586), non_quoted_text_string(2588, 2597),
                                                non_quoted_text_string(2618, 2630), non_quoted_text_string(2634, 2647),
                                                non_quoted_text_string(2659, 2672), non_quoted_text_string(2675, 2688),
                                                non_quoted_text_string(2700, 2713), non_quoted_text_string(2716, 2729),
                                            stop_keyword(2730, 2735),
                                        stop_keyword(2741, 2746)
                                    ])
                                ])
                            ]),
                            save_keyword(2790, 2795)

                        ])
                    ]),

                    EOI(2842, 2842)
                ])
            ]
        }
    }

    #[test]
    fn semi_colon_bounded_string_full() {
        let file_path = "tests/test_data/simple_comma_string.str";
        let test_string = std::fs::read_to_string(file_path).unwrap();

         let successful_parse = StarParser::parse(Rule::star_file, &test_string);
        // println!("{:?}", successful_parse);
        println!("{}", successful_parse.unwrap());

        parses_to! {
            parser: StarParser,
            input:  &test_string,
            rule:   Rule::star_file,
            tokens: [
                star_file(0, 189, [
                    data_block(0, 189, [
                        data_heading(0, 16),
                        data(22, 60, [data_name(22, 37), single_quote_string(49, 60)]),
                        data(66, 99, [data_name(66, 84), single_quote_string(93, 99)]),
                        data(105, 141, [data_name(105, 118), non_quoted_text_string(132, 141)]),
                        data(147, 189, [data_name(147, 156), semi_colon_bounded_text_string(156, 189, [
                            new_line_semi_colon(156, 158),
                            line_of_text_newline(159, 184),
                            line_of_text_new_line_semi_colon(184, 189)])
                        ])
                    ]),
                    EOI(189, 189)
                ])
            ]
        }

    }

    #[test]
    fn semi_colon_bounded_string_full_bad() {
        let file_path = "tests/test_data/simple_comma_string_bad.str";
        let test_string = std::fs::read_to_string(file_path).unwrap();


        fails_with! {
            parser: StarParser,
            input: &test_string,
            rule: Rule::star_file,
            positives: vec![
                Rule::EOI,
                Rule::global_keyword,
                Rule::data,
                Rule::data_heading,
                Rule::save_heading
            ],
            negatives: vec![],
            pos: 160
        }
    }
    //TODO: add tests of data blocks  with save frames and loops
}

/*
notes
1. v1. allows for an empty star file v2 must have at least on data_block
2. NMRStar [pynmrstar] does not allow data inside a data_block before save frames
3. version 2 and version 1 allow data and save frames to be interspersed in any order
4. NMRStar doesn't allow for empty data loops
5. save_ data_ global_ etc are case insenitive [certainly in v2]
6. it appears v2 doesn't require loops to have data [pynmrstar does but NEF doesn't!]
7. quoted strings allow embedded quotation marks as long as they are not proceeded/followed/surrounded by a space...
8. cif v1 has some interesting conventions on item element lengths etc
9. pathalogical strings with ""a" amd ''a' are allowed but """ and ''' are not
   and neither are "a"" or ''a' the rule is <D_quote> <no_blank_char> | <not_a_D_quote>
*/