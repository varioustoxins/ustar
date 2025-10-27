## TODO / Notes

1. add tests of data blocks with save frames and loops
2. v1. allows for an empty star file v2 must have at least one data_block
3. NMRStar [pynmrstar] does not allow data inside a data_block before save frames
4. version 2 and version 1 allow data and save frames to be interspersed in any order
5. NMRStar doesn't allow for empty data loops
6. save_ data_ global_ etc are case insenitive [certainly in v2]
7. it appears v2 doesn't require loops to have data [pynmrstar does but NEF doesn't!]
8. quoted strings allow embedded quotation marks as long as they are not proceeded/followed/surrounded by a space...
9. cif v1 has some interesting conventions on item element lengths etc
10. pathalogical strings with ""a" amd ''a' are allowed but """ and ''' are not and neither are "a"" or ''a' the rule is <D_quote> <no_blank_char> | <not_a_D_quote>
