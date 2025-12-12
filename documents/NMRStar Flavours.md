NMRStar Flavours

V1 - the original ascii version
V2 - adds Lists, Dictionaries and triple quoted strings  & unicode - no used Triple Quoted string are implmented by Python SAS

NEF Star Files

- Only one data section per file
- No globals
- No loops or data outside SaveFrames
- Loops cannot be nested
- Loops end with required stop_
- Loops can be empty 
- ascii only [is extended supported?]

NMRStar files

- as with NMR Star except that loops should not be empty

## CIF File format - https://www.chem.gla.ac.uk/~louis/software/wingx/hlp/cif102.htm
- ascii only [not extended]
- line length limit of 80 characters [relaxed in version 2]
- allows multiple data blocks but their names must be unique
- allows values and loops inside a datablock
- no globals 
- no save frames
- loops cannot be nested
- no requirements for stop_ in loops [they aren't mentioned] 
- Data names and block codes may not exceed 32 characters in length, and should be treated as case-insensitive. NOTE This only applies to CIF's which conform to the Core Dictionary Version 1. There is NO formal restriction in Version 2 (though in practise the length is restricted to 76 characters)

## dicts  - DDL1 / DDLm

## DDL 
- allows multiple data sections
- stops_'s are not required
- allows nested loops?
- allows globals 

## cif dictionaries
- more example dictionary files! https://www.iucr.org/resources/cif/dictionaries
- contains single datablocks 
- contain values and loops at the datablock level
- can contain saveframes
- should be ascii but some dictionaies are 'naughty'!
- loops do not require stop_'s

## DDLm
- can use vectors [] in lists
- uses nested save_frames which are not part of the star v1 format
- DDLm is an updated version
- save_'s at the end of save frames are not required

## mmcif
-