/// VMProgramは.vmファイルの内容を保持する構造体
#[derive(PartialEq, Eq, Debug)]
pub struct VMProgram {
    commands: Vec<Command>,
    label_id: u32,     // 処理ごとにラベルを一意にしたいケースにsuffixとして利用する値
    file_name: String, // staticセグメントを機械語に変換する際に必要。`Foo.vm`で`static i`への参照があったとき`Foo.i`というシンボルを生成する。
}

impl VMProgram {
    // .vmファイルをparseする
    pub fn new(file_name: String, content: String) -> Self {
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

        Self {
            commands,
            label_id: 0,
            file_name,
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_hack_assembly(&mut self) -> String {
        let init_commands = ["// init", "@256", "D=A", "@SP", "M=D"]
            .iter()
            .map(|c| c.to_string())
            .collect(); // SPの初期化コマンド
        let parsed_commands: Vec<String> = {
            // プログラム本体
            let mut result = vec!["// body".to_string()];
            for command in &self.commands.clone() {
                let (a, should_increment_label_number) = command.to_commands(self.label_id, &self.file_name);
                result.extend(a);
                if should_increment_label_number {
                    self.increment_label_id();
                }
            }
            result
        };
        let shutdown_commands = ["// end", "(END)", "@END", "0;JMP"]
            .iter()
            .map(|c| c.to_string())
            .collect(); // 終了用の無限ループ
                        //
        [init_commands, parsed_commands, shutdown_commands].concat().join("\n")
    }

    fn increment_label_id(&mut self) {
        self.label_id += 1
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
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
    fn to_commands(&self, label_suffix: u32, file_name: &str) -> (Vec<String>, bool) {
        match self {
            Command::Arithmetic(ArithmeticCommand::Add) => {
                let commands = [
                    // x: RAM[SP-2], y: RAM[SP-1]としたときのx+yの結果を返す
                    vec!["// add".to_string()],
                    // 先頭2つの値をpopする
                    Command::Pop(Segment::Argument(1))
                        .to_commands(label_suffix, file_name)
                        .0,
                    Command::Pop(Segment::Argument(2))
                        .to_commands(label_suffix, file_name)
                        .0,
                    Segment::Argument(2).get_address_instructions(file_name), // x
                    vec!["D=M".to_string()],                                  // Dにxを格納
                    Segment::Argument(1).get_address_instructions(file_name), // y
                    [
                        // NOTE: 計算の順序をM+DではなくD+Mにしたいので先にxをDに格納している。
                        // P89 図4-5 に定義されている命令セットに厳密に従いたいのでこうしている。
                        // (D&Mは定義されているがM&Dは定義されていないのでMの前にDにが来るような順番で統一したい)
                        "D=D+M", // add
                        // 結果をpushする
                        "@SP", "A=M", "M=D", "@SP", "M=M+1",
                    ]
                    .iter()
                    .map(|c| c.to_string())
                    .collect(),
                ]
                .concat();
                (commands, false)
            }
            // TODO: 他のArithmeticCommandをすべて実装する
            Command::Arithmetic(ArithmeticCommand::Sub) => {
                let commands = [
                    // x: RAM[SP-2], y: RAM[SP-1]としたときのx-yの結果を返す
                    vec!["// sub".to_string()],
                    // 先頭2つの値をpopする
                    Command::Pop(Segment::Argument(1))
                        .to_commands(label_suffix, file_name)
                        .0, // y
                    Command::Pop(Segment::Argument(2))
                        .to_commands(label_suffix, file_name)
                        .0, // x
                    Segment::Argument(2).get_address_instructions(file_name), // x
                    vec!["D=M".to_string()],                                  // Dにxを格納
                    Segment::Argument(1).get_address_instructions(file_name), // y
                    [
                        "D=D-M", // sub
                        // 結果をpushする
                        "@SP", "A=M", "M=D", "@SP", "M=M+1",
                    ]
                    .iter()
                    .map(|c| c.to_string())
                    .collect(),
                ]
                .concat();
                (commands, false)
            }
            Command::Arithmetic(ArithmeticCommand::Neg) => {
                let commands = [
                    // !x
                    vec!["// neg".to_string()],
                    Command::Pop(Segment::Argument(1))
                        .to_commands(label_suffix, file_name)
                        .0,
                    Segment::Argument(1).get_address_instructions(file_name),
                    [
                        "D=-M", // neg
                        // 結果をpushする
                        "@SP", "A=M", "M=D", "@SP", "M=M+1",
                    ]
                    .iter()
                    .map(|c| c.to_string())
                    .collect(),
                ]
                .concat();
                (commands, false)
            }
            Command::Arithmetic(ArithmeticCommand::Eq) => {
                // x: RAM[SP-2], y: RAM[SP-1]としたときのx==yの結果を返す
                let true_label = format!("TRUE_{:05}", label_suffix);
                let false_label = format!("FALSE_{:05}", label_suffix);
                let end_if_label = format!("END_IF_{:05}", label_suffix);
                let commands = [
                    // NOTE: x == yを実現するために以下の判定を行う。
                    // x-y == 0のときはTRUE_suffixラベルに、x-y != 0のときはFALSE_suffixラベルに
                    // 移動する。それぞれのラベルの末尾でEND_suffixラベルに移動することで条件分岐を実現する
                    vec!["// eq".to_string()],
                    // 先頭2つの値をpopする
                    Command::Pop(Segment::Argument(1))
                        .to_commands(label_suffix, file_name)
                        .0, // y
                    Command::Pop(Segment::Argument(2))
                        .to_commands(label_suffix, file_name)
                        .0, // x
                    Segment::Argument(2).get_address_instructions(file_name), // x
                    vec!["D=M".to_string()],                                  // Dをxを格納
                    Segment::Argument(1).get_address_instructions(file_name), // y
                    [
                        "D=D-M", // x-y
                        format!("@{}", true_label).as_str(),
                        "D;JEQ",
                        // x != yの場合
                        format!("({})", false_label).as_str(),
                        "D=0",
                        format!("@{}", end_if_label).as_str(),
                        "0;JMP",
                        // x == yの場合
                        format!("({})", true_label).as_str(),
                        "D=-1",
                        format!("({})", end_if_label).as_str(),
                        // 結果をpushする
                        "@SP",
                        "A=M",
                        "M=D",
                        "@SP",
                        "M=M+1",
                    ]
                    .iter()
                    .map(|c| c.to_string())
                    .collect(),
                ]
                .concat();
                (commands, true)
            }
            Command::Arithmetic(ArithmeticCommand::Gt) => {
                // x: RAM[SP-2], y: RAM[SP-1]としたときのx>yの結果を返す
                let true_label = format!("TRUE_{:05}", label_suffix);
                let false_label = format!("FALSE_{:05}", label_suffix);
                let end_if_label = format!("END_IF_{:05}", label_suffix);
                let commands = [
                    vec!["// gt".to_string()],
                    // 先頭2つの値をpopする
                    Command::Pop(Segment::Argument(1))
                        .to_commands(label_suffix, file_name)
                        .0, // y
                    Command::Pop(Segment::Argument(2))
                        .to_commands(label_suffix, file_name)
                        .0, // x
                    Segment::Argument(2).get_address_instructions(file_name), // x
                    vec!["D=M".to_string()],                                  // Dをxを格納
                    Segment::Argument(1).get_address_instructions(file_name), // y
                    [
                        "D=D-M", // x-y
                        format!("@{}", true_label).as_str(),
                        "D;JGT",
                        // x <= yの場合
                        format!("({})", false_label).as_str(),
                        "D=0",
                        format!("@{}", end_if_label).as_str(),
                        "0;JMP",
                        // x > yの場合
                        format!("({})", true_label).as_str(),
                        "D=-1",
                        format!("({})", end_if_label).as_str(),
                        // 結果をpushする
                        "@SP",
                        "A=M",
                        "M=D",
                        "@SP",
                        "M=M+1",
                    ]
                    .iter()
                    .map(|c| c.to_string())
                    .collect(),
                ]
                .concat();
                (commands, true)
            }
            Command::Arithmetic(ArithmeticCommand::Lt) => {
                // x: RAM[SP-2], y: RAM[SP-1]としたときのx<yの結果を返す
                let true_label = format!("TRUE_{:05}", label_suffix);
                let false_label = format!("FALSE_{:05}", label_suffix);
                let end_if_label = format!("END_IF_{:05}", label_suffix);
                let commands = [
                    vec!["// lt".to_string()],
                    // 先頭2つの値をpopする
                    Command::Pop(Segment::Argument(1))
                        .to_commands(label_suffix, file_name)
                        .0, // y
                    Command::Pop(Segment::Argument(2))
                        .to_commands(label_suffix, file_name)
                        .0, // x
                    Segment::Argument(2).get_address_instructions(file_name), // x
                    vec!["D=M".to_string()],                                  // Dをxを格納
                    Segment::Argument(1).get_address_instructions(file_name), // y
                    [
                        "D=D-M", // x-y
                        format!("@{}", true_label).as_str(),
                        "D;JLT",
                        // x >= yの場合
                        format!("({})", false_label).as_str(),
                        "D=0",
                        format!("@{}", end_if_label).as_str(),
                        "0;JMP",
                        // x < yの場合
                        format!("({})", true_label).as_str(),
                        "D=-1",
                        format!("({})", end_if_label).as_str(),
                        // 結果をpushする
                        "@SP",
                        "A=M",
                        "M=D",
                        "@SP",
                        "M=M+1",
                    ]
                    .iter()
                    .map(|c| c.to_string())
                    .collect(),
                ]
                .concat();
                (commands, true)
            }
            Command::Arithmetic(ArithmeticCommand::And) => {
                let commands = [
                    // x: RAM[SP-2], y: RAM[SP-1]としたときのx&yを行う
                    vec!["// and".to_string()],
                    // 先頭2つの値をpopする
                    Command::Pop(Segment::Argument(1))
                        .to_commands(label_suffix, file_name)
                        .0, // y
                    Command::Pop(Segment::Argument(2))
                        .to_commands(label_suffix, file_name)
                        .0, // x
                    Segment::Argument(2).get_address_instructions(file_name), // x
                    vec!["D=M".to_string()],                                  // Dをxを格納
                    Segment::Argument(1).get_address_instructions(file_name), // y
                    [
                        "D=D&M", // and
                        "@SP", "A=M", "M=D", // 結果をpushする
                        "@SP", "M=M+1",
                    ]
                    .iter()
                    .map(|c| c.to_string())
                    .collect(),
                ]
                .concat();
                (commands, false)
            }
            Command::Arithmetic(ArithmeticCommand::Or) => {
                let commands = [
                    // x: RAM[SP-2], y: RAM[SP-1]としたときのx&yを行う
                    vec!["// or".to_string()],
                    // 先頭2つの値をpopする
                    Command::Pop(Segment::Argument(1))
                        .to_commands(label_suffix, file_name)
                        .0, // y
                    Command::Pop(Segment::Argument(2))
                        .to_commands(label_suffix, file_name)
                        .0, // x
                    Segment::Argument(2).get_address_instructions(file_name), // x
                    vec!["D=M".to_string()],                                  // Dをxを格納
                    Segment::Argument(1).get_address_instructions(file_name), // y
                    [
                        "D=D|M", // or
                        "@SP", "A=M", "M=D", // 結果をpushする
                        "@SP", "M=M+1",
                    ]
                    .iter()
                    .map(|c| c.to_string())
                    .collect(),
                ]
                .concat();
                (commands, false)
            }
            Command::Arithmetic(ArithmeticCommand::Not) => {
                let commands = [
                    vec!["// not".to_string()],
                    // 先頭2つの値をpopする
                    Command::Pop(Segment::Argument(1))
                        .to_commands(label_suffix, file_name)
                        .0,
                    Segment::Argument(1).get_address_instructions(file_name),
                    [
                        "D=!M", // not
                        "@SP", "A=M", "M=D", // 結果をpushする
                        "@SP", "M=M+1",
                    ]
                    .iter()
                    .map(|c| c.to_string())
                    .collect(),
                ]
                .concat();
                (commands, false)
            }
            Command::Push(segment) => {
                let commands = [
                    vec!["// push".to_string()],
                    segment.clone().get_address_instructions(file_name),
                    vec![
                        format!("D={}", segment.get_value_register_name()).as_str(),
                        "@SP",
                        "A=M",
                        "M=D",
                        "@SP",
                        "M=M+1",
                    ]
                    .into_iter()
                    .map(|c| c.to_string())
                    .collect(),
                ]
                .concat();
                (commands, false)
            }
            Command::Pop(segment) => {
                let commands = [
                    vec!["// pop", "@SP", "A=M-1", "D=M", "M=0"]
                        .into_iter()
                        .map(|c| c.to_string())
                        .collect(), // RAM[SP]の値をDに格納しMを初期化
                    segment.get_address_instructions(file_name), // Aにpopのdescを設定(ここでDを使うのでRAM[SP]の値が消えてしまうので注意)
                    vec!["M=D"].into_iter().map(|c| c.to_string()).collect(), // L34
                    vec!["@SP", "M=M-1"].into_iter().map(|c| c.to_string()).collect(),
                ]
                .concat();
                (commands, false)
            }
            Command::Label => todo!(),
            Command::GoTo => todo!(),
            Command::If => todo!(),
            Command::Function => todo!(),
            Command::Return => todo!(),
            Command::Call => todo!(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
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
    fn get_address_instructions(&self, file_name: &str) -> Vec<String> {
        match self {
            Self::Argument(index) => {
                // format!("@{}", index).as_str(), "A=D+A" のようにすれば対象のアドレスを取得できるが意図的にA=A+1の繰り返しで処理している。
                // Dレジスタを使ってしまうとpopの処理時にSPの値を記憶しておくことができなくなってしまうため。
                [
                    vec![format!("// argument {}", index), format!("@{}", self.segment_name())],
                    vec!["A=A+1".to_string(); *index as usize],
                ]
                .concat()
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
            }
            Self::Local(index) => ["@LCL", format!("A=M+{}", index).as_str()]
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>(),

            Self::Static(index) => vec![format!("@{}.{}", file_name, index)],
            Self::Constant(value) => [format!("@{}", value).as_str()]
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>(),

            Self::This(index) => ["@THIS", format!("A=M+{}", index).as_str()]
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>(),

            Self::That(index) => ["@THAT", format!("A=M+{}", index).as_str()]
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>(),

            Self::Pointer(_index) => todo!("P176を参照して実装する"),
            Self::Temp(_index) => todo!("P176を参照して実装する"),
        }
        .iter()
        .map(|c| c.to_string())
        .collect()
    }

    // そのセグメントのデータが格納されているレジスタ名を返す(AまたはD)
    fn get_value_register_name(&self) -> String {
        if let Self::Constant(_) = self {
            "A".to_string()
        } else {
            "M".to_string()
        }
    }

    fn segment_name(&self) -> String {
        match self {
            Self::Argument(_) => "ARG",
            Self::Local(_) => "LCL",
            Self::Static(_) => "STATIC",
            Self::Constant(_) => "CONST",
            Self::This(_) => "THIS",
            Self::That(_) => "THAT",
            Self::Pointer(_) => "POINTER",
            Self::Temp(_) => "TEMP",
        }
        .to_string()
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
                .to_string(),
                "foo".to_string(),
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
                ],
                label_id: 0,
                file_name: "foo".to_string(),
            }
        );
    }
}
