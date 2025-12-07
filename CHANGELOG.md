## 0.1.4
- replace line numbers with LineColumn positions in SASContentHandler trait
- add global block reporting to SAS interface  
- add start_stream and end_stream callbacks to SAS interface
- improve snapshot testing utilities with external diff support
- add support for loops with empty data_loop_values
- fix nested loop handling in SAS walker
- improve delimiter handling in SAS walker tests

## 0.1.3
- added fast line number and column lookup
- use snapshots for testing
- split project into a tools crate and library crate

## 0.1.2
- fixed a bug in SASWalker preventing early extit from parsing by the user

## 0.1.1 
- package name crates.io is now ustar-parser to avoid a clash with the moribund ustar package

## 0.1.0
- initial release of ustar mostly a demonstration but all major components work

