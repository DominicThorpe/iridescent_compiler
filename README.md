# The Iridescent Programming Language
*Iridescent* - *Showing luminous colours that seem to change when seen from different angles.*

The iridescent programming language is a personal project designed to be run on the Iridium Computer Emulator. It is designed to be simple, yet powerful, with features such as strong, static typing, and immutable-by-default variables. Most of all, however, it is designed to be easy to write a compiler for, so that a computer scientist with even just a basic understanding of compiler principles can write one as a beginner's project.


## Syntax

### Functions
Functions are declared with the `fn` keyword, followed by a return type (can be `void`), an identifier, and a list of parameters in parentheses. The body of the function is contained in curly brackets, and is composed of 0 or more statements. A function with a return type must contain a return statement.

```
fn <type> <identifier> (<parameter>*) {
    <statement>*
}

fn int addition_minus_one(int a, int b) {
    let int added = ((a, b)+, 1)-;
    return added;
}
```


### Expressions

When making an expression for a statement, such as declaring a variable, the operations are done differently than in most programming languages, in that the mathematical operations are *postfix* instead of the usual *prefix* or *infix*. Therefore instead of `(7 + 7) / 2`, we do `((7, 7)+, 2)/ `, so the arguments are in brackets, and the operator goes after the brackets. If we want to get the value -5, we do numerical negation on 5, which is `(5)-` instead of `-5`.

The reason for doing this is so that expressions are extremely easy to parse as associativity and operator precedence is not an issue when generating the abstract syntax tree (AST), which becomes complex for expressions such as 5 + 7 + 7 * 3 / (5 + 2), etc...


### Variable Declaration and Assignment

Variables are declared using the `let` keyword followed by a type, an optional mutability modifier (`mut` for mutable, `const` for immuatble) which is immutable by default, and then an identifier. The value is set after the `=` sign, and is terminated with a semicolon. An initial value must always be given.

`let <type> <mutability>? <identifer> = <value | expression>;`

```
let int x = 5;
let mut int y = (5, 5)+;
let const z = (0xFA8, x)>>;
```

A mutable variable can be reassigned by giving the variable name, then an `=`, and then the new value, terminated with a semicolon.

```
let mut int x = 7;
x = 9;
```

We can also cast variables of one type to another with the syntax `<type>(<value>)` where value is set to the type given, so we can do:
```
fn long main() {
  let int x = 5;
  let long y = long(x);

  return y;
}
```


### If, Else if, Else Statements

If-Else-If-Else (IEIE) is used to branch when a given condition is true. They are in the form:
```
if <condition> {
    statements
}

else if <condition> {
    statements
}

else {
    statements
}
```

There can be any number of else if blocks in the IEIE structure, and at most 1 else block, which is the only one not to have a condition, and it must come at the end. The condiitons are boolean expressions, such as `((x, y)>, (y, (z)!)==)&&`. Note that boolean expressions can only be used in conditions and not in assignment (use ternary statements for that) and boolean connectives can only be used outside of boolean terms, so `((x, y)==, (y,z)>)&&` is valid but `((x,y)&&, 3)>` is not. Boolean NOT `!` can be used on both boolean expressions and terms.

Currently supported boolean operations are: `!`, `>`, `>=`, `<`, `<=`, `==`, `!=`.

Currently supported boolean connectors are: `&&`, `||`, `^^` (XOR).


### Indefinite Loops

These loops repeat until they hit a break statement and require a break keyword to be semantically valid. The syntax for them is as follows:
```
loop {
  <statements>*
}
```


### While Loops

These loops repeat until the condition they are given becomes false (note that they can never run if the condition starts false). The syntax for them is as follows:
```
while <condition> {
  <statements>*
}
```


### For Loops 

These loops repeat a fixed number of times, but can still contain `break` and `continue` statements. They are in the following format:

```
for <type> <identifier> = <expression> until <expression> step <expression> {
  <statements>*
}
```

The type specified must be either `int` or `long`, and the step part is optional, and defaults to 1. For example, `for long i = 0 until (50, 1000)* step (1,1)+` and `for int i = 0 until 10` are valid, but `for float i = 0.0 until 10.0 step 0.1` is not.


## Current and Planned Features

### Functions

Functions must be declared in global scope (i.e. cannot be declared within each other) and are required to return the correct type (can be `void`). They cannot be passed as arguments to functions and are not 1st class.

Currently, parameters are not supported, however, they will function similar to C, with arbitrary numbers of arguments. Functions will be callable within expressions.


### Datatypes

Currently, only the `int` (signed 16 bit) datatype is available, and the `void` datatype is available as a return type. 

In future I plan to add support for `float` (signed 32 bit) numbers, `long` (signed 32 bit) and `byte` (unsigned 8 bit) integers, as well as `char` and `string` datatypes, which will all be primitives. There will also be structs, arrays, and pointers built into the language.


### Control Structures

Currently, the language supports the following flow control structures:
  - functions
  - if, else-if, else (IEIE) structures,
  - indefinite loops
  - while loops,

The following are also planned:
  - for loops, 
  - ternary structures

These shall function much the same as their equivalents in C.


### Statements

Statements are the basic non-flow-control actions the program can take. Currently supported are:
  - function return
  - variable declaration (mutable and immutable)
  - variable assignment

In the future, the following shall also be supported:
  - print
  - variable casting (may be moved to an expression)


### Expressions

Expressions are chains of actions or simple identifiers or values which statements are performed on. For example, doing ((3, 4)+, 2)- is an expression. Mostly expressions involve doing mathematics (currently implemented), or calling functions (planned), or both. The following are currently supported expression operations:
  - function calls

  - binary (takes 2 arguments):
    - addition (+)
    - subtraction (-)
    - multiplication (*)
    - division (/)
    - logical AND (&)
    - logical OR (|)
    - logical XOR (^)
    - right logical shift (>>)
    - left logical shift (<<)
    - right arithmetic shift (>>>)

  - unary (takes 1 argument):
    - complement (~)
    - numerical negation (-)
    - logical negation (!)


## How it Works

The functioning of the compiler can be split into the following stages:
  - *Lexical Analysis* - this is handled by the *Pest* external library and splits the program text into a tree of tokens which can then be used to create the AST. This phase detects some invalid syntax, such as missing keywords or invalid identifiers.
  - *Syntactic Analysis* - takes the output of lexical analysis and transforms it into an AST. Detects problems such as invalid literals or expressions.
  - *Semantic Analysis* - checks the AST for problems taking the context of the whole program into account. Finds problems such as scoping errors, undeclared identifiers, and bad return values.
  - *Intermediate Code Generation* - takes the AST and transforms it into a simple, stack-based language which makes target-code generation easier. It can be used to more easily create code for any target instruction set architecture.
  - *Optimisation* - Takes intermediate code and makes any optimisations it can find, such as removing extraneous load and store instructions and constant folding (**stage not implemented**).
  - *Target Code Generation* - Takes intermediate code and converts it into the final target code.


## State of Development

Currently working on adding type casting to the language up to the intermediate code generation phase.
