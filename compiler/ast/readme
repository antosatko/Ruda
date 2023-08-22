AST stands for abstract syntax tree
===================================

It is Ruda compiler specific structure, you wont find it anywhere else.

This file can be considered its documentation.

Syntax errors are non-existent at the moment so think twice before writing code. (you might find yourself in an infinite loop :P)

Indentation is tabs only (\\t), comments are not allowed, each structure must follow a set of rules:

1.  Head - defines appearence of your structure
    *   identifier - structures are accesible by their identifiers
    *   parameters - params represent data returned by the structure
2.  Body - defines internal structure of elements
    *   modifiers:
        *   expected element - structure throws an exception if the element is incorrect. Exceptions can be cought by optional elements, compiler reports syntax error if not.
        *   optional elements (?) - optionals can nest another elements, these are accesed if the parent element is true
        *   commands (!) - commands are executable scripts embedded in compiler. They help to write cleaner, more intelligent code.
        *   optional arguments (=) - runs nested code if passed argument is equal
    *   element types:
        *   token (,) - tokens are usually special characters
        *   text ("text") - a literal value
        *   type ("'string") - types represent only type with any value
        *   structure (type) - executes code for specified structure
    *   arguments: small commands for individual elements
        *   type - "=" - value (set="params")
3.  End - ends structure definition. Semicolon at the end of last element (last element, NOT the end of branch, always only one end).

Compiler specific Element arguments:

*   end ('any) - structure returns its current state
*   set (<target>) - head parameter is set/pushed elements return value
*   back (unsigned integer) - code cursor is sent back
*   print (text) - prints to the terminal
*   err (message) - returns error message if node does not fail
*   peek ('any) - token idx wont move forward
*   harderr (bool) - true: any error occuring from now on carries to the prev structure
*   advance (usize) - advances token idx
*   debug ('any) - prints current token
*   data ('identifier) - prints own data

To create own arguments just pass them to the structure.
