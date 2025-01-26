/// VMProgramは.vmファイルの内容を保持する構造体
#[derive(PartialEq, Eq, Debug)]
pub struct VMProgram {
    commands: Vec<Command>,
    // 処理ごとにラベルを一意にしたいケースにsuffixとして利用する値
    label_id: u32,
    // staticセグメントを機械語に変換する際に必要。`Foo.vm`で`static i`への参照があったとき`Foo.i`というシンボルを生成する。
    file_name: String,
    // returnアドレスのsuffix
    // Xxx.vmの中のfoo関数の中で任意の関数を呼び出したとき、`Xxx.foo$ret.{return_address_id}`
    return_address_id: u32,
    // 現在の命令が所属する関数名
    current_function_name: String,
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
                "label" => terms.get(1).map(|first_arg| Command::Label(first_arg.to_string())),
                "goto" => terms.get(1).map(|first_arg| Command::GoTo(first_arg.to_string())),
                "if-goto" => terms.get(1).map(|first_arg| Command::IfGoTo(first_arg.to_string())),
                "function" => terms.get(1).zip(terms.get(2)).and_then(|(function_name, vars_length)| {
                    vars_length
                        .parse::<u32>()
                        .ok()
                        .map(|len| Command::Function(function_name.to_string(), len))
                }),
                "call" => terms.get(1).zip(terms.get(2)).and_then(|(function_name, args_length)| {
                    args_length
                        .parse::<u32>()
                        .ok()
                        .map(|len| Command::Call(function_name.to_string(), len))
                }),
                "return" => Some(Command::Return),
                _ => todo!("{} is invalid command", *terms.first().unwrap()),
            };
            if let Some(command) = command {
                commands.push(command);
            }
        }

        Self {
            commands,
            label_id: 0,
            file_name,
            return_address_id: 0,
            current_function_name: String::new(),
        }
    }

    pub fn combine_and_assemble(programs: Vec<VMProgram>) -> String {
        let mut result: Vec<String>;

        let init_stack_pointer = [vec!["// init", "@256", "D=A", "@SP", "M=D"]]
            .concat()
            .iter()
            .map(|c| c.to_string())
            .collect();
        // TODO: to_commandsの第1引数、本当はSys.initが定義されているファイル名を取る必要がある
        let (call_init, _, _, _) = Command::Call("Sys.init".to_string(), 0).to_commands("Sys", 0, 0, "Init");
        result = [init_stack_pointer, call_init].concat();

        for mut p in programs {
            result = [result, p.to_machine_language()].concat()
        }
        let shutdown_loop = ["// end", "(END)", "@END", "0;JMP"]
            .iter()
            .map(|c| c.to_string())
            .collect(); // 終了用の無限ループ
        result = [result, shutdown_loop].concat();

        result.join("\n")
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_machine_language(&mut self) -> Vec<String> {
        let mut result = vec!["// body".to_string()];
        for command in &self.commands.clone() {
            let (commands, should_increment_label_number, should_increment_return_address_id, new_function_name) =
                command.to_commands(
                    &self.file_name,
                    self.label_id,
                    self.return_address_id,
                    &self.current_function_name,
                );
            result.extend(commands);

            if should_increment_label_number {
                self.increment_label_id();
            }
            if should_increment_return_address_id {
                self.increment_return_address_id();
            }
            if let Some(func_name) = new_function_name {
                self.update_current_function_name(func_name);
            }
        }
        result
    }

    fn increment_label_id(&mut self) {
        self.label_id += 1
    }

    fn increment_return_address_id(&mut self) {
        self.return_address_id += 1
    }

    fn update_current_function_name(&mut self, function_name: String) {
        self.current_function_name = function_name;
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum Command {
    Arithmetic(ArithmeticCommand),
    Push(Segment),
    Pop(Segment),
    Label(String),
    GoTo(String),
    IfGoTo(String),
    Call(String, u32),
    Function(String, u32),
    Return,
}

impl Command {
    // command, should_increment_label_number, should_increment_return_address_id,
    // new_function_nameを返す
    fn to_commands(
        &self,
        file_name: &str,
        label_suffix: u32,
        return_address_id: u32,
        current_function_name: &str,
    ) -> (Vec<String>, bool, bool, Option<String>) {
        // RAM[SP]にDを格納する
        let push_d: Vec<&str> = vec!["@SP", "A=M", "M=D", "@SP", "M=M+1"];

        // 1つのオペランドを取る処理の前処理
        // RAM[SP-1]をMに格納する
        let get_1_operand: Vec<&str> = vec!["@SP", "A=M", "A=A-1"];

        // 2つのオペランドを取る処理の前処理
        // 計算の順序をM+DではなくD+Mにしたいので先にxをDに格納している。
        // P89 図4-5 に定義されている命令セットに厳密に従いたいのでこうしている。
        // (D&Mは定義されているがM&Dは定義されていないのでMの前にDにが来るような順番で統一したい)
        let get_2_operand: Vec<&str> = [
            // RAM[SP-2]をDに格納
            vec!["@SP", "A=M", "A=A-1", "A=A-1", "D=M"],
            // RAM[SP-1]をMに格納
            vec!["@SP", "A=M", "A=A-1"],
        ]
        .concat();

        // 1つのオペランドを取る計算の結果を格納する(計算結果がDに入っていることを期待している)
        let save_result_1_operand: Vec<&str> = [
            // RAM[SP-1]をMに格納
            vec!["@SP", "A=M", "A=A-1"],
            // RAM[SP-1]を0にする
            vec!["M=0"],
            // SPをSP-1する
            vec!["@SP", "M=M-1"],
            // 結果をpushする
            push_d.clone(),
        ]
        .concat();

        // 2つのオペランドを取る計算の結果を格納する(計算結果がDに入っていることを期待している)
        let save_result_2_operand: Vec<&str> = [
            // RAM[SP-2]を0にする
            vec!["@SP", "A=M", "A=A-1", "A=A-1", "M=0"],
            // RAM[SP-1]を0にする
            vec!["@SP", "A=M", "A=A-1", "M=0"],
            // SPをSP-2する
            vec!["@SP", "M=M-1", "M=M-1"],
            // 結果をpushする
            push_d.clone(),
        ]
        .concat();

        // NOTE: 各命令を["@FOO", "M=0"]のように配列の形で書くのが一番シンプルだが、
        // そうするとコメントが意図しない位置で改行されてしまうのでvectorの配列をconcatしている。
        match self {
            Command::Arithmetic(ArithmeticCommand::Add) => {
                // x: RAM[SP-2], y: RAM[SP-1]としたときのx+yの結果を返す
                let commands = [
                    vec![format!("// {:?}", self).as_str()],
                    get_2_operand,
                    // add
                    vec!["D=D+M"],
                    save_result_2_operand,
                ]
                .concat()
                .iter()
                .map(|c| c.to_string())
                .collect();
                (commands, false, false, None)
            }

            Command::Arithmetic(ArithmeticCommand::Sub) => {
                // x: RAM[SP-2], y: RAM[SP-1]としたときのx-yの結果を返す
                let commands = [
                    vec![format!("// {:?}", self).as_str()],
                    get_2_operand,
                    // sub
                    vec!["D=D-M"],
                    save_result_2_operand,
                ]
                .concat()
                .iter()
                .map(|c| c.to_string())
                .collect();
                (commands, false, false, None)
            }

            Command::Arithmetic(ArithmeticCommand::Neg) => {
                // x: RAM[SP-1]としたときの-xの結果を返す
                let commands = [
                    vec![format!("// {:?}", self).as_str()],
                    get_1_operand,
                    // neg
                    vec!["D=-M"],
                    save_result_1_operand,
                ]
                .concat()
                .iter()
                .map(|c| c.to_string())
                .collect();
                (commands, false, false, None)
            }

            Command::Arithmetic(ArithmeticCommand::Eq) => {
                // x: RAM[SP-2], y: RAM[SP-1]としたときのx==yの結果を返す
                let true_label = format!("TRUE_{:05}", label_suffix);
                let false_label = format!("FALSE_{:05}", label_suffix);
                let end_if_label = format!("END_IF_{:05}", label_suffix);
                let commands = [
                    vec![format!("// {:?}", self).as_str()],
                    get_2_operand,
                    // eqの判定ここから
                    vec!["D=D-M"], // x-y
                    // x-y==0ならtrue_labelにジャンプ
                    vec![format!("@{}", true_label).as_str(), "D;JEQ"],
                    // x != yの場合
                    vec![
                        format!("({})", false_label).as_str(),
                        "D=0",
                        format!("@{}", end_if_label).as_str(),
                        "0;JMP",
                    ],
                    // x == yの場合
                    vec![format!("({})", true_label).as_str(), "D=-1"],
                    // end_if_label
                    vec![format!("({})", end_if_label).as_str()],
                    // eqの判定ここまで
                    save_result_2_operand,
                ]
                .concat()
                .iter()
                .map(|c| c.to_string())
                .collect();
                (commands, true, false, None)
            }

            Command::Arithmetic(ArithmeticCommand::Gt) => {
                // x: RAM[SP-2], y: RAM[SP-1]としたときのx>yの結果を返す
                let true_label = format!("TRUE_{:05}", label_suffix);
                let false_label = format!("FALSE_{:05}", label_suffix);
                let end_if_label = format!("END_IF_{:05}", label_suffix);
                let commands = [
                    // x: RAM[SP-2], y: RAM[SP-1]としたときのx>yの結果を返す
                    vec![format!("// {:?}", self).as_str()],
                    get_2_operand,
                    // gtの判定ここから
                    vec!["D=D-M"], // x-y
                    // x-y==0ならtrue_labelにジャンプ
                    vec![format!("@{}", true_label).as_str(), "D;JGT"],
                    // x != yの場合
                    vec![
                        format!("({})", false_label).as_str(),
                        "D=0",
                        format!("@{}", end_if_label).as_str(),
                        "0;JMP",
                    ],
                    // x == yの場合
                    vec![format!("({})", true_label).as_str(), "D=-1"],
                    // end_if_label
                    vec![format!("({})", end_if_label).as_str()],
                    // gtの判定ここまで
                    save_result_2_operand,
                ]
                .concat()
                .iter()
                .map(|c| c.to_string())
                .collect();
                (commands, true, false, None)
            }

            Command::Arithmetic(ArithmeticCommand::Lt) => {
                // x: RAM[SP-2], y: RAM[SP-1]としたときのx<yの結果を返す
                let true_label = format!("TRUE_{:05}", label_suffix);
                let false_label = format!("FALSE_{:05}", label_suffix);
                let end_if_label = format!("END_IF_{:05}", label_suffix);
                let commands = [
                    vec![format!("// {:?}", self).as_str()],
                    get_2_operand,
                    // ltの判定ここから
                    vec!["D=D-M"], // x-y
                    // x-y==0ならtrue_labelにジャンプ
                    vec![format!("@{}", true_label).as_str(), "D;JLT"],
                    // x != yの場合
                    vec![
                        format!("({})", false_label).as_str(),
                        "D=0",
                        format!("@{}", end_if_label).as_str(),
                        "0;JMP",
                    ],
                    // x == yの場合
                    vec![format!("({})", true_label).as_str(), "D=-1"],
                    // end_if_label
                    vec![format!("({})", end_if_label).as_str()],
                    // ltの判定ここまで
                    save_result_2_operand,
                ]
                .concat()
                .iter()
                .map(|c| c.to_string())
                .collect();
                (commands, true, false, None)
            }

            Command::Arithmetic(ArithmeticCommand::And) => {
                // x: RAM[SP-2], y: RAM[SP-1]としたときのx&yの結果を返す
                let commands = [
                    vec![format!("// {:?}", self).as_str()],
                    get_2_operand,
                    // and
                    vec!["D=D&M"],
                    save_result_2_operand,
                ]
                .concat()
                .iter()
                .map(|c| c.to_string())
                .collect();
                (commands, false, false, None)
            }

            Command::Arithmetic(ArithmeticCommand::Or) => {
                let commands = [
                    vec![format!("// {:?}", self).as_str()],
                    get_2_operand,
                    // or
                    vec!["D=D|M"],
                    save_result_2_operand,
                ]
                .concat()
                .iter()
                .map(|c| c.to_string())
                .collect();
                (commands, false, false, None)
            }

            Command::Arithmetic(ArithmeticCommand::Not) => {
                // x: RAM[SP-1]としたときの!xの結果を返す
                let commands = [
                    vec![format!("// {:?}", self).as_str()],
                    get_1_operand,
                    // not
                    vec!["D=!M"],
                    save_result_1_operand,
                ]
                .concat()
                .iter()
                .map(|c| c.to_string())
                .collect();
                (commands, false, false, None)
            }

            Command::Push(segment) => {
                let commands = [
                    vec![format!("// {:?}", self)],
                    segment.clone().get_address_instructions(file_name),
                    vec![format!("D={}", segment.get_value_register_name()).as_str()]
                        .into_iter()
                        .map(|c| c.to_string())
                        .collect(),
                    push_d.clone().into_iter().map(|c| c.to_string()).collect(),
                ]
                .concat();
                (commands, false, false, None)
            }

            Command::Pop(segment) => {
                let commands = [
                    vec![format!("// {:?}", self)],
                    vec!["@SP", "A=M-1", "D=M", "M=0"]
                        .into_iter()
                        .map(|c| c.to_string())
                        .collect(), // RAM[SP]の値をDに格納しMを初期化
                    segment.get_address_instructions(file_name), // Aにpopのdescを設定(ここでDを使うのでRAM[SP]の値が消えてしまうので注意)
                    vec!["M=D"].into_iter().map(|c| c.to_string()).collect(), // L34
                    vec!["@SP", "M=M-1"].into_iter().map(|c| c.to_string()).collect(),
                ]
                .concat();
                (commands, false, false, None)
            }

            Command::Label(label_name) => {
                let commands = [vec![format!("// {:?}", self)], vec![format!("({})", label_name)]].concat();
                (commands, false, false, None)
            }
            Command::GoTo(label_name) => {
                let commands = [
                    vec![format!("// {:?}", self)],
                    vec![format!("@{}", label_name), "0;JMP".to_string()],
                ]
                .concat();
                (commands, false, false, None)
            }
            Command::IfGoTo(label_name) => {
                let commands = [
                    vec![format!("// {:?}", self)],
                    // スタックの最上位の値xをpopし、x!=0ならばJUMPする
                    vec![
                        "@SP",
                        "A=M-1",
                        "D=M",
                        "M=0",
                        "@SP",
                        "M=M-1",
                        format!("@{}", label_name).as_str(),
                        "D;JNE",
                    ]
                    .into_iter()
                    .map(|c| c.to_string())
                    .collect(),
                ]
                .concat();
                (commands, false, false, None)
            }
            Command::Call(function_name, vars_length) => {
                let return_address_label = format!(
                    "{}.{}$ret.{}", // リターンアドレスを宣言し、Dに格納
                    file_name, current_function_name, return_address_id
                );
                // defined_file_nameは本来はVec<VMProgram>を走査して定義されているファイルを特定する必要があるが
                // ここではfunction
                // Class1.getのようにファイル名が先頭につくのでそれを利用している。
                let defined_file_name = function_name.split('.').collect::<Vec<&str>>()[0].to_string();
                let go_to_address_label = format!(
                    "{}.{}", // 呼び出し先関数のアドレスを宣言し、Dに格納
                    defined_file_name, function_name
                );
                let commands = [
                    vec![format!("// {:?}", self).as_str()],
                    // リターンアドレスを宣言してスタックにpush
                    vec![format!("@{}", return_address_label).as_str(), "D=A"],
                    push_d.clone(),
                    // LCL(RAM[1]: ローカルのベースアドレス)をpushし元のデータを消去
                    vec!["@1", "D=M"],
                    push_d.clone(),
                    // ARG(RAM[2]: argumentのベースアドレス)をpushし元のデータを消去
                    vec!["@2", "D=M"],
                    push_d.clone(),
                    // THIS(RAM[3]: thisのベースアドレス)をpushし元のデータを消去
                    vec!["@3", "D=M"],
                    push_d.clone(),
                    // THAT(RAM[4]: thatのベースアドレス)をpushし元のデータを消去
                    vec!["@4", "D=M"],
                    push_d.clone(),
                    // ARGを`SP-5-nArgs`に変更する
                    vec!["@SP", "D=M", "@2", "M=D"],
                    vec!["M=M-1"; (5 + vars_length) as usize],
                    // LCLをSPの値に変更する
                    vec!["@SP", "D=M", "@1", "M=D"],
                    // 呼び出される側に制御を移す
                    vec![format!("@{}", go_to_address_label).as_str(), "0;JMP"],
                    // リターンアドレスラベルを挿入
                    vec![format!("({})", return_address_label).as_str()],
                    // MEMO: 呼び出される側のthis, that, pointer, tempを明示的に初期化する必要ってある？
                ]
                .concat()
                .into_iter()
                .map(|c| c.to_string())
                .collect();
                (commands, false, true, None)
            }
            Command::Function(function_name, vars_length) => {
                let go_to_address_label = format!(
                    "{}.{}", // 呼び出し先関数のアドレスを宣言し、Dに格納
                    file_name, function_name
                );
                let init_local_segment = {
                    let mut result: Vec<&str> = vec![];
                    for _ in 0..*vars_length {
                        result = [result, vec!["@0", "D=A"], push_d.clone()].concat();
                    }
                    result
                };
                let commands = [
                    vec![format!("// {:?}", self).as_str()],
                    // 開始ラベルを挿入する
                    vec![format!("({})", go_to_address_label).as_str()],
                    // ローカルセグメントを初期化する(必要な数だけ0うめする）
                    init_local_segment,
                ]
                .concat()
                .into_iter()
                .map(|c| c.to_string())
                .collect();
                (commands, false, false, Some(function_name.to_string()))
            }
            Command::Return => {
                let commands = [
                    vec![format!("// {:?}", self).as_str()],
                    // LCLを一時変数(R13)に保存(以降のコメントではR13のことをframeと呼ぶ)
                    vec!["@1", "D=M", "@13", "M=D"],
                    // リターンアドレス*(frame-5)を一時変数(R14)に保存(以降はretAddrと呼ぶ)
                    [vec!["@13", "A=M"], vec!["A=A-1"; 5], vec!["D=M", "@14", "M=D"]].concat(),
                    // 戻り値(スタックの先頭にあるはず)をRAM[ARG]にpopする
                    vec!["@SP", "A=M-1", "D=M", "@2", "A=M", "M=D"],
                    // SPをARG+1の位置に設定する
                    vec!["@2", "D=M+1", "@SP", "M=D"],
                    // 呼び出し側のTHATを復元する(*(frame-1))
                    vec!["@13", "A=M-1", "D=M", "@4", "M=D"],
                    // 呼び出し側のTHISを復元する(*(frame-2))
                    [vec!["@13", "A=M"], vec!["A=A-1"; 2], vec!["D=M", "@3", "M=D"]].concat(),
                    // 呼び出し側のARGを復元する
                    [vec!["@13", "A=M"], vec!["A=A-1"; 3], vec!["D=M", "@2", "M=D"]].concat(),
                    // 呼び出し側のLCLを復元する
                    [vec!["@13", "A=M"], vec!["A=A-1"; 4], vec!["D=M", "@1", "M=D"]].concat(),
                    // リターンアドレスに移動する
                    vec!["@R14", "A=M", "0;JMP"],
                ]
                .concat()
                .into_iter()
                .map(|c| c.to_string())
                .collect();
                (commands, false, false, None)
            }
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
                    vec![format!("// argument {}", index).as_str(), "@2", "A=M"],
                    vec!["A=A+1"; *index as usize],
                ]
                .concat()
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
            }
            Self::Local(index) => [vec!["@1", "A=M"], vec!["A=A+1"; *index as usize]]
                .concat()
                .into_iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>(),
            Self::Static(index) => vec![format!("@{}.{}", file_name, index)],
            Self::Constant(value) => [format!("@{}", value).as_str()]
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>(),
            Self::This(index) => [vec!["@3", "A=M"], vec!["A=A+1"; *index as usize]]
                .concat()
                .into_iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>(),
            Self::That(index) => [vec!["@4", "A=M"], vec!["A=A+1"; *index as usize]]
                .concat()
                .into_iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>(),
            Self::Pointer(index) => {
                if *index == 0 {
                    vec!["@3".to_string()]
                } else if *index == 1 {
                    vec!["@4".to_string()]
                } else {
                    vec![]
                }
            }
            Self::Temp(index) => {
                if 7 < *index {
                    vec![]
                } else {
                    [vec!["@5"], vec!["A=A+1"; *index as usize]]
                        .concat()
                        .into_iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<String>>()
                }
            }
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
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_vm_program_new() {
        // push, pop, add
        assert_eq!(
            VMProgram::new(
                "foo.vm".to_string(),
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
                file_name: "foo.vm".to_string(),
                return_address_id: 0,
                current_function_name: String::new(),
            }
        );

        // label, goto, if-goto, function, call, return
        assert_eq!(
            VMProgram::new(
                "foo.vm".to_string(),
                r#"
label LOOP
goto LOOP
if-goto LOOP
function f_name 2
call f_name 2
return
                "#
                .to_string(),
            ),
            VMProgram {
                commands: vec![
                    Command::Label("LOOP".to_string()),
                    Command::GoTo("LOOP".to_string()),
                    Command::IfGoTo("LOOP".to_string()),
                    Command::Function("f_name".to_string(), 2),
                    Command::Call("f_name".to_string(), 2),
                    Command::Return,
                ],
                label_id: 0,
                file_name: "foo.vm".to_string(),
                return_address_id: 0,
                current_function_name: String::new(),
            }
        );
    }
}
