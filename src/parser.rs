#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Instruction {
    Push(i64),
    Add,
    Subtract,
    Multiply,
    Halt,
}

pub fn assemble(source_code: &str) -> Result<Vec<Instruction>, String> {
    // A UTF-8 byte-order mark at the start of the file would otherwise become
    // part of the first instruction's name.
    let source_code = source_code.trim_start_matches('\u{feff}');
    let mut program = Vec::new();

    for (line_index, line) in source_code.lines().enumerate() {
        let line_number = line_index + 1;
        let line = line.split('#').next().unwrap().trim();

        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        match parts[0] {
            "push" => {
                if parts.len() != 2 {
                    return Err(format!("Line {line_number}: push needs one number"));
                }

                let number = parts[1]
                    .parse::<i64>()
                    .map_err(|_| format!("Line {line_number}: '{}' is not a number", parts[1]))?;

                program.push(Instruction::Push(number));
            }
            "add" | "subtract" | "multiply" | "halt" => {
                if parts.len() != 1 {
                    return Err(format!(
                        "Line {line_number}: {} does not take arguments",
                        parts[0]
                    ));
                }
                let instruction = match parts[0] {
                    "add" => Instruction::Add,
                    "subtract" => Instruction::Subtract,
                    "multiply" => Instruction::Multiply,
                    "halt" => Instruction::Halt,
                    _ => unreachable!(),
                };
                program.push(instruction);
            }
            _ => return Err(format!("Line {line_number}: I do not know '{}'.", parts[0])),
        }
    }

    if program.is_empty() {
        return Err("The program is empty".to_string());
    }

    Ok(program)
}

pub fn instruction_text(instruction: Instruction) -> String {
    match instruction {
        Instruction::Push(number) => format!("push {number}"),
        Instruction::Add => "add".to_string(),
        Instruction::Subtract => "subtract".to_string(),
        Instruction::Multiply => "multiply".to_string(),
        Instruction::Halt => "halt".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Instruction, assemble};

    #[test]
    fn assembles_a_small_program() {
        let program = assemble("push 2\npush 3\nadd\nhalt").unwrap();

        assert_eq!(
            program,
            vec![
                Instruction::Push(2),
                Instruction::Push(3),
                Instruction::Add,
                Instruction::Halt,
            ]
        );
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        let program = assemble("# a comment\n\npush 2 # inline\nadd\n").unwrap();

        assert_eq!(program, vec![Instruction::Push(2), Instruction::Add]);
    }

    #[test]
    fn pushes_negative_numbers() {
        let program = assemble("push -7\nhalt").unwrap();

        assert_eq!(program, vec![Instruction::Push(-7), Instruction::Halt]);
    }

    #[test]
    fn tells_you_which_line_has_a_problem() {
        let error = assemble("push 2\nplus\nhalt").unwrap_err();

        assert_eq!(error, "Line 2: I do not know 'plus'.");
    }

    #[test]
    fn rejects_push_without_a_number() {
        let error = assemble("push\nhalt").unwrap_err();

        assert_eq!(error, "Line 1: push needs one number");
    }

    #[test]
    fn rejects_push_with_a_non_number() {
        let error = assemble("push abc\nhalt").unwrap_err();

        assert_eq!(error, "Line 1: 'abc' is not a number");
    }

    #[test]
    fn rejects_an_empty_program() {
        let error = assemble("# nothing here").unwrap_err();

        assert_eq!(error, "The program is empty");
    }

    #[test]
    fn rejects_extra_arguments_on_zero_arg_instructions() {
        let error = assemble("add 5").unwrap_err();

        assert_eq!(error, "Line 1: add does not take arguments");
    }

    #[test]
    fn rejects_arguments_on_halt() {
        let error = assemble("halt now").unwrap_err();

        assert_eq!(error, "Line 1: halt does not take arguments");
    }

    #[test]
    fn strips_a_utf8_bom_before_the_first_instruction() {
        let program = assemble("\u{feff}push 2\nhalt").unwrap();

        assert_eq!(program, vec![Instruction::Push(2), Instruction::Halt]);
    }
}
