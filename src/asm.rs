//! # asm, the module that provides structures that represent assembly instructions
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
};

use core::fmt;
use spin::Mutex;

/// This is the stack size that is used if the assembly file
/// does not specify one.
pub const DEFAULT_STACK_SIZE: usize = 256;

/// This specifies the number of predefined registers.
/// This is VERY important to get right, if this is too small,
/// user defined registers will overwrite the Accumulator,
/// StackPointer, and other predefined registers.
pub const PREDEFINED_REGISTERS: usize = 2;

lazy_static! {
    /// This tracks the current address where the next register will be allocated
    pub(crate) static ref REGISTER_POINTER: Mutex<usize> = Mutex::new(PREDEFINED_REGISTERS);

    /// This tracks the named registers
    pub(crate) static ref NAMED_REGISTERS: Mutex<BTreeMap<String, Register>> = Mutex::new(BTreeMap::new());
}

/// The Register enum represents a register in an assembly program (obviously)
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Register {
    /// The register that points to where the next item on the stack
    /// will be stored to
    StackPointer,

    /// The register that stores temporary values in several instructions
    Accumulator,

    /// A register defined by the user. Each register's address and size is
    /// known statically.
    Named {
        /// the name of the user defined register
        name: String,
        /// the number of cells that the register occupies
        size: usize,
        /// the address of the register
        addr: usize,
    },
}

impl Register {
    /// Get a user defined register by its name.
    /// The register MUST be previously defined.
    pub fn named(name: impl fmt::Display) -> Option<Self> {
        let registers = NAMED_REGISTERS.lock();
        if let Some(r) = registers.get(&name.to_string()) {
            Some(r.clone())
        } else {
            None
        }
    }

    /// Define a Register with a given name and size. This will
    /// create a Register in the NAMED_REGISTERS map with the
    /// value of REGISTER_POINTER as the Register's address.
    pub fn define(name: impl fmt::Display, size: usize) -> Self {
        // The pointer to where this register will be stored
        let mut rptr = REGISTER_POINTER.lock();
        // The map of defined registers
        let mut registers = NAMED_REGISTERS.lock();

        // This register
        let result = Self::Named {
            name: name.to_string(),
            size,
            addr: *rptr,
        };

        // Increment the register pointer so that the next
        // defined register will not overwrite this register
        *rptr += size;

        // Insert this register into the map
        registers.insert(name.to_string(), result.clone());

        // Return this register
        result
    }

    /// Get the address where this register is stored. This is
    /// used in many instructions, most notably the `refer` instruction.
    pub fn get_addr(&self) -> usize {
        match self {
            Self::Named { addr, .. } => *addr,
            Self::Accumulator => 0,
            Self::StackPointer => 1
        }
    }

    /// Get the number of cells this register occupies. Both the
    /// StackPointer and the Accumulator are one cell each
    pub fn get_size(&self) -> usize {
        match self {
            Self::Named { size, .. } => *size,
            _ => 1,
        }
    }
}

/// The Literal enum represents a literal in an assembly program (duh).
/// A literal can either be a double precision float, or a character literal
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Literal {
    /// A character literal
    Character(char),

    /// A double precision float literal
    Number(f64),
}

impl Literal {
    /// Create a character literal from a character
    pub fn ch(ch: char) -> Self {
        Self::Character(ch)
    }

    /// Create a number literal from a double precision float
    pub fn num(n: f64) -> Self {
        Self::Number(n)
    }

    /// Get the value of this literal as a double precision float
    pub fn get(&self) -> f64 {
        match self {
            // a character cannot directly cast as a float
            Self::Character(ch) => *ch as i32 as f64,
            Self::Number(n) => *n,
        }
    }
}

/// This enum represents a single instruction that can be assembled.
/// The majority of the work to implement Target for a target
/// programming language is defining how to convert an instruction
/// to a properly formatted string for the target.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Instruct {
    /// The `refer` instruction takes a register and pushes a
    /// pointer to that register onto the stack. The motivation
    /// for only allowing registers to be referenced is very
    /// simple to explain. Values on the stack are meant to be
    /// temporary. Literals pretty much exclusively exist on the
    /// stack. Allowing references to literals promotes bad practices.
    Refer(Register),

    /// The `deref_ld` instruction takes no arguments. This instruction
    /// pops an address off the stack and pushes the value stored at the
    /// cell the address points to.
    ///
    /// Essentially, this derefences the pointer as a double precision float pointer.
    DerefLoad,

    /// The `deref_st` instruction takes no arguments. This instruction
    /// pops an address cell and a value cell off the stack. The value cell is stored
    /// as a double precision float at the location the pointer points to.
    ///
    /// This is equivalent to `*(double*)address = value`
    DerefStore,

    /// The `ld` instruction takes a register as an argument. This instruction
    /// loads the value stored in the register and pushes it onto the stack.
    Load(Register),

    /// The `st` instruction takes a register as an argument. This instruction
    /// pops a value off of the stack and stores it in the register.
    ///
    /// This is equivalent to `register = value`
    Store(Register),

    /// The `push` instruction takes either a literal as an argument. This literal
    /// can either be a character or a double precision float. This instruction pushes
    /// the literal onto the stack.
    Push(Literal),

    /// The `pop` instruction pops a value off of the stack and stores it in the ACC register.
    ///
    /// This is equivalent to `ACC = value`
    Pop,

    /// The `alloc` instruction takes a register as an argument, and pops a `size` value
    /// off of the stack. Then, `alloc` stores a pointer `size` number of consecutive free cells.
    ///
    /// This is equivalent to `register = (double*)malloc(size)`
    Alloc(Register),

    /// The `free` instruction takes a register as an argument, and pops a `size` value
    /// off of the stack. Then, `free` frees `size` number of values at the location the
    /// register points to.
    Free(Register),

    /// The `dup` instruction takes no argument, and simply duplicates the top item on
    /// the stack.
    Duplicate,

    /// The `add` instruction pops two cells off the stack and pushes their sum
    Add,

    /// The `sub` instruction pops two cells off the stack and pushes the first minus the second
    Subtract,

    /// The `mul` instruction pops two cells off the stack and pushes their product
    Multiply,

    /// The `div` instruction pops two cells off the stack and pushes the first divided by the second
    Divide,

    /// The `outc` instruction pops a cell off the stack and prints `cell % 256` as a character to STDOUT
    OutputChar,

    /// The `outn` instruction pops a cell off the stack and prints the cell as a number to STDOUT
    OutputNumber,

    /// The `inc` instruction pushes a character from STDIN onto the stack
    InputChar,

    /// The `inn` instruction pushes a double precision float from STDIN onto the stack
    InputNumber,

    /// The `cmp` instruction compares two cells on the stack.
    /// If the first popped value is less than the second, -1 is pushed.
    /// If the first popped value is equal to the second, 0 is pushed.
    /// If the first popped value is greater than to the second, 1 is pushed.
    Compare,

    /// The `loop` instruction the start of a loop. At the start of each iteration, a test value is popped from the stack. While the value is not zero, the loop continues. Else, the loop jumps to the matching `endloop`
    WhileNotZero,

    /// The `endloop` instruction marks the end of a loop
    EndWhile,
}
