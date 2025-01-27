#[derive(Debug, PartialEq, Eq)]
pub struct Tokens {
    tokens: Vec<Token>,
}

impl Tokens {
    pub fn new(source_code: String) -> Self {
        let mut tokens: Vec<Token> = vec![];
        // 解析途中のトークンのデータを保持する。トークンの終端が見つかったら空文字列にリセットする。
        let mut parsing_token = "".to_string();
        let mut is_string_const = false;

        for char in source_code.chars() {
            if char == '"' {
                // 文字列の始点
                if !is_string_const {
                    is_string_const = true;
                } else {
                    // 文字列の終端
                    tokens.push(Token::StringConstant(parsing_token));
                    parsing_token = String::new();
                    is_string_const = false;
                }
                continue;
            }

            if let Some(c) = Symbol::new(char) {
                if !is_string_const {
                    if !parsing_token.is_empty() {
                        tokens.push(Self::parse_token(parsing_token));
                        parsing_token = String::new();
                    }
                    tokens.push(Token::Sym(c));
                    continue;
                }
            }

            if char.is_whitespace() {
                if is_string_const {
                    parsing_token += &char.to_string();
                    continue;
                }
                if !parsing_token.is_empty() {
                    tokens.push(Self::parse_token(parsing_token));
                    parsing_token = String::new();
                    continue;
                }
                continue;
            }

            parsing_token += &char.to_string();
        }
        // EOFまでいったらparse結果をxmlとして出力する

        Self { tokens }
    }

    pub fn to_xml(&self) -> String {
        // TODO: インデントをどうするか問題をあとで考える
        let result: String = "".to_string();
        result
    }

    fn parse_token(token: String) -> Token {
        if let Some(k) = Keyword::new(token.clone()) {
            Token::Key(k)
        } else {
            Token::Identifier(token)
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum Token {
    Key(Keyword),
    Sym(Symbol),
    IntegerConstant(u32),
    StringConstant(String),
    Identifier(String),
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum Keyword {
    Class,
    Constructor,
    Function,
    Method,
    Field,
    Static,
    Var,
    Int,
    Char,
    Boolean,
    Void,
    True,
    False,
    Null,
    This,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
}

impl Keyword {
    pub fn new(c: String) -> Option<Self> {
        match c.as_str() {
            "class" => Some(Keyword::Class),
            "constructor" => Some(Keyword::Constructor),
            "function" => Some(Keyword::Function),
            "method" => Some(Keyword::Method),
            "field" => Some(Keyword::Field),
            "static" => Some(Keyword::Static),
            "var" => Some(Keyword::Var),
            "int" => Some(Keyword::Int),
            "char" => Some(Keyword::Char),
            "boolean" => Some(Keyword::Boolean),
            "void" => Some(Keyword::Void),
            "true" => Some(Keyword::True),
            "false" => Some(Keyword::False),
            "null" => Some(Keyword::Null),
            "this" => Some(Keyword::This),
            "let" => Some(Keyword::Let),
            "do" => Some(Keyword::Do),
            "if" => Some(Keyword::If),
            "else" => Some(Keyword::Else),
            "while" => Some(Keyword::While),
            "return" => Some(Keyword::Return),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Symbol {
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Dot,
    Comma,
    SemiColon,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Ampersand,
    Pipe,
    LessThan,
    MoreThan,
    Equal,
    Tilde,
}

impl Symbol {
    pub fn new(c: char) -> Option<Self> {
        match c {
            '{' => Some(Symbol::LeftBrace),
            '}' => Some(Symbol::RightBrace),
            '(' => Some(Symbol::LeftParen),
            ')' => Some(Symbol::RightParen),
            '[' => Some(Symbol::LeftBracket),
            ']' => Some(Symbol::RightBracket),
            '.' => Some(Symbol::Dot),
            ',' => Some(Symbol::Comma),
            ';' => Some(Symbol::SemiColon),
            '+' => Some(Symbol::Plus),
            '-' => Some(Symbol::Minus),
            '*' => Some(Symbol::Asterisk),
            '/' => Some(Symbol::Slash),
            '&' => Some(Symbol::Ampersand),
            '|' => Some(Symbol::Pipe),
            '<' => Some(Symbol::LessThan),
            '>' => Some(Symbol::MoreThan),
            '=' => Some(Symbol::Equal),
            '~' => Some(Symbol::Tilde),
            _ => None,
        }
    }
}

#[allow(dead_code)]
struct CompilationEngine {
    // Tokenizerからわたってきた字句解析結果
    token: Vec<Symbol>,
    // TODO: その他必要な状態を持たせる
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    #[test]
    fn test_tokenizer_new() {
        assert_eq!(
            Tokens::new(
                r#"
                class Main {
                  function void main() {
                    do Output.printString("hello. world!");
                    return;
                  }
                }
                                "#
                .to_string()
            ),
            Tokens {
                tokens: vec![
                    Token::Key(Keyword::Class),
                    Token::Identifier("Main".to_string()),
                    Token::Sym(Symbol::LeftBrace),
                    Token::Key(Keyword::Function),
                    Token::Key(Keyword::Void),
                    Token::Identifier("main".to_string()),
                    Token::Sym(Symbol::LeftParen),
                    Token::Sym(Symbol::RightParen),
                    Token::Sym(Symbol::LeftBrace),
                    Token::Key(Keyword::Do),
                    Token::Identifier("Output".to_string()),
                    Token::Sym(Symbol::Dot),
                    Token::Identifier("printString".to_string()),
                    Token::Sym(Symbol::LeftParen),
                    Token::StringConstant("hello. world!".to_string()),
                    Token::Sym(Symbol::RightParen),
                    Token::Sym(Symbol::SemiColon),
                    Token::Key(Keyword::Return),
                    Token::Sym(Symbol::SemiColon),
                    Token::Sym(Symbol::RightBrace),
                    Token::Sym(Symbol::RightBrace),
                ]
            }
        );
    }
}
