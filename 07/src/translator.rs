/// VMProgramは.vmファイルの内容を保持する構造体
#[derive(PartialEq, Eq, Debug)]
pub struct VMProgram {
    commands: Vec<Command>,
}

impl VMProgram {
    pub fn new(content: String) -> Self {
        let mut commands = vec![];
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            let terms: Vec<&str> = trimmed.split_whitespace().collect();

            // NOTE: trimmed.is_empty()の場合は早期リターンしているのでunwrapしても問題ないはず
            let command = match *terms.first().unwrap() {
                "add" => Some(Command::Arithmetic(ArithmeticCommand::Add)),
                "sub" => Some(Command::Arithmetic(ArithmeticCommand::Sub)),
                "neg" => Some(Command::Arithmetic(ArithmeticCommand::Neg)),
                "eq" => Some(Command::Arithmetic(ArithmeticCommand::Eq)),
                "gt" => Some(Command::Arithmetic(ArithmeticCommand::Gt)),
                "lt" => Some(Command::Arithmetic(ArithmeticCommand::Lt)),
                "and" => Some(Command::Arithmetic(ArithmeticCommand::And)),
                "or" => Some(Command::Arithmetic(ArithmeticCommand::Or)),
                "not" => Some(Command::Arithmetic(ArithmeticCommand::Not)),
                "push" => match (terms.get(1), terms.get(2)) {
                    (Some(first_arg), Some(second_arg)) => match (Segment::new(first_arg), second_arg.parse::<u32>()) {
                        (Some(segment), Ok(index)) => Some(Command::Push(segment, index)),
                        _ => None,
                    },
                    _ => None,
                },
                "pop" => match (terms.get(1), terms.get(2)) {
                    (Some(first_arg), Some(second_arg)) => match (Segment::new(first_arg), second_arg.parse::<u32>()) {
                        (Some(segment), Ok(index)) => Some(Command::Pop(segment, index)),
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            };
            if let Some(command) = command {
                commands.push(command);
            }
        }

        Self { commands }
    }
}

impl std::fmt::Display for VMProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.commands
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Command {
    Arithmetic(ArithmeticCommand),
    Push(Segment, u32),
    Pop(Segment, u32),
    // これ以降のvariantは第8章で実装する
    #[allow(dead_code)]
    Label,
    #[allow(dead_code)]
    GoTo,
    #[allow(dead_code)]
    If,
    #[allow(dead_code)]
    Function,
    #[allow(dead_code)]
    Return,
    #[allow(dead_code)]
    Call,
}

impl Command {
    fn new(line: String) -> Self {
        Command::Call
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("impl");
        // write!(f, "{}", self.to_string())
    }
}

#[derive(PartialEq, Eq, Debug)]
enum ArithmeticCommand {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
}

impl std::fmt::Display for ArithmeticCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("impl");
        // write!(f, "{}", self.to_string())
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Segment {
    Argument,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp,
}

impl Segment {
    fn new(arg: &str) -> Option<Self> {
        match arg {
            "argument" => Some(Self::Argument),
            "local" => Some(Self::Local),
            "static" => Some(Self::Static),
            "constant" => Some(Self::Constant),
            "this" => Some(Self::This),
            "that" => Some(Self::That),
            "pointer" => Some(Self::Pointer),
            "temp" => Some(Self::Temp),
            _ => None,
        }
    }
}

impl std::fmt::Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("impl");
        // write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    #[test]
    fn test_vm_program_new() {
        assert_eq!(
            VMProgram::new(
                r#"
// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/7/MemoryAccess/BasicTest/BasicTest.vm

// Executes pop and push commands.

push constant 10
pop local 0
push constant 21
push constant 22
pop argument 2
pop argument 1
push constant 36
pop this 6
push constant 42
push constant 45
pop that 5
pop that 2
push constant 510
pop temp 6
push local 0
push that 5
add
push argument 1
sub
push this 6
push this 6
add
sub
push temp 6
add
                "#
                .to_string()
            ),
            VMProgram {
                commands: vec![
                    Command::Push(Segment::Constant, 10),
                    Command::Pop(Segment::Local, 0),
                    Command::Push(Segment::Constant, 21),
                    Command::Push(Segment::Constant, 22),
                    Command::Pop(Segment::Argument, 2),
                    Command::Pop(Segment::Argument, 1),
                    Command::Push(Segment::Constant, 36),
                    Command::Pop(Segment::This, 6),
                    Command::Push(Segment::Constant, 42),
                    Command::Push(Segment::Constant, 45),
                    Command::Pop(Segment::That, 5),
                    Command::Pop(Segment::That, 2),
                    Command::Push(Segment::Constant, 510),
                    Command::Pop(Segment::Temp, 6),
                    Command::Push(Segment::Local, 0),
                    Command::Push(Segment::That, 5),
                    Command::Arithmetic(ArithmeticCommand::Add),
                    Command::Push(Segment::Argument, 1),
                    Command::Arithmetic(ArithmeticCommand::Sub),
                    Command::Push(Segment::This, 6),
                    Command::Push(Segment::This, 6),
                    Command::Arithmetic(ArithmeticCommand::Add),
                    Command::Arithmetic(ArithmeticCommand::Sub),
                    Command::Push(Segment::Temp, 6),
                    Command::Arithmetic(ArithmeticCommand::Add),
                ]
            }
        );
    }
}
