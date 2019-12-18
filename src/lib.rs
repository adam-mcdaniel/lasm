//! # lasm, a minimal and portable assembly langauge
//!
//! The spirit of this crate is to make the most small and correct
//! assembly language as possible. A reduced instruction set is valued
//! above all else. If possible, speed is also an admirable trait.
//!
//! # purpose
//!
//! Writing a compiler is very **_very_** hard. A lot of that
//! difficulty comes from trying to manage memory and trying to represent
//! high level concepts in terms of low level instructions.
//!
//! So, with these problems in mind, I wrote this assembly language.
//!
//! # features
//!
//! The most high level feature is the infinite number of registers. This allows
//! the compiler to declare and use variables **significantly** easier.
//! The last time I wrote a compiler, the absolute hardest part was managing
//! when variables were allocated and freed. As a result, I wrote this assembly
//! language to take care of that!
//!
//! ### procedures
//!
//! Another high level feature is managing procedure declarations. When the
//! assembly is parsed, the procedures are each defined before they are checked
//! for semantic errors. So, procedures can be defined in any order.
//!
//! ### portability
//!
//! The final, and best feature is portability. lasm is _extremely_ compact:
//! the entire C implementation of lasm's instruction set is nearly 150 lines.
//! Writing an implementation for lasm is extremely simple, and so compiling to
//! lasm allows the compiler to target several different programming languages
//! and platforms.
//!
//! # basic instructions
//!
//! | Stack Instruction | Description |
//! |-------------------|-------------|
//! | `push LITERAL` | Push the LITERAL argument onto the stack. The LITERAL argument MUST be a character or float |
//! | `pop` | Pop a value off of the stack and into the ACC register |
//! | `ld REGISTER` | Push the value stored in REGISTER onto the stack. The REGISTER being loaded MUST be defined before being loaded |
//! | `st REGISTER` | Pop a value off of the stack into REGISTER. The REGISTER being stored to MUST be declared before being stored |
//! | `dup` | Duplicate the top item on the stack |
//!
//! | Pointer Instruction | Description |
//! |---------------------|-------------|
//! | `refer REGISTER` | Push a pointer to REGISTER onto the stack |
//! | `deref_ld` | Pop a pointer off of the stack, and push the value stored at where the pointer points. This will only push a single cell onto the stack, not more than one cell |
//! | `deref_st` | Pop a pointer and a cell off of the stack, and store the cell at the pointer |
//! | `alloc REGISTER` | Pop a SIZE value off of the stack, and store the address to SIZE free cells in REGISTER |
//! | `free REGISTER` | Pop a SIZE value off of the stack, and free the memory stored at the pointer stored in REGISTER |
//!
//!
//! | Math Instruction | Description |
//! |------------------|-------------|
//! | `add` | Pop two cells off of the stack, and push their sum |
//! | `sub` | Pop two cells off of the stack, and push the first minus the second |
//! | `div` | Pop two cells off of the stack, and push their product |
//! | `mul` | Pop two cells off of the stack, and push the first divided by the second |
//! | `cmp` | Pop two cells off of the stack, and push -1 if the first is less than the second, 0 if they are equal, and 1 otherwise |
//!
//! | IO Instruction | Description |
//! |----------------|-------------|
//! | `outc` | Pop a cell off of the stack and print it as a character |
//! | `outn` | Pop a cell off of the stack and print it as a float |
//! | `inc` | Get a character from STDIN and push it into the stack |
//! | `inn` | Get a float from STDIN and push it into the stack |
//!
//! | Control Instruction | Description |
//! |---------------------|-------------|
//! | `loop` | Marks the start of a loop. At the start of each iteration, a test value is popped from the stack. While the value is not zero, the loop continues. Else, the loop jumps to the matching `endloop` |
//! | `endloop` | Marks the end of a loop |
//!
//! # examples
//!
//! This assembly language is a bit simpler than most others because
//! portability and compactness are the two largest goals in mind.
//! As a result, examples are pretty simple.
//!
//! ## fibonacci
//! 
//! This simply implements fibonacci by doing arithmetic on three variables `a`, `b`, and `c`.
//! To simplify outputing the numbers, a few helper procedures are defined.
//! 
//! ```rust,ignore,no_run
//! // comments are C-style
//! // The `stack_size` flag can ONLY be used at the top of the file.
//! // Anywhere else, this flag will show up as a syntax error.
//! // The purpose of the flag is to set the size of memory used
//! // outside of the statically determined memory. Any loads,
//! // pushes, allocs, etc. require a bit of memory on the stack.
//!
//! // If this flag is not present, 256 cells are used by default.
//! stack_size 1024
//!
//! // The start procedure is the entry point
//! proc start
//!     // Declare the registers we will use
//!     define a, 1
//!     // Push 0 and store it in 'a'
//!     push 0 st a
//!     define b, 1
//!     // Push 1 and store it in 'b'
//!     push 1 st b
//!     define c, 1
//!     // Push 0 and store it in 'c'
//!     push 0 st c
//!
//!     // This will determine the number of times to iterate
//!     define n, 1
//!     // Push 10 and store it in 'n'
//!     push 10 st n
//!
//!     // loop while n is not zero
//!     ld n
//!     loop
//!         ld a st c // c = a
//!         ld b st a // a = b
//!         ld a call print_num // print a
//!         ld c ld b add st b // b = c + b
//!
//!         push 1
//!         ld n
//!         // subtract 1 from n
//!         sub
//!         // store the result in n again
//!         st n
//!
//!         // Load n again for the loop test
//!         ld n
//!     endloop
//! endproc
//!
//!
//! proc print_num
//!     // the define keyword takes two arguments,
//!     // the name of the register and the size of
//!     // the newly created register.
//!
//!     // This simply tells the assembler to allocate permanent
//!     // space for a register with a given size. It also tells
//!     // the assembler how many cells to pop off of the stack when
//!     // storing a value in this register.
//!     define n, 1
//!     
//!     // When we call print_num, we expect a single argument on the
//!     // stack. So, we store this argument in the register n for later
//!     // usage.
//!     st n
//!
//!     // Now we load the value stored in n back onto the stack
//!     // and print the value as a number
//!     ld n outn
//!
//!     // Now we print a newline using the newline procedure
//!     call nl
//! endproc
//!
//!
//! proc nl
//!     // 10 (the character code for '\n') is pushed onto the stack
//!     // and printed out as a character
//!     push 10 outc
//! endproc
//! ```
//!
//! # implementation
//! 
//! lasm's implementation is very simple: there are very few instructions to implement
//! when targeting a new programming language. Additionally, lasm's structure is very
//! simple to implement in low level languages.
//! 
//! There are a few **very** important notes for lasm's implementation
//! 1. lasm's memory is implemented using an array of double precision floats, or 64 bit floats
//! 2. lasm tracks allocs and frees for each individual cell of the memory array. This is most
//! simply done using an array of booleans with identical length to the data tape
//! 3. allocating more than the available amount of memory is undefined behavior (if possible, this should cause the program to exit)
//! 4. the implementation should _always_ mark memory reserved for registers as allocated (so that alloc may not return a pointer to register memory)
//! 5. memory reserved for registers always lies **immediately** before the stack
//! 6. the accumulator register always lies at address `0`
//! 7. the stack pointer register always lies at address `1`
//! 8. user defined registers lie between the stack pointer register and the stack
//! 9. the `inn` and `inc` instructions return `0` on `EOF` and on other input errors

#![no_std]
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate lazy_static;

pub mod asm;
pub use asm::{Instruct, Register};
pub(crate) mod ast;
pub mod target;
pub use target::Target;
pub mod error;
pub use error::{Error, Result};
pub(crate) mod parser;
pub(crate) use parser::program;

use alloc::{string::String, vec::Vec, collections::BTreeMap};

/// assemble takes an assembly target, and the assembly code an object that implements display
/// 
/// This function will 
/// 1. parse the assembly code and convert it into an abstract syntax tree
/// 2. convert the abstract syntax tree into a list of executable assembly instructions
/// 3. transform the list of assembly instructions into output code using the assembly target
pub fn assemble(target: impl Target, asm_code: impl core::fmt::Display) -> Result<String> {
    use asm::{REGISTER_POINTER, NAMED_REGISTERS, PREDEFINED_REGISTERS};

    let (code, stack_size) = compile(asm_code)?;
    let initial_stack_ptr = *REGISTER_POINTER.lock();

    // Reset the global variables used for assembling an input file
    *REGISTER_POINTER.lock() = PREDEFINED_REGISTERS;
    *NAMED_REGISTERS.lock() = BTreeMap::new();
    
    // Assemble using the targets assembly method
    Ok(target.assemble(initial_stack_ptr, stack_size, code))
}


fn compile(s: impl core::fmt::Display) -> Result<(Vec<Instruct>, usize)> {
    let (ast, stack_size) = program(
        &comment::c::strip(s)
            .unwrap()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" "),
    )?;

    let code = ast.lower()?;

    let mut loop_count = 0;
    for instruct in &code {
        match instruct {
            Instruct::WhileNotZero => loop_count += 1,
            Instruct::EndWhile => loop_count -= 1,
            _ => {}
        }
    }

    if loop_count != 0 {
        Err(Error::UnmatchedLoop)
    } else {
        Ok((code, stack_size))
    }
}
