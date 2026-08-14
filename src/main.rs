mod parser;
mod vm;

use std::env;
use std::fs;
use std::io::{self, Write};

use parser::instruction_text;
use vm::Vm;

fn print_result(vm: &Vm) {
    if vm.stack().is_empty() {
        println!("The program did not leave a result on the stack");
    } else {
        println!("result: {}", vm.stack()[0]);
    }
}

fn run_with_trace(vm: &mut Vm) {
    println!("Memento trace\n");

    loop {
        if vm.instruction_pointer() >= vm.program().len() {
            println!("The program ended without HALT");
            break;
        }

        let instruction = vm.program()[vm.instruction_pointer()];

        println!(
            "step {}: {}",
            vm.instruction_pointer(),
            instruction_text(instruction)
        );
        println!("  stack before: {:?}", vm.stack());

        let keep_running = vm.step();
        println!("  stack after:  {:?}\n", vm.stack());

        if !keep_running {
            break;
        }
    }

    print_result(vm);
}

fn print_debug_state(vm: &Vm) {
    println!("stack: {:?}", vm.stack());

    if vm.instruction_pointer() < vm.program().len() {
        let instruction = vm.program()[vm.instruction_pointer()];
        println!(
            "next instruction ({}): {}",
            vm.instruction_pointer(),
            instruction_text(instruction)
        );
    } else {
        println!("next instruction: finished");
    }
}

fn run_debugger(vm: &mut Vm) {
    run_debugger_loop(vm, io::stdin().lock());
}

fn run_debugger_loop(vm: &mut Vm, mut input: impl io::BufRead) {
    println!("Memento debugger");
    println!("Type n for next, b for back, or q to quit.\n");
    print_debug_state(vm);

    loop {
        print!("\n> ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        match input.read_line(&mut command) {
            // End of input: quit instead of asking for a command forever.
            Ok(0) | Err(_) => {
                println!();
                break;
            }
            Ok(_) => {}
        }

        match command.trim() {
            "n" => {
                if vm.instruction_pointer() >= vm.program().len() {
                    println!("The program is already finished");
                } else {
                    vm.step();
                    print_debug_state(vm);
                }
            }
            "b" => {
                vm.step_back();
                print_debug_state(vm);
            }
            "q" => break,
            _ => println!("Please type n, b, or q"),
        }
    }
}

fn main() {
    let arguments: Vec<String> = env::args().collect();

    if arguments.len() < 2 || arguments.len() > 3 {
        println!("Usage: cargo run -- <program file> [--trace | --debug]");
        return;
    }

    let source_code = match fs::read_to_string(&arguments[1]) {
        Ok(source_code) => source_code,
        Err(error) => {
            println!("Could not read '{}': {error}", arguments[1]);
            return;
        }
    };

    let program = match parser::assemble(&source_code) {
        Ok(program) => program,
        Err(error) => {
            println!("Could not assemble the program: {error}");
            return;
        }
    };

    let mut vm = Vm::new(program);

    if arguments.len() == 3 && arguments[2] == "--trace" {
        run_with_trace(&mut vm);
    } else if arguments.len() == 3 && arguments[2] == "--debug" {
        run_debugger(&mut vm);
    } else if arguments.len() == 2 {
        vm.run();
        print_result(&vm);
    } else {
        println!("I do not know '{}'.", arguments[2]);
        println!("Try --trace or --debug");
    }
}

#[cfg(test)]
mod tests {
    use super::run_debugger_loop;
    use crate::parser::assemble;
    use crate::vm::Vm;
    use std::io;

    #[test]
    fn debugger_quits_when_input_ends() {
        let mut vm = Vm::new(assemble("push 1\nhalt").unwrap());

        // End-of-file must quit the debugger, not loop forever.
        run_debugger_loop(&mut vm, io::empty());
    }
}
