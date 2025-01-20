/// VMProgramは.vmファイルの内容を保持する構造体
#[derive(PartialEq, Eq, Debug)]
pub struct VMProgram {
    commands: Vec<Command>,
}

impl VMProgram {
    // .vmファイルをparseする
    pub fn new(content: String) -> Self {
        let mut commands = vec![];
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            let terms: Vec<&str> = trimmed.split_whitespace().collect();

            // NOTE: trimmed.is_empty()の場合は早期リターンしているのでunwrapしても問題ないはず
            // TODO: Commandのメソッドとしたほうが凝集性高そう
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
                "push" => terms.get(1).zip(terms.get(2)).and_then(|(first_arg, second_arg)| {
                    second_arg
                        .parse::<u32>()
                        .ok()
                        .and_then(|index| Segment::new(first_arg, index))
                        .map(Command::Push)
                }),
                "pop" => terms.get(1).zip(terms.get(2)).and_then(|(first_arg, second_arg)| {
                    second_arg
                        .parse::<u32>()
                        .ok()
                        .and_then(|index| Segment::new(first_arg, index))
                        .map(Command::Pop)
                }),
                _ => None,
            };
            if let Some(command) = command {
                commands.push(command);
            }
        }

        Self { commands }
    }

    pub fn to_commands(&self) -> String {
        let init_commands = ["@256", "D=A", "@SP", "M=D"].iter().map(|c| c.to_string()).collect(); // SPの初期化コマンド
        let parsed_commands: Vec<String> = {
            // プログラム本体
            let mut result = vec![];
            for command in &self.commands {
                result.extend(command.to_commands())
            }
            result
        };
        let shutdown_commands = ["(END)", "@END", "0;JMP"].iter().map(|c| c.to_string()).collect(); // 終了用の無限ループ
                                                                                                    //
        [init_commands, parsed_commands, shutdown_commands].concat().join("\n")
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Command {
    Arithmetic(ArithmeticCommand),
    Push(Segment),
    Pop(Segment),
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
    // 機械語に翻訳するときにやること
    // ## push
    // 1. 値を取得
    //    segment[index]のメモリアドレスを解決(segmentのメモリアドレス + index)
    //    segment[index]の値を取得
    // 2. 取得した値をstackにpush

    // ## pop
    // 1. 値を取得
    //    segment[index]のメモリアドレスを解決(segmentのメモリアドレス + index)
    //    segment[index]の値を取得
    // 2. 取得した値をstackにpush
    fn to_commands(&self) -> Vec<String> {
        match self {
            Command::Arithmetic(ArithmeticCommand::Add) => {
                [
                    // 先頭2つの値をpopする
                    Command::Pop(Segment::Argument(1)).to_commands(),
                    Command::Pop(Segment::Argument(2)).to_commands(),
                    // addする
                    Segment::Argument(1).get_address_instructions(),
                    ["D=M".to_string()].to_vec(),
                    Segment::Argument(2).get_address_instructions(),
                    [
                        "D=D+M", "@SP", "A=M", "M=D", // 結果をpushする
                        "@SP", "M=M+1", // SPをインクリメント
                    ]
                    .map(|c| c.to_string())
                    .to_vec(),
                ]
                .concat()
                .to_vec()
            }
            Command::Push(segment) => [
                segment.clone().get_address_instructions(),
                [
                    format!("D={}", segment.get_value_register_name()).as_str(),
                    "@SP",
                    "A=M",
                    "M=D",
                    "@SP",
                    "M=M+1",
                ]
                .map(|c| c.to_string())
                .to_vec(),
            ]
            .concat(),
            Command::Pop(segment) => [
                ["@SP", "A=M", "D=M"].map(|c| c.to_string()).to_vec(),
                segment.get_address_instructions(),
                ["M=D"].map(|c| c.to_string()).to_vec(),
                ["@SP", "M=M-1"].map(|c| c.to_string()).to_vec(),
            ]
            .concat(),
            _ => todo!("not implemented"),
        }
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

#[derive(PartialEq, Eq, Debug, Clone)]
enum Segment {
    Argument(u32),
    Local(u32),
    Static(u32),
    Constant(u32),
    This(u32),
    That(u32),
    Pointer(u32),
    Temp(u32),
}

impl Segment {
    fn new(arg: &str, index: u32) -> Option<Self> {
        match arg {
            "argument" => Some(Self::Argument(index)),
            "local" => Some(Self::Local(index)),
            "static" => Some(Self::Static(index)),
            "constant" => Some(Self::Constant(index)),
            "this" => Some(Self::This(index)),
            "that" => Some(Self::That(index)),
            "pointer" => Some(Self::Pointer(index)),
            "temp" => Some(Self::Temp(index)),
            _ => None,
        }
    }

    // Segmentの実アドレスを返す命令群を返す
    fn get_address_instructions(&self) -> Vec<String> {
        match self {
            Self::Argument(index) => vec!["@ARG".to_string(), format!("A=M+{}", index)],
            Self::Local(index) => vec!["@LCL".to_string(), format!("A=M+{}", index)],
            Self::Static(index) => todo!("P177を参照して実装する"),
            Self::Constant(value) => vec![format!("@{}", value)],
            Self::This(index) => vec!["@THIS".to_string(), format!("A=M+{}", index)],
            Self::That(index) => vec!["@THAT".to_string(), format!("A=M+{}", index)],
            Self::Pointer(index) => todo!("P176を参照して実装する"),
            Self::Temp(index) => todo!("P176を参照して実装する"),
        }
    }

    // そのセグメントのデータが格納されているレジスタ名を返す(AまたはD)
    fn get_value_register_name(&self) -> String {
        if let Self::Constant(_) = self {
            "A".to_string()
        } else {
            "M".to_string()
        }
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
                    Command::Push(Segment::Constant(10)),
                    Command::Pop(Segment::Local(0)),
                    Command::Push(Segment::Constant(21)),
                    Command::Push(Segment::Constant(22)),
                    Command::Pop(Segment::Argument(2)),
                    Command::Pop(Segment::Argument(1)),
                    Command::Push(Segment::Constant(36)),
                    Command::Pop(Segment::This(6)),
                    Command::Push(Segment::Constant(42)),
                    Command::Push(Segment::Constant(45)),
                    Command::Pop(Segment::That(5)),
                    Command::Pop(Segment::That(2)),
                    Command::Push(Segment::Constant(510)),
                    Command::Pop(Segment::Temp(6)),
                    Command::Push(Segment::Local(0)),
                    Command::Push(Segment::That(5)),
                    Command::Arithmetic(ArithmeticCommand::Add),
                    Command::Push(Segment::Argument(1)),
                    Command::Arithmetic(ArithmeticCommand::Sub),
                    Command::Push(Segment::This(6)),
                    Command::Push(Segment::This(6)),
                    Command::Arithmetic(ArithmeticCommand::Add),
                    Command::Arithmetic(ArithmeticCommand::Sub),
                    Command::Push(Segment::Temp(6)),
                    Command::Arithmetic(ArithmeticCommand::Add),
                ]
            }
        );
    }
}
