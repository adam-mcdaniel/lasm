//! # error, the module that provides basic structures for error handling
//!
//! Error handling is essential when writing an assembler or compiler.
//! Without proper error enums or descriptive error documentation, it can be very difficult
//! for compiler-writers to understand how to interface with error messages.
//!
//! Additionally, because this assembler also functions as a library, keeping error handling
//! elegant is crucial. As such, I've tried to make error handling as simple as possible.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::fmt;
use nom::error::{VerboseError, VerboseErrorKind};

/// The Result type is shorthand for a result that uses the Error
/// enum as the Err type. The Error type is just used as the general
/// crate-wide Error type.
pub type Result<T> = core::result::Result<T, Error>;

/// Get the first word in a string
fn first(s: impl core::fmt::Display) -> String {
    let s = s.to_string();
    let l = s
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    if !l.is_empty() {
        l[0].to_string()
    } else {
        String::from("")
    }
}

/// The Error type is used when assembling and parsing an assembly file  
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// When a procedure that doesn't exist is called, this error is returned
    ProcedureNotDefined(String),

    /// Using a register that has not been declared with
    /// the `define` keyword throws this error
    RegisterNotDefined(String),

    /// This is returned when an invalid argument to the `ld` instruction is supplied
    InvalidLoadArg(String),

    /// This is returned when an invalid argument to the `push` instruction is supplied
    InvalidPushArg(String),

    /// This is returned when an invalid argument to the `st` instruction is supplied
    InvalidStoreArg(String),

    /// This is returned when an invalid argument to the `refer` instruction is supplied
    InvalidReferArg(String),

    /// This is returned when an invalid argument to the `free` instruction is supplied
    InvalidFreeArg(String),

    /// This is returned when an invalid argument to the `alloc` instruction is supplied
    InvalidAllocArg(String),

    /// This is returned when an identifier is expected, but the identifier parser fails
    InvalidIdentifer(String),

    /// This is returned when a valid procedure is expected, but the procedure parser fails
    InvalidProcedure(String),

    /// This is returned when the `proc` keyword is not followed by a procedure name
    NoProcedureName(String),

    /// This is returned when an integer is expected but not found
    InvalidSize(String),

    /// This is returned when an unknown parser error is returned
    Unknown(String),

    /// This is returned when no procedures are found in the assembly file
    NoProcedureFound,

    /// This is returned when there are an odd number of `loop` and `endloop` keywords
    UnmatchedLoop,
}

impl Error {
    pub const REGISTER_NOT_DEFINED: &'static str = "register not defined";
    pub const INVALID_FREE_ARG: &'static str = "invalid argument supplied to free";
    pub const INVALID_LOAD_ARG: &'static str = "invalid argument supplied to ld";
    pub const INVALID_PUSH_ARG: &'static str = "invalid argument supplied to push";
    pub const INVALID_STORE_ARG: &'static str = "invalid argument supplied to st";
    pub const INVALID_ALLOC_ARG: &'static str = "invalid argument supplied to alloc";
    pub const INVALID_REFER_ARG: &'static str = "invalid argument supplied to refer";
    pub const INVALID_IDENTIFIER: &'static str = "not an identifier";
    pub const INVALID_PROCEDURE: &'static str = "invalid procedure";
    pub const INVALID_SIZE: &'static str = "invalid size value";
    pub const NO_PROC_NAME: &'static str = "procedure requires name";
    pub const NO_PROC_FOUND: &'static str = "no procedure found";
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> core::result::Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::ProcedureNotDefined(s) => format!("procedure not defined: '{}'", s),
                Self::RegisterNotDefined(s) => format!("{}: '{}'", Self::REGISTER_NOT_DEFINED, s),
                Self::InvalidLoadArg(s) => format!("{}: '{}'", Self::INVALID_LOAD_ARG, s),
                Self::InvalidPushArg(s) => format!("{}: '{}'", Self::INVALID_PUSH_ARG, s),
                Self::InvalidStoreArg(s) => format!("{}: '{}'", Self::INVALID_STORE_ARG, s),
                Self::InvalidReferArg(s) => format!("{}: '{}'", Self::INVALID_REFER_ARG, s),
                Self::InvalidFreeArg(s) => format!("{}: '{}'", Self::INVALID_FREE_ARG, s),
                Self::InvalidAllocArg(s) => format!("{}: '{}'", Self::INVALID_ALLOC_ARG, s),
                Self::InvalidIdentifer(s) => format!("{}: '{}'", Self::INVALID_IDENTIFIER, s),
                Self::InvalidProcedure(s) => format!("{}: '{}'", Self::INVALID_PROCEDURE, s),
                Self::NoProcedureName(s) => format!("{}: '{}'", Self::NO_PROC_NAME, s),
                Self::InvalidSize(s) => format!("{}: '{}'", Self::INVALID_SIZE, s),
                Self::Unknown(_) => "unknown error".to_string(),
                Self::NoProcedureFound => Self::NO_PROC_FOUND.to_string(),
                Self::UnmatchedLoop => "unmatched loop".to_string(),
            }
        )
    }
}

impl<'a> From<VerboseError<&'a str>> for Error {
    fn from(mut e: VerboseError<&'a str>) -> Self {
        let mut result = Self::Unknown(String::new());
        e.errors.reverse();
        for (input, err_kind) in &e.errors {
            let e = first(input);
            result = match err_kind {
                VerboseErrorKind::Context(s) => match *s {
                    Self::REGISTER_NOT_DEFINED => Self::RegisterNotDefined(e),
                    Self::INVALID_LOAD_ARG => Self::InvalidLoadArg(e),
                    Self::INVALID_FREE_ARG => Self::InvalidFreeArg(e),
                    Self::INVALID_ALLOC_ARG => Self::InvalidAllocArg(e),
                    Self::INVALID_PUSH_ARG => Self::InvalidPushArg(e),
                    Self::INVALID_STORE_ARG => Self::InvalidStoreArg(e),
                    Self::INVALID_REFER_ARG => Self::InvalidReferArg(e),
                    Self::INVALID_IDENTIFIER => Self::InvalidIdentifer(e),
                    Self::INVALID_PROCEDURE => Self::InvalidProcedure(e),
                    Self::INVALID_SIZE => Self::InvalidSize(e),
                    Self::NO_PROC_NAME => Self::NoProcedureName(e),
                    Self::NO_PROC_FOUND => Self::NoProcedureFound,
                    other => Self::Unknown(format!("{:?}", other)),
                },
                other => Self::Unknown(format!("{:?}", other)),
            };

            if let Self::Unknown(_) = result {
                continue;
            } else if let Self::NoProcedureFound = result {
                continue;
            } else if let Self::NoProcedureName(_) = result {
                continue;
            }
            break;
        }

        result
    }
}
