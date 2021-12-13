# rpn
### Simple Rust Reverse Polish Notation calculator library
rpn is a rust library providing a simple Reverse Polish Notation calculator library
Although conceived as an exercise to learn some of rust fundamentals through a real life exercise, it does provide however some interesting functionalities with a low memory footprint and CPU requirements. It can be useful notably in the area of embedded devices or remote automation, to add simple but powerful virtual variables from actual inputs or outputs.

### How does it work?
- Given an expression as main input, the said expression is first lexicographically analyzed and tokenized in a vector of tokens
- This vector is then parsed into a rpn 'parse stack' (more or less [the Shunting-yard algorithm](https://en.wikipedia.org/wiki/Shunting-yard_algorithm))
- Algorithm to evaluate the parse stack and produce a result:
  - Elements of the parse stack (tokens) are either numbers, constants, operators or functions
  - Tokens representing numbers or constants contain a value
  - Tokens representing operators and functions include a number of parameters and a pointer to an evaluation function
  - Operators can be unary or binary; unary operators have 1 parameter, binary ones 2
  - Functions can have a fixed number of parameters (e.g. 1 for sinus, 2 for power) or a variable number of parameters (e.g. max or average)
  - All elements of the parse stack are initialized as 'not consumed', and operators/functions 'not executed'
  - Start with the first operator or function in the stack not yet executed
  - From this element, search downwards in the stack for the first 'not consumed' operands and evaluate the result with the eval function associated with the operator or function which will use its required number of operands/parameters; the get_operand function will consume the operands so that they are no longer available
  - The value returned by the eval function is associated with the operator/function token which is then no longer available for execution but becomes available as a normal operand to be consumed by the next operator or function

### Implementation
- Values associated with tokens are variant numbers (EVar) in a separate module; EVar are implemented as enum, and use operators overloading



