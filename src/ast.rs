use crate::{asm, Error, Result};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::fmt::Display;

/// The Ast object stores the entire abstract syntax tree
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Ast {
    procs: Vec<Procedure>,
}

impl Ast {
    /// Create an Ast object from a list of procedures
    pub(crate) fn new(procs: Vec<Procedure>) -> Self {
        Self { procs }
    }

    /// Lower the list of procedures into a list of instructions.
    /// Procedures are not stored in memory, but are inlined instead.
    /// All this method does is resolve all `call` instructions.
    pub fn lower(self) -> Result<Vec<asm::Instruct>> {
        let mut entry_proc = Procedure::new(Procedure::ENTRY_POINT, Vec::new());

        for prc in &self.procs {
            if prc.is_entry_point() {
                entry_proc = prc.clone();
            }
        }

        entry_proc.lower(&self.procs)
    }
}

/// This represents an instruction in an assembly file
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub(crate) enum Exec {
    /// This is not used by the user, but is used internally for simplicity
    Nop,

    /// A procedure call
    Call(String),

    /// An assembly instruction
    Assembly(asm::Instruct),
}

impl Exec {
    /// Create a call instruction
    pub fn call(name: impl Display) -> Self {
        Self::Call(name.to_string())
    }

    /// Create an assembly instruction
    pub fn asm(i: asm::Instruct) -> Self {
        Self::Assembly(i)
    }
}

/// The Procedure struct is only
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub(crate) struct Procedure {
    /// The name of the procedure
    name: String,

    /// The list of instructions the procedure will execute when called
    code: Vec<Exec>,
}

impl Procedure {
    /// The name of the entry point function
    const ENTRY_POINT: &'static str = "start";

    pub fn new(name: impl Display, code: Vec<Exec>) -> Self {
        Self {
            name: name.to_string(),
            code,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn is_entry_point(&self) -> bool {
        self.get_name() == Self::ENTRY_POINT
    }

    pub fn is_proc(&self, name: impl Display) -> bool {
        self.get_name() == name.to_string()
    }

    pub fn lower(&self, procs: &[Self]) -> Result<Vec<asm::Instruct>> {
        let mut result = Vec::new();
        for expr in &self.code {
            match expr {
                Exec::Nop => {}
                Exec::Call(name) => {
                    let mut proc_exists = false;
                    for prc in procs {
                        if prc.is_proc(&name) {
                            proc_exists = true;
                            result.extend(prc.lower(procs)?);
                            break;
                        }
                    }

                    if !proc_exists {
                        return Err(Error::ProcedureNotDefined(name.clone()));
                    }
                }
                Exec::Assembly(i) => result.push(i.clone()),
            }
        }
        Ok(result)
    }
}
