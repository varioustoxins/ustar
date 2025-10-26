use ustar::{StarParser, Rule};
use pest::Parser;

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
    println!("{}", successful_parse.unwrap());
}