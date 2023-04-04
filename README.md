A simple incremental compiler using the [Salsa](https://github.com/salsa-rs/salsa) incremental computation library (new "2022" version).

This is the [calc](https://github.com/salsa-rs/salsa/tree/master/examples-2022/calc) example modified in some ways:

- The parser was rewritten to use `lalrpop` (and syntax changed a bit to avoid dealing with whitespace sensitivity)

- The span representation was changed to make sure modifying a function doesn't change Spans in the functions below it, achieving true per-function incremental compilation.
  - Note: we don't yet resolve them back to the true source location - this should be done during diagnostic reporting.

## Try it out

Run `cargo run program1.txt program2.txt`. The programs differ only in func1 and func3 - func2 and func4 should be unchanged.

Notice how on the second compilation only func1 and func3 are typechecked.
