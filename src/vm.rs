use crate::parser::Instruction;

#[derive(Clone)]
struct Snapshot {
    stack: Vec<i64>,
    instruction_pointer: usize,
}

pub struct Vm {
    stack: Vec<i64>,
    instruction_pointer: usize,
    program: Vec<Instruction>,
    history: Vec<Snapshot>,
}

impl Vm {
    pub fn new(program: Vec<Instruction>) -> Vm {
        Vm {
            stack: Vec::new(),
            instruction_pointer: 0,
            program,
            history: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        while self.step() {}
    }

    pub fn step(&mut self) -> bool {
        if self.instruction_pointer >= self.program.len() {
            println!("The program ended without HALT");
            return false;
        }

        self.history.push(Snapshot {
            stack: self.stack.clone(),
            instruction_pointer: self.instruction_pointer,
        });

        let instruction = self.program[self.instruction_pointer];
        self.instruction_pointer += 1;

        let result = match instruction {
            Instruction::Push(number) => {
                self.stack.push(number);
                Ok(())
            }
            Instruction::Add => self.do_math("add"),
            Instruction::Subtract => self.do_math("subtract"),
            Instruction::Multiply => self.do_math("multiply"),
            Instruction::Halt => return false,
        };

        if let Err(message) = result {
            // A failed instruction must not change anything: restore the
            // state we saved and stop running.
            let snapshot = self.history.pop().unwrap();
            self.stack = snapshot.stack;
            self.instruction_pointer = snapshot.instruction_pointer;
            println!("{message}");
            return false;
        }

        true
    }

    pub fn step_back(&mut self) -> bool {
        if self.history.is_empty() {
            println!("There are no steps to go back to");
            return false;
        }

        let last_snapshot = self.history.pop().unwrap();
        self.stack = last_snapshot.stack;
        self.instruction_pointer = last_snapshot.instruction_pointer;

        true
    }

    pub fn stack(&self) -> &[i64] {
        &self.stack
    }

    pub fn instruction_pointer(&self) -> usize {
        self.instruction_pointer
    }

    pub fn program(&self) -> &[Instruction] {
        &self.program
    }

    fn do_math(&mut self, math_type: &str) -> Result<(), String> {
        if self.stack.len() < 2 {
            return Err(format!("Not enough numbers on the stack to {math_type}"));
        }

        let right = self.stack.pop().unwrap();
        let left = self.stack.pop().unwrap();

        let answer = match math_type {
            "add" => left.checked_add(right),
            "subtract" => left.checked_sub(right),
            "multiply" => left.checked_mul(right),
            _ => unreachable!(),
        };

        match answer {
            Some(answer) => {
                self.stack.push(answer);
                Ok(())
            }
            None => Err(format!(
                "Overflow when trying to {math_type} {left} and {right}"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Vm;
    use crate::parser::{Instruction, assemble};

    fn program_from(source: &str) -> Vec<Instruction> {
        assemble(source).unwrap()
    }

    #[test]
    fn computes_a_full_program() {
        let mut vm = Vm::new(program_from(
            "push 2\npush 3\nadd\npush 4\nmultiply\npush 15\nsubtract\nhalt",
        ));

        vm.run();

        assert_eq!(vm.stack(), &[5]);
    }

    #[test]
    fn subtracts_left_minus_right() {
        let mut vm = Vm::new(program_from("push 20\npush 15\nsubtract\nhalt"));

        vm.run();

        assert_eq!(vm.stack(), &[5]);
    }

    #[test]
    fn leaves_everything_on_the_stack_after_halt() {
        let mut vm = Vm::new(program_from("push 2\npush 3\nadd\nhalt\npush 999"));

        vm.run();

        assert_eq!(vm.stack(), &[5]);
        assert_eq!(vm.instruction_pointer(), 4);
    }

    #[test]
    fn goes_back_to_the_stack_before_add() {
        let mut vm = Vm::new(program_from("push 2\npush 3\nadd\nhalt"));

        vm.step();
        vm.step();
        vm.step();
        vm.step_back();

        assert_eq!(vm.stack(), &[2, 3]);
        assert_eq!(vm.instruction_pointer(), 2);
    }

    #[test]
    fn step_back_only_goes_as_far_as_history_allows() {
        let mut vm = Vm::new(program_from("push 2\nhalt"));

        assert!(!vm.step_back());
        vm.step();
        assert!(vm.step_back());
        assert!(!vm.step_back());
    }

    #[test]
    fn rewinds_repeatedly_to_the_start() {
        let mut vm = Vm::new(program_from("push 1\npush 2\nadd\nhalt"));

        vm.step();
        vm.step();
        vm.step();
        vm.step_back();
        vm.step_back();
        vm.step_back();

        assert_eq!(vm.stack(), &[]);
        assert_eq!(vm.instruction_pointer(), 0);
    }

    #[test]
    fn stops_when_there_are_not_enough_numbers() {
        let mut vm = Vm::new(program_from("push 5\nsubtract\npush 100\nhalt"));

        assert!(vm.step());
        assert!(!vm.step());

        // The failed subtract changed nothing: stack and pointer are as if
        // it never ran, and it added no new history entry. One step back
        // goes straight to the start of the program.
        assert_eq!(vm.stack(), &[5]);
        assert_eq!(vm.instruction_pointer(), 1);
        assert!(vm.step_back());
        assert_eq!(vm.stack(), &[]);
        assert_eq!(vm.instruction_pointer(), 0);
        assert!(!vm.step_back());
    }

    #[test]
    fn stops_when_arithmetic_overflows() {
        let mut vm = Vm::new(program_from("push 9223372036854775807\npush 1\nadd\nhalt"));

        vm.step();
        vm.step();
        assert!(!vm.step());

        assert_eq!(vm.stack(), &[9223372036854775807, 1]);
        assert_eq!(vm.instruction_pointer(), 2);
    }

    #[test]
    fn stops_when_subtraction_overflows() {
        let mut vm = Vm::new(program_from("push -9223372036854775808\npush 1\nsubtract"));

        vm.step();
        vm.step();
        assert!(!vm.step());

        assert_eq!(vm.stack(), &[i64::MIN, 1]);
        assert_eq!(vm.instruction_pointer(), 2);
    }

    #[test]
    fn stops_when_multiplication_overflows() {
        let mut vm = Vm::new(program_from("push 9223372036854775807\npush 2\nmultiply"));

        vm.step();
        vm.step();
        assert!(!vm.step());

        assert_eq!(vm.stack(), &[i64::MAX, 2]);
        assert_eq!(vm.instruction_pointer(), 2);
    }

    #[test]
    fn step_after_the_end_reports_the_program_is_over() {
        let mut vm = Vm::new(program_from("push 1"));

        assert!(vm.step());
        assert!(!vm.step());
    }

    #[test]
    fn rewinds_past_halt() {
        let mut vm = Vm::new(program_from("push 1\nhalt"));

        vm.step();
        vm.step();

        assert!(vm.step_back());
        assert_eq!(vm.stack(), &[1]);
        assert_eq!(vm.instruction_pointer(), 1);

        assert!(vm.step_back());
        assert_eq!(vm.stack(), &[]);
        assert_eq!(vm.instruction_pointer(), 0);
    }

    #[test]
    fn run_stops_on_a_math_error() {
        let mut vm = Vm::new(program_from("push 5\nsubtract\npush 100\nhalt"));

        vm.run();

        assert_eq!(vm.stack(), &[5]);
        assert_eq!(vm.instruction_pointer(), 1);
    }
}
