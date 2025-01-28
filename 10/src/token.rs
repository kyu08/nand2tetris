#[derive(Debug, PartialEq, Eq)]
pub struct Tokens {
    tokens: Vec<Token>,
    parsing_token: String,
    is_string_const: bool,
}

impl Tokens {
    // トークナイズする関数
    pub fn new(source_code: String) -> Self {
        let mut tokens = Tokens {
            tokens: vec![],
            parsing_token: "".to_string(),
            is_string_const: false,
        };

        for char in source_code.chars() {
            if char == '"' {
                // 文字列の始点
                if !tokens.is_string_const {
                    tokens.toggle_is_string_const();
                } else {
                    // 文字列の終端
                    let token = Token::StringConstant(tokens.parsing_token.clone());
                    tokens.push_token(token);
                    tokens.toggle_is_string_const();
                }
                continue;
            }

            if let Some(c) = Symbol::new(char) {
                if !tokens.is_string_const {
                    if !tokens.parsing_token.is_empty() {
                        let token = Self::parse_as_keyword_or_identifier(tokens.parsing_token.clone());
                        tokens.push_token(token);
                    }
                    tokens.tokens.push(Token::Sym(c));
                    continue;
                }
            }

            if char.is_whitespace() {
                if tokens.is_string_const {
                    tokens.parsing_token += &char.to_string();
                    continue;
                }
                if !tokens.parsing_token.is_empty() {
                    let token = Self::parse_as_keyword_or_identifier(tokens.parsing_token.clone());
                    tokens.push_token(token);
                    continue;
                }
                continue;
            }

            tokens.parsing_token += &char.to_string();
        }

        tokens
    }

    fn toggle_is_string_const(&mut self) {
        self.is_string_const = !self.is_string_const;
    }

    pub fn to_xml(&self) -> String {
        // TODO: インデントをどうするか問題をあとで考える
        let result: String = "".to_string();
        result
    }

    fn parse_as_keyword_or_identifier(token: String) -> Token {
        if let Some(k) = Keyword::new(token.clone()) {
            Token::Key(k)
        } else {
            Token::Identifier(token)
        }
    }

    fn push_token(&mut self, token: Token) {
        self.tokens.push(token);
        self.parsing_token = String::new();
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
                ],
                parsing_token: String::new(),
                is_string_const: false,
            }
        );

        // program with comments
        // assert_eq!(
        //     Tokens::new(
        //         r#"
        //         class Main {
        //           /** api comment */
        //           function void main() {
        //             /*
        //              multi line comment
        //              */
        //             do Output.printString("hello. world!");
        //             return; // comment
        //           }
        //         }
        //                         "#
        //         .to_string()
        //     ),
        //     Tokens {
        //         tokens: vec![
        //             Token::Key(Keyword::Class),
        //             Token::Identifier("Main".to_string()),
        //             Token::Sym(Symbol::LeftBrace),
        //             Token::Key(Keyword::Function),
        //             Token::Key(Keyword::Void),
        //             Token::Identifier("main".to_string()),
        //             Token::Sym(Symbol::LeftParen),
        //             Token::Sym(Symbol::RightParen),
        //             Token::Sym(Symbol::LeftBrace),
        //             Token::Key(Keyword::Do),
        //             Token::Identifier("Output".to_string()),
        //             Token::Sym(Symbol::Dot),
        //             Token::Identifier("printString".to_string()),
        //             Token::Sym(Symbol::LeftParen),
        //             Token::StringConstant("hello. world!".to_string()),
        //             Token::Sym(Symbol::RightParen),
        //             Token::Sym(Symbol::SemiColon),
        //             Token::Key(Keyword::Return),
        //             Token::Sym(Symbol::SemiColon),
        //             Token::Sym(Symbol::RightBrace),
        //             Token::Sym(Symbol::RightBrace),
        //         ],
        //         parsing_token: String::new(),
        //         is_string_const: false,
        //     }
        // );
    }
}
