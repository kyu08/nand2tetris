/// hack機械語をparseした結果を保持する構造体
#[derive(Debug, PartialEq)]
pub struct ParseHackResult {
    lines: Vec<Line>,
}

const VARIABLE_ADDRESS_OFFSET: u32 = 16;

struct SymbolTable {
    symbol_table: std::collections::HashMap<String, u32>,
    next_variable_address: u32,
}

impl ParseHackResult {
    pub fn new(content: String) -> ParseHackResult {
        let mut symbol_table = Self::init_symbol_table(content.clone());

        let mut lines = vec![];
        for line in content.lines() {
            if let Some(line) = Line::new(line, &mut symbol_table) {
                lines.push(line);
            }
        }

        ParseHackResult { lines }
    }

    /// シンボルテーブルを作成する
    /// 1. 定義済みシンボルを登録
    /// 2. ファイル内容を走査しラベルのアドレスを登録
    fn init_symbol_table(content: String) -> SymbolTable {
        let predefined_symbols = std::collections::HashMap::from([
            ("R0".to_string(), 0_u32),
            ("R1".to_string(), 1_u32),
            ("R2".to_string(), 2_u32),
            ("R3".to_string(), 3_u32),
            ("R4".to_string(), 4_u32),
            ("R5".to_string(), 5_u32),
            ("R6".to_string(), 6_u32),
            ("R7".to_string(), 7_u32),
            ("R8".to_string(), 8_u32),
            ("R9".to_string(), 9_u32),
            ("R10".to_string(), 10_u32),
            ("R11".to_string(), 11_u32),
            ("R12".to_string(), 12_u32),
            ("R13".to_string(), 13_u32),
            ("R14".to_string(), 14_u32),
            ("R15".to_string(), 15_u32),
            ("SP".to_string(), 0_u32),
            ("LCL".to_string(), 1_u32),
            ("ARG".to_string(), 2_u32),
            ("THIS".to_string(), 3_u32),
            ("THAT".to_string(), 4_u32),
            ("SCREEN".to_string(), 16384_u32),
            ("KBD".to_string(), 24576_u32),
        ]);

        let mut label_map = Self::build_label_map(content);
        label_map.extend(predefined_symbols);

        SymbolTable {
            symbol_table: label_map,
            next_variable_address: VARIABLE_ADDRESS_OFFSET,
        }
    }

    fn build_label_map(content: String) -> std::collections::HashMap<String, u32> {
        let mut map = std::collections::HashMap::new();
        let mut program_counter = None;
        for line in content.lines() {
            let trimmed = line.trim();
            if Line::should_ignore(trimmed) {
                continue;
            }
            if Line::is_a_instruction(trimmed)
                || Line::is_c_instruction_dest_comp(trimmed)
                || Line::is_c_instruction_comp_jump(trimmed)
            {
                program_counter = match program_counter {
                    None => Some(0),
                    Some(c) => Some(c + 1),
                };
                continue;
            }

            let label = trimmed.trim_matches(['(', ')']);
            if !label.is_empty() {
                let count = match program_counter {
                    Some(c) => c + 1,
                    None => 0,
                };
                map.insert(label.to_string(), count);
            }
        }

        map
    }

    /// NOTE: 未定義の変数をシンボルテーブルに追加する際、symbol_table.values()を走査して16以降の
    /// 空いているアドレスに格納しようとするとlabel宣言のアドレスと混ざって意図しない挙動になる。
    /// そのため変数へのメモリ割当をどこまで行ったかを保持しておく必要がある
    fn add_variable_to_symbol_table(symbol_table: &mut SymbolTable, var_name: String) -> u32 {
        let current_variable_address = symbol_table.next_variable_address;
        symbol_table.symbol_table.insert(var_name, current_variable_address);
        symbol_table.next_variable_address += 1;

        current_variable_address
    }
}

impl std::fmt::Display for ParseHackResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.lines
                .iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// hack機械語の各行をparseした結果
#[derive(Debug, PartialEq)]
enum Line {
    AInstruction(u32), // address or 定数
    CInstruction(CInstruction),
}

impl Line {
    fn new(line: &str, symbol_table: &mut SymbolTable) -> Option<Line> {
        // `//`で始まる場合と空行の場合はNone
        let trimmed = line.trim();
        if Self::should_ignore(trimmed) {
            return None;
        }

        // A命令のparse
        if Self::is_a_instruction(trimmed) {
            // A命令には以下の3パターンが存在する。
            // - @定数(0~32767の範囲の10進数)
            // - @定義済みシンボル
            // - @変数
            // MEMO: Symbolの仕様
            // 文字 数字 _ . $ :からなる。ただし数字から始まることはできない
            return match trimmed[1..].parse::<u32>() {
                Ok(c) => Some(Line::AInstruction(c)),
                Err(_) => {
                    // u32としてparseできない => symbol
                    let symbol = &trimmed[1..];
                    match symbol_table.symbol_table.get(symbol) {
                        Some(address) => Some(Line::AInstruction(*address)),
                        None => {
                            // 未定義の変数をシンボルテーブルに追加する
                            let address =
                                ParseHackResult::add_variable_to_symbol_table(symbol_table, symbol.to_string());
                            Some(Line::AInstruction(address))
                        }
                    }
                }
            };
        };

        // C命令のparse
        if Self::is_c_instruction_dest_comp(trimmed) {
            let parts: Vec<&str> = trimmed.split("=").collect();
            let dest = Some(parts[0].to_string());
            let comp = parts[1].to_string();
            return Some(Line::CInstruction(CInstruction { dest, comp, jump: None }));
        }
        if Self::is_c_instruction_comp_jump(trimmed) {
            let parts: Vec<&str> = trimmed.split(";").collect();
            let comp = parts[0].to_string();
            let jump = Some(parts[1].to_string());
            return Some(Line::CInstruction(CInstruction { dest: None, comp, jump }));
        }

        None
    }

    fn should_ignore(line: &str) -> bool {
        line.is_empty() || line.starts_with("//")
    }

    fn is_a_instruction(line: &str) -> bool {
        line.starts_with("@") && line.len() > 1
    }

    fn is_c_instruction_dest_comp(line: &str) -> bool {
        line.contains("=")
    }

    fn is_c_instruction_comp_jump(line: &str) -> bool {
        line.contains(";")
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            Line::AInstruction(num) => {
                format!("0{:015b}", num)
            }
            Line::CInstruction(c_instruction) => {
                let (a, c) = match c_instruction.comp.as_str() {
                    "0" => ("0", "101010"),
                    "1" => ("0", "111111"),
                    "-1" => ("0", "111010"),
                    "D" => ("0", "001100"),
                    "A" => ("0", "110000"),
                    "M" => ("1", "110000"),
                    "!D" => ("0", "001101"),
                    "!A" => ("0", "110001"),
                    "!M" => ("1", "110001"),
                    "-D" => ("0", "001111"),
                    "-A" => ("0", "110011"),
                    "-M" => ("1", "110011"),
                    "D+1" => ("0", "011111"),
                    "A+1" => ("0", "110111"),
                    "M+1" => ("1", "110111"),
                    "D-1" => ("0", "001110"),
                    "A-1" => ("0", "110010"),
                    "M-1" => ("1", "110010"),
                    "D+A" => ("0", "000010"),
                    "D+M" => ("1", "000010"),
                    "D-A" => ("0", "010011"),
                    "D-M" => ("1", "010011"),
                    "A-D" => ("0", "000111"),
                    "M-D" => ("1", "000111"),
                    "D&A" => ("0", "000000"),
                    "D&M" => ("1", "000000"),
                    "D|A" => ("0", "010101"),
                    "D|M" => ("1", "010101"),
                    _ => ("0", "000000"),
                };
                let d = match &c_instruction.dest {
                    Some(dest) => match dest.as_str() {
                        "M" => "001",
                        "D" => "010",
                        "DM" | "MD" => "011",
                        "A" => "100",
                        "AM" | "MA" => "101",
                        "AD" | "DA" => "110",
                        "ADM" | "AMD" | "DAM" | "DMA" | "MAD" | "MDA" => "111",
                        _ => "000",
                    },
                    _ => "000",
                };
                let j = match &c_instruction.jump {
                    Some(jump) => match jump.as_str() {
                        "JGT" => "001",
                        "JEQ" => "010",
                        "JGE" => "011",
                        "JLT" => "100",
                        "JNE" => "101",
                        "JLE" => "110",
                        "JMP" => "111",
                        _ => "000",
                    },
                    _ => "000",
                };
                format!("111{}{}{}{}", a, c, d, j)
            }
        };
        write!(f, "{}", result)
    }
}

/// C命令はdest=comp;jumpの形式で表されるが実際のパターンとしてはdest=comp || comp;jump
/// 各フィールドは大文字でなければならない
#[derive(Debug, PartialEq)]
struct CInstruction {
    dest: Option<String>,
    comp: String, // compは必須
    jump: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_file() {
        assert_eq!(
            ParseHackResult::new(
                r#"
@10

M=D

// comment
                "#
                .to_string()
            ),
            ParseHackResult {
                lines: vec![
                    Line::AInstruction(10),
                    Line::CInstruction(CInstruction {
                        dest: Some("M".to_string()),
                        comp: "D".to_string(),
                        jump: None,
                    })
                ],
            }
        );
    }

    #[test]
    fn test_line_new() {
        let symbol_table = || SymbolTable {
            symbol_table: std::collections::HashMap::new(),
            next_variable_address: VARIABLE_ADDRESS_OFFSET,
        };
        assert_eq!(Line::new("", &mut symbol_table()), None);
        assert_eq!(Line::new("// comment", &mut symbol_table()), None);
        assert_eq!(Line::new("@12", &mut symbol_table()), Some(Line::AInstruction(12)));
        assert_eq!(Line::new("@x", &mut symbol_table()), Some(Line::AInstruction(16)));
        assert_eq!(
            Line::new("D=D-M", &mut symbol_table()),
            Some(Line::CInstruction(CInstruction {
                dest: Some("D".to_string()),
                comp: "D-M".to_string(),
                jump: None,
            }))
        );
        assert_eq!(
            Line::new("0;JMP", &mut symbol_table()),
            Some(Line::CInstruction(CInstruction {
                dest: None,
                comp: "0".to_string(),
                jump: Some("JMP".to_string()),
            }))
        );
    }

    #[test]
    fn test_build_label_map() {
        assert_eq!(
            ParseHackResult::build_label_map(
                r#"
@10
M=D
// comment
(x)
@SCREEN
(y)
@R0
                "#
                .to_string()
            ),
            std::collections::HashMap::from([("x".to_string(), 2), ("y".to_string(), 3)])
        );
    }
}
