This is my attempt at a SysY → RISC-V compiler,
following the [PKU Compiler Course](https://pku-minic.github.io/online-doc/).
Read [the write-up on my blog](https://satgo1546.github.io/archives/pku-minic/).

I have taken various shortcuts in the implementation,
that the compiler is *just compliant enough* to pass the tests.
A non-exhaustive list of corners I have cut follows:

- No error handling is ever done.
  Errors either result in a panic or silent miscompilation.
- No type checking at all.
- All variables — even globals — are placed on the stack.
  It may be necessary to raise `ulimit -s` to pass some of the tests.
  - Global zero-initialized arrays are filled upon program start in tight store loops.
- `const` keyword has no effect on arrays.
- Uninitialized constants are treated as variables.
- Variable definitions can shadow other variables in the same block.
- Functions have an upper limit of 520 parameters.
  Go beyond the limit and it emits invalid assembly.
- Functions always return `int`.
  Even `void` functions return 0.
- `[]` and `[0]` (and even with expressions that evaluate to 0 such as `[1-1]`) are synonyms in array type definitions.

On the other hand, it supports these extra language features:

- binary integer literal `0b11011111101010010`
- bitwise not `~` (but not other bitwise operators)
- optional trailing comma in parameter lists and initializer lists
- strict left-to-right evaluation order
- unordered global variable and function definitions
  - There is neither need nor support for forward declarations.
  - Constants evaluated at compile time are still required to be in order.
- external function declaration (per [awesome-sysy/mandelbrot](https://github.com/pku-minic/awesome-sysy/blob/master/mandelbrot))

Code in this branch is licensed under GPL-3.0,
because the `koopa` crate is licensed under GPL-3.0.
