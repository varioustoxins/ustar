# Minimal STAR file with nested loop for testing

#data_test_nested_loop
#
#loop_
#    _outer_tag_1
#    _outer_tag_2
#    loop_
#        _inner_tag_1
#        _inner_tag_2
#    stop_
#    OUT1  OUT2
#        IN1  IN2
#        IN3  IN4
#    OUT3  OUT4
#        IN5  IN6
#    stop_
stop_

data_test_hall_loop

loop_
    _atom_identity_node
    _atom_identity_symbol
    loop_
        _atom_bond_node_1
        _atom_bond_node_2
        _atom_bond_order
        A1 B1 1 2 single              stop_
        A2 B2 1 6 double 30 40 triple stop_
        A3 B3 1 7 single              stop_