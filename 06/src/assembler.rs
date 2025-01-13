use std::fmt;

/// hack機械語をparseした結果を保持する構造体
#[derive(Debug, PartialEq)]
pub struct ParseHackResult {
    lines: Vec<Line>,
}

impl ParseHackResult {
    pub fn new(content: String) -> ParseHackResult {
        let mut lines = vec![];
        for line in content.lines() {
            if let Some(line) = Line::new(line) {
                lines.push(line);
            }
        }

        ParseHackResult { lines }
    }
}

impl fmt::Display for ParseHackResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
// NOTE: 先頭の空白文字と空行、コメント行は無視する
#[derive(Debug, PartialEq)]
enum Line {
    AInstruction(AInstruction),
    CInstruction(CInstruction),
    // L instruction(括弧で囲まれたシンボル)
    // LInstruction(String),
}

impl Line {
    fn new(line: &str) -> Option<Line> {
        // `//`で始まる場合と空行の場合はNone
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            return None;
        }

        // A命令のparse
        if trimmed.starts_with("@") && trimmed.len() > 1 {
            return Some(Line::AInstruction(AInstruction::Number(trimmed[1..].parse().unwrap())));
        }

        // C命令のparse
        if trimmed.contains("=") {
            let parts: Vec<&str> = trimmed.split("=").collect();
            let dest = Some(parts[0].to_string());
            let comp = parts[1].to_string();
            return Some(Line::CInstruction(CInstruction { dest, comp, jump: None }));
        }
        if trimmed.contains(";") {
            let parts: Vec<&str> = trimmed.split(";").collect();
            let comp = parts[0].to_string();
            let jump = Some(parts[1].to_string());
            return Some(Line::CInstruction(CInstruction { dest: None, comp, jump }));
        }

        None
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = match self {
            Line::AInstruction(AInstruction::Number(num)) => {
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

/// A命令には以下の3パターンが存在する。
/// - @定数(0~32767の範囲の10進数)
/// - @変数
/// - @定義済みシンボル
// MEMO: Symbolの仕様
// 文字 数字 _ . $ :からなる。ただし数字から始まることはできない
#[derive(Debug, PartialEq)]
enum AInstruction {
    Number(u16),
    // Variable(String),
    // DefinedSymbol(String),
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
                    Line::AInstruction(AInstruction::Number(10)),
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
    fn test_parse_line() {
        assert_eq!(Line::new(""), None);
        assert_eq!(Line::new("// comment"), None);
        assert_eq!(Line::new("@12"), Some(Line::AInstruction(AInstruction::Number(12))));
        // TODO: advancedの実装時にコメントインする
        // assert_eq!(parse_line("@x".to_string()), Some(Line::AInstruction(AInstruction::Variable("x".to_string()))));
        assert_eq!(
            Line::new("D=D-M"),
            Some(Line::CInstruction(CInstruction {
                dest: Some("D".to_string()),
                comp: "D-M".to_string(),
                jump: None,
            }))
        );
        assert_eq!(
            Line::new("0;JMP"),
            Some(Line::CInstruction(CInstruction {
                dest: None,
                comp: "0".to_string(),
                jump: Some("JMP".to_string()),
            }))
        );
    }
}
