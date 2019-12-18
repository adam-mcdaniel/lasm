//! # target, the module responsible for providing common targets for lasm
//!
//! As stated in the top level documentation of the crate,
//! the purpose of lasm is to be as portable as possible.
//! To maximize portability, the `target` module provides the
//! `Target` trait and a few builtin implementations for 
//! common programming languages.
//! 
//! Using the Instruct enum in the `asm` module, though,
//! Target can be implemented for other programming languages
//! by other crates that use this library. Additionally,
//! uses can write more optimized implementations for languages
//! that already have one.

use crate::Instruct;
use alloc::{string::String, vec::Vec};

/// This trait should be implemented for a struct that represents
/// a target language that lasm assembles to.
pub trait Target {
    /// This function assembles a list of instructions with a given stack size and initial stack pointer.
    ///
    /// The reason the initial stack pointer is necessary is because the output code must know how large
    /// the memory allocated for registers is. Without the initial stack pointer, the output code would
    /// have no clue how much memory registers use.
    fn assemble(&self, initial_stack_ptr: usize, stack_size: usize, code: Vec<Instruct>) -> String;
}

/// C is a target
pub struct C;

impl Target for C {
    fn assemble(&self, initial_stack_ptr: usize, stack_size: usize, code: Vec<Instruct>) -> String {
        let total_mem_size = initial_stack_ptr + stack_size;

        let mut result = format!(
            "#include <stdio.h>
#include <stdbool.h>

const int INIT_STACK_PTR = {reg_size};
const int MEMORY_SIZE = {mem_size};",
            reg_size = initial_stack_ptr,
            mem_size = total_mem_size
        );

        result += r#"
const int ACC = 0;
const int SPR = 1;


void init(double tape[], bool alloc_tape[]) {
    tape[SPR] = INIT_STACK_PTR;
    for (int i=0; i<INIT_STACK_PTR; i++) {
        alloc_tape[i] = true;
    }
}

void push_cell(double tape[], double value) {
    tape[(int)tape[SPR]++] = value;
}

void pop_cell(double tape[], int addr) {
    tape[addr] = tape[(int)--tape[SPR]];
    tape[(int)tape[SPR]] = 0;
}


void deref_load(double tape[]) {
    pop_cell(tape, ACC);
    int addr = tape[ACC];
    push_cell(tape, (int)tape[addr]);
}

void deref_store(double tape[]) {
    pop_cell(tape, ACC);
    int addr = tape[ACC];
    pop_cell(tape, addr);
}


void store(double tape[], int addr, int size) {
    for (int i=0; i<size; i++) {
        pop_cell(tape, addr + size - i - 1);
    }
}

void load(double tape[], int addr, int size) {
    for (int i=0; i<size; i++) {
        push_cell(tape, tape[(int)(addr + i)]);
    }
}


void add(double tape[]) {
    pop_cell(tape, ACC);
    double a = tape[ACC];
    pop_cell(tape, ACC);
    double b = tape[ACC];
    push_cell(tape, a + b);
}

void sub(double tape[]) {
    pop_cell(tape, ACC);
    double a = tape[ACC];
    pop_cell(tape, ACC);
    double b = tape[ACC];
    push_cell(tape, a - b);
}

void div(double tape[]) {
    pop_cell(tape, ACC);
    double a = tape[ACC];
    pop_cell(tape, ACC);
    double b = tape[ACC];
    push_cell(tape, a / b);
}

void mul(double tape[]) {
    pop_cell(tape, ACC);
    double a = tape[ACC];
    pop_cell(tape, ACC);
    double b = tape[ACC];
    push_cell(tape, a * b);
}

void dup(double tape[]) {
    pop_cell(tape, ACC);
    push_cell(tape, tape[ACC]);
    push_cell(tape, tape[ACC]);
}

void cmp(double tape[]) {
    pop_cell(tape, ACC);
    double a = tape[ACC];
    pop_cell(tape, ACC);
    double b = tape[ACC];
    if (a < b) {
        push_cell(tape, -1);
    } else if (a == b) {
        push_cell(tape, 0);
    } else if (a > b) {
        push_cell(tape, 1);
    }
}


void outc(double tape[]) {
    pop_cell(tape, ACC);
    printf("%c", (int)(tape[ACC]) % 256);
}

void outn(double tape[]) {
    pop_cell(tape, ACC);
    printf("%lG", tape[ACC]);
}

void inc(double tape[]) {
    char ch;
    if (scanf("%c", &ch) == 1) {
        push_cell(tape, ch);
    } else {
        push_cell(tape, 0);
    }
}

void inn(double tape[]) {
    double d;
    if (scanf("%lG", &d) == 1) {
        push_cell(tape, d);
    } else {
        push_cell(tape, 0);
    }
}

bool pop_bool(double tape[]) {
    pop_cell(tape, ACC);
    int a = tape[ACC];
    return a;
}


void lasm_alloc(double tape[], bool alloc_tape[], int ptr_addr) {
    pop_cell(tape, ACC);
    int size = tape[ACC];
    int cons_zeroes = 0;
    int i;
    for (i=MEMORY_SIZE-1; i>0; i--) {
        if (!alloc_tape[i]) {
            cons_zeroes++;
        } else {
            cons_zeroes = 0;
        }

        if (cons_zeroes == size) {
            push_cell(tape, i);
            pop_cell(tape, ptr_addr);
            break;
        }
    }

    for (int n=0; n<size; n++) {
        alloc_tape[i+n] = true;
    }
}

void lasm_free(double tape[], bool alloc_tape[], int ptr_addr) {
    pop_cell(tape, ACC);
    int size = tape[ACC];
    int addr = tape[ptr_addr];
    
    for (int n=0; n<size; n++) {
        tape[addr+n] = 0;
        alloc_tape[addr+n] = false;
    }
}

int main() {
    double tape[MEMORY_SIZE];
    bool alloc_tape[MEMORY_SIZE];
    for (int i=0; i<MEMORY_SIZE; i++) tape[i] = 0;
    for (int i=0; i<MEMORY_SIZE; i++) alloc_tape[i] = false;
    init(tape, alloc_tape);
"#;

        for line in &code {
            result += &(String::from("    ")
                + &match line {
                    Instruct::Refer(r) => format!("push_cell(tape, {});", r.get_addr()),
                    Instruct::DerefLoad => String::from("deref_load(tape);"),
                    Instruct::DerefStore => String::from("deref_store(tape);"),
                    Instruct::Alloc(r) => {
                        format!("lasm_alloc(tape, alloc_tape, {});", r.get_addr())
                    }
                    Instruct::Free(r) => format!("lasm_free(tape, alloc_tape, {});", r.get_addr()),
                    Instruct::Load(r) => format!("load(tape, {}, {});", r.get_addr(), r.get_size()),
                    Instruct::Store(r) => {
                        format!("store(tape, {}, {});", r.get_addr(), r.get_size())
                    }
                    Instruct::Push(l) => format!("push_cell(tape, {});", l.get()),
                    Instruct::Pop => String::from("pop_cell(tape, ACC);"),
                    Instruct::Duplicate => String::from("dup(tape);"),
                    Instruct::Add => String::from("add(tape);"),
                    Instruct::Subtract => String::from("sub(tape);"),
                    Instruct::Multiply => String::from("mul(tape);"),
                    Instruct::Divide => String::from("div(tape);"),
                    Instruct::InputChar => String::from("inc(tape);"),
                    Instruct::InputNumber => String::from("inn(tape);"),
                    Instruct::OutputChar => String::from("outc(tape);"),
                    Instruct::OutputNumber => String::from("outn(tape);"),
                    Instruct::Compare => String::from("cmp(tape);"),
                    Instruct::WhileNotZero => String::from("while (pop_bool(tape)) {"),
                    Instruct::EndWhile => String::from("}"),
                }
                + "\n");
        }

        result += r#"
        
    // printf("...\n\n");
    // for (int i=0; i<MEMORY_SIZE; i++) {
    //     if (i == INIT_STACK_PTR) {
    //         printf("STK_BGN ");
    //     }
    //     if (i == (int)tape[SPR]) {
    //         printf("STK_END ");
    //     }
    //     printf("%G ", tape[i]);
    // }
    // printf("\n...\n");
    // for (int i=0; i<MEMORY_SIZE; i++) {
    //     if (i == INIT_STACK_PTR) {
    //         printf("STK_BGN ");
    //     }
    //     if (i == (int)tape[SPR]) {
    //         printf("STK_END ");
    //     }
    //     printf("%d ", alloc_tape[i]);
    // }

    return 0;
}"#;
        result
    }
}
