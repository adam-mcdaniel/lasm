use crate::{
    asm::{Instruct, Literal, Register, DEFAULT_STACK_SIZE},
    ast::{Ast, Exec, Procedure},
    Error, Result,
};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{anychar, char, space0},
    combinator::{cut, map_opt},
    error::{context, VerboseError},
    multi::{many0, many1},
    number::complete::double,
    IResult,
};

/// The ParseResult type is used to make interfacing with nom's IResult type simpler
pub type ParseResult<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

/// Parses a number literal as an unsigned size value
fn size(input: &str) -> ParseResult<usize> {
    let (input, num) = double(input)?;
    Ok((input, num as usize))
}

/// Parses a number literal as an actual instance of Literal.
/// This makes defining the `literal` parser much simpler
fn num(input: &str) -> ParseResult<Literal> {
    let (input, num) = double(input)?;
    Ok((input, Literal::num(num)))
}

/// Parses a character literal as an actual instance of Literal.
/// This makes defining the `literal` parser much simpler
fn ch(input: &str) -> ParseResult<Literal> {
    let (input, _) = char('\'')(input)?;
    let (input, ch) = alt((
        // This lambda function accounts for escape characters
        |input| {
            let (input, _) = tag("\\")(input)?;
            let (input, ch) = anychar(input)?;
            Ok((
                input,
                // convert the escaped character into its escaped form
                match ch {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '0' => '\0',
                    // if the character isnt recognized as an escape character,
                    // just pass it along as a non-escaped character
                    other => other,
                },
            ))
        },
        anychar,
    ))(input)?;
    let (input, _) = char('\'')(input)?;
    Ok((input, Literal::ch(ch)))
}

/// This parses either a character or number literal
fn literal(input: &str) -> ParseResult<Literal> {
    let (input, _) = space0(input)?;
    let (input, n) = alt((ch, num))(input)?;
    let (input, _) = space0(input)?;
    Ok((input, n))
}

/// This parses an identifier, which is composed of alphanumeric characters and underscores.
/// identifiers can start with numbers.
fn identifier(input: &str) -> ParseResult<&str> {
    let (input, _) = space0(input)?;
    let (input, i) = take_while1(|input: char| input.is_alphanumeric() || input == '_')(input)?;
    let (input, _) = space0(input)?;
    Ok((input, i))
}

/// register parses predefined registers and user defined registers using the Register
/// structure and the identifier parser.
fn register(input: &str) -> ParseResult<Register> {
    context(
        Error::REGISTER_NOT_DEFINED,
        cut(map_opt(identifier, |name| match name {
            "ACC" => Some(Register::Accumulator),
            "SPR" => Some(Register::StackPointer),
            other => Register::named(other),
        })),
    )(input)
}

/// This parser parses lasm's opcodes
fn opcode(input: &str) -> ParseResult<&str> {
    let (input, _) = space0(input)?;
    let (input, op) = alt((
        tag("refer"),
        tag("deref_ld"),
        tag("deref_st"),
        tag("free"),
        tag("alloc"),
        tag("ld"),
        tag("st"),
        tag("push"),
        tag("pop"),
        tag("dup"),
        tag("add"),
        // a second alt must be added because the number of arguments
        // is too large for a single call to alt
        alt((
            tag("inc"),
            tag("inn"),
            tag("sub"),
            tag("mul"),
            tag("div"),
            tag("outc"),
            tag("outn"),
            tag("cmp"),
            tag("loop"),
            tag("call"),
            tag("define"),
            tag("endloop"),
        )),
    ))(input)?;
    let (input, _) = space0(input)?;
    Ok((input, op))
}

fn instruction(input: &str) -> ParseResult<Exec> {
    let (input, op) = opcode(input)?;
    match op {
        "alloc" => {
            let (input, reg) = context(Error::INVALID_ALLOC_ARG, cut(register))(input)?;
            Ok((input, Exec::asm(Instruct::Alloc(reg))))
        }
        "free" => {
            let (input, reg) = context(Error::INVALID_FREE_ARG, cut(register))(input)?;
            Ok((input, Exec::asm(Instruct::Free(reg))))
        }
        "refer" => {
            let (input, reg) = context(Error::INVALID_REFER_ARG, cut(register))(input)?;
            Ok((input, Exec::asm(Instruct::Refer(reg))))
        }
        "deref_ld" => Ok((input, Exec::asm(Instruct::DerefLoad))),
        "deref_st" => Ok((input, Exec::asm(Instruct::DerefStore))),
        "ld" => {
            let (input, reg) = context(Error::INVALID_LOAD_ARG, cut(register))(input)?;
            Ok((input, Exec::asm(Instruct::Load(reg))))
        }
        "st" => {
            let (input, reg) = context(Error::INVALID_STORE_ARG, cut(register))(input)?;
            Ok((input, Exec::asm(Instruct::Store(reg))))
        }
        "push" => {
            let (input, lit) = context(Error::INVALID_PUSH_ARG, cut(literal))(input)?;
            Ok((input, Exec::asm(Instruct::Push(lit))))
        }
        "pop" => Ok((input, Exec::asm(Instruct::Pop))),
        "dup" => Ok((input, Exec::asm(Instruct::Duplicate))),
        "add" => Ok((input, Exec::asm(Instruct::Add))),
        "sub" => Ok((input, Exec::asm(Instruct::Subtract))),
        "mul" => Ok((input, Exec::asm(Instruct::Multiply))),
        "div" => Ok((input, Exec::asm(Instruct::Divide))),
        "outc" => Ok((input, Exec::asm(Instruct::OutputChar))),
        "outn" => Ok((input, Exec::asm(Instruct::OutputNumber))),
        "inc" => Ok((input, Exec::asm(Instruct::InputChar))),
        "inn" => Ok((input, Exec::asm(Instruct::InputNumber))),
        "cmp" => Ok((input, Exec::asm(Instruct::Compare))),
        "loop" => Ok((input, Exec::asm(Instruct::WhileNotZero))),
        "endloop" => Ok((input, Exec::asm(Instruct::EndWhile))),
        "call" => {
            let (input, i) = cut(identifier)(input)?;
            Ok((input, Exec::call(i)))
        }
        "define" => {
            let (input, i) = context(Error::INVALID_IDENTIFIER, cut(identifier))(input)?;
            let (input, _) = char(',')(input)?;
            let (input, _) = space0(input)?;
            let (input, s) = context(Error::INVALID_SIZE, cut(size))(input)?;
            let (input, _) = space0(input)?;
            Register::define(i, s);
            Ok((input, Exec::Nop))
        }
        _ => unreachable!(),
    }
}

fn procedure(input: &str) -> ParseResult<Procedure> {
    let (input, _) = space0(input)?;
    let (input, _) = context(Error::INVALID_PROCEDURE, tag("proc"))(input)?;
    let (input, name) = context(Error::NO_PROC_NAME, identifier)(input)?;
    let (input, code) = cut(many0(instruction))(input)?;
    let (input, _) = context(Error::INVALID_PROCEDURE, tag("endproc"))(input)?;
    let (input, _) = space0(input)?;
    Ok((input, Procedure::new(name, code)))
}

pub fn program(mut input: &str) -> Result<(Ast, usize)> {
    let stack_size;
    match (|input| -> ParseResult<usize> {
        let (input, _) = space0(input)?;
        let (input, _) = tag("stack_size")(input)?;
        let (input, _) = space0(input)?;
        let (input, n) = size(input)?;
        let (input, _) = space0(input)?;

        Ok((input, n))
    })(input)
    {
        Ok((i, s)) => {
            input = i;
            stack_size = s;
        }
        Err(_) => stack_size = DEFAULT_STACK_SIZE,
    }

    let res = context(Error::NO_PROC_FOUND, many1(procedure))(input);

    match res {
        Ok((_, procs)) => Ok((Ast::new(procs), stack_size)),
        Err(e) => match e {
            nom::Err::Error(e) => Err(Error::from(e)),
            nom::Err::Failure(f) => Err(Error::from(f)),
            e => Err(Error::Unknown(format!("{:?}", e))),
        },
    }
}
