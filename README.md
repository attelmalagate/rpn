# rpn
### Simple Rust Reverse Polish Notation calculator
rpn is a rust library providing the functionalies of simple Reverse Polish Notation calculator.
Although conceived as an exercise to learn some of the fundamentals of the rust programming language, rpn does provide some interesting functionalities with a low memory footprint, limited CPU requirements and a respectable execution speed. It can be useful notably in the area of embedded devices or remote automation, to add variables elaborated from actual inputs or outputs without programmation, from expressions written in simple mathematical infix notation (i.e. operators are written in-between their operands).

### How does it work?
- Given an expression as main input (infix notation), the said expression is first lexicographically analyzed and tokenized in a vector of tokens
- This vector is then parsed into a rpn 'parse stack' (more or less [the Shunting-yard algorithm](https://en.wikipedia.org/wiki/Shunting-yard_algorithm)) which becomes available for evaluation.
- Evaluation algorithm, traversing the parse stack to produce a result:
  - Elements of the parse stack (tokens) are either numbers, constants, operators or functions
  - Operands (Numbers or Constants) contain a fixed value
  - Operators and Functions are described by their number of parameters (or operands for operators), and a reference to an evaluation function
  - Operators can be unary or binary; unary operators have 1 parameter, binary ones 2
  - Functions can have a fixed number of parameters (e.g. 1 for sinus, 2 for pow) or a variable number of parameters (e.g. max or average)
  - All elements of the parse stack are initialized as 'not consumed', and operators/functions 'not executed'
  - Start with the first operator or function not yet executed
  - From this element, search downwards in the stack for the first 'not consumed' operand(s) and evaluate the result with the eval function associated with the operator or function which will use its required number of operands/parameters; the get_operand function will consume the operands so that they are no longer available
  - The value returned by the eval function is associated with the operator/function token which is then no longer available for execution but becomes available as a normal operand to be consumed by the next operator or function

### Implementation
- Values associated with tokens are variant numbers (EVar), implemented in a separate module; EVar are represented as enum, and use operators overloading. 
For the sake of simplicity, 4 types of variant have been defined so far: String, i64, f64 and bool



