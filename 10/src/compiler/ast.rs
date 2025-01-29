use crate::analyzer::token;

#[allow(dead_code)]
struct CompilationEngine {
    // Tokenizerからわたってきた字句解析の結果
    token: Vec<token::Tokens>,
}

impl CompilationEngine {
    fn new(token: Vec<token::Tokens>) -> Self {
        Self { token }
    }
}

struct Ast {
    class: Class,
}

impl Ast {
    fn new(tokens: Vec<token::Token>) -> Self {
        // let index = 1;
        let class = match tokens.first() {
            Some(token::Token::Key(token::Keyword::Class)) => match Class::new(&tokens, 1) {
                Some(class) => class,
                _ => panic!("{}", invalid_token(&tokens, 1)),
            },
            Some(_) => panic!("{}", invalid_token(&tokens, 0)),
            None => panic!("{}", invalid_token(&tokens, 0)),
        };

        Self { class }
    }
}

/*
 * プログラムの構造
 */
struct Class {
    name: ClassName,
    var_dec: Vec<ClassVarDec>,
    subroutine_dec: Vec<SubroutineDec>,
}
impl Class {
    // parse結果と次のトークン読み出し位置を返す
    fn new(tokens: &Vec<token::Token>, index: usize) -> Option<Self> {
        // `class`は取得できているものと仮定
        let (name, index) = ClassName::new(tokens, index);
        let index = {
            if let Some(token::Token::Sym(token::Symbol::LeftBrace)) = tokens.get(index) {
                index + 1
            } else {
                panic!("{}", invalid_token(tokens, 0))
            }
        };

        let (var_dec, index) = ClassVarDec::new(tokens, index);
        let (subroutine_dec, index) = SubroutineDec::new(tokens, index);

        // 最後にセミコロンがあることをチェック
        match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::SemiColon)) => {}
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        Some(Class {
            name,
            var_dec,
            subroutine_dec: vec![],
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ClassVarDec {
    kind: ClassVarKind,
    type_: Type,
    var_names: Vec<VarName>,
}
impl ClassVarDec {
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Vec<Self>, usize) {
        let mut class_var_decs = vec![];
        let mut index = index;
        loop {
            let kind = match tokens.get(index) {
                Some(token::Token::Key(token::Keyword::Static)) => {
                    index += 1;
                    ClassVarKind::Static
                }
                Some(token::Token::Key(token::Keyword::Field)) => {
                    index += 1;
                    ClassVarKind::Field
                }
                _ => break,
            };

            let type_ = match tokens.get(index) {
                Some(token::Token::Key(token::Keyword::Int)) => {
                    index += 1;
                    Type::Int
                }
                Some(token::Token::Key(token::Keyword::Char)) => {
                    index += 1;
                    Type::Char
                }
                Some(token::Token::Key(token::Keyword::Boolean)) => {
                    index += 1;
                    Type::Boolean
                }
                Some(token::Token::Identifier(id)) => {
                    index += 1;
                    Type::ClassName(id.0.clone())
                }
                _ => panic!("{}", invalid_token(tokens, index)),
            };

            let var_name = match tokens.get(index) {
                Some(token::Token::Identifier(id)) => {
                    index += 1;
                    VarName(id.clone())
                }
                _ => panic!("{}", invalid_token(tokens, index)),
            };

            // このあとは `, varName`が任意の回数続く。
            let mut var_names = vec![var_name];
            while let Some(token::Token::Sym(token::Symbol::Comma)) = tokens.get(index) {
                index += 1; // , が取得できたのでindexを1進める

                match tokens.get(index) {
                    Some(token::Token::Identifier(id)) => {
                        index += 1;
                        var_names.push(VarName(id.clone()))
                    }
                    _ => panic!("{}", invalid_token(tokens, index)),
                }
            }

            // 最後にセミコロンがあることをチェック
            match tokens.get(index) {
                Some(token::Token::Sym(token::Symbol::SemiColon)) => index += 1,
                _ => panic!("{}", invalid_token(tokens, index)),
            };

            class_var_decs.push(Self { kind, type_, var_names });
        }

        // ここから(可能な数だけSelfへのparseを試みる。失敗した位置が次の要素の場所なのでそのindexを返す(+1して返さないように注意))
        (class_var_decs, index)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ClassVarKind {
    Static,
    Field,
}

#[derive(Debug, PartialEq, Eq)]
enum Type {
    Int,
    Char,
    Boolean,
    ClassName(String),
}

#[derive(Debug, PartialEq, Eq)]
struct SubroutineDec {
    kind: SubroutineDecKind,
    type_: SubroutineDecType,
    subroutine_name: String,
    parameter_list: Vec<ParameterList>,
    body: SubroutineBody,
}
impl SubroutineDec {
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Vec<Self>, usize) {
        // TODO: kindの判定
        // TODO: typeの判定
        // TODO: nameを取り出す
        // TODO: parameterList(引数リスト)のパース
        // TODO: subroutineBodyのパース
        (vec![], 0)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SubroutineDecKind {
    Constructor,
    Function,
    Method,
}

#[derive(Debug, PartialEq, Eq)]
enum SubroutineDecType {
    Void,
    Type_(Type),
}

#[derive(Debug, PartialEq, Eq)]
struct ParameterList(Vec<(Type, VarName)>);

#[derive(Debug, PartialEq, Eq)]
struct SubroutineBody {
    var_dec: Vec<VarDec>,
    statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Eq)]
struct VarDec {
    type_: Type,
    var_name: Vec<VarName>,
}
#[derive(Debug, PartialEq, Eq)]
struct ClassName(token::Identifier);
impl ClassName {
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Self, usize) {
        if let token::Token::Identifier(token::Identifier(s)) = tokens.get(index).unwrap() {
            (ClassName(token::Identifier(s.to_string())), index + 1)
        } else {
            panic!("{}", invalid_token(tokens, index))
        }
    }
}

struct SubroutineName(token::Identifier);
#[derive(Debug, PartialEq, Eq)]
struct VarName(token::Identifier);

/*
 * 文
 */
#[derive(Debug, PartialEq, Eq)]
struct Statements(Vec<Statement>);
#[derive(Debug, PartialEq, Eq)]
enum Statement {
    Let(LetStatement),
    If(IfStatement),
    While(WhileStatement),
    Do(DoStatement),
    Return(ReturnStatement),
}

#[derive(Debug, PartialEq, Eq)]
struct LetStatement {
    var_name: VarName,
    // index: Option<Expression>,
    right_hand_side: Expression,
}

#[derive(Debug, PartialEq, Eq)]
struct IfStatement {
    condition: Expression,
    positive_case_body: Statements,
    negative_case_body: Option<Statements>,
}

#[derive(Debug, PartialEq, Eq)]
struct WhileStatement {
    condition: Expression,
    body: Statements,
}

#[derive(Debug, PartialEq, Eq)]
struct DoStatement(SubroutineCall);

#[derive(Debug, PartialEq, Eq)]
struct ReturnStatement(Option<Expression>);

/*
 * 式
 */
#[derive(Debug, PartialEq, Eq)]
struct Expression {
    term: Term,
    // TODO: あとでコメントインする
    // op_term: Vec<(Op, Term)>,
}

#[derive(Debug, PartialEq, Eq)]
enum Term {
    KeyWordConstant,
    // TODO: あとで拡張する
}

#[derive(Debug, PartialEq, Eq)]
enum KeyWordConstant {
    True,
    False,
    Null,
    This,
}

#[derive(Debug, PartialEq, Eq)]
struct SubroutineCall {
    name: String,
    receiver: Option<Receiver>,
    arguments: Vec<ExpressionList>,
}

#[derive(Debug, PartialEq, Eq)]
enum Receiver {
    ClassName(ClassName),
    VarName(VarName),
}

#[derive(Debug, PartialEq, Eq)]
struct ExpressionList(Vec<Expression>);

// TODO: Op
// TODO: UnaryOp

fn invalid_token(tokens: &Vec<token::Token>, index: usize) -> String {
    format!("invalid token {:?}[@{}]", tokens, index)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_class_var_dec_new() {
        /*
        class SquareGame {
           field int x;
           static bool y, z;
        */
        let input = ClassVarDec::new(
            &vec![
                token::Token::Key(token::Keyword::Class),
                token::Token::Identifier(token::Identifier("Main".to_string())),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Key(token::Keyword::Field),
                token::Token::Key(token::Keyword::Int),
                token::Token::Identifier(token::Identifier("x".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Static),
                token::Token::Key(token::Keyword::Boolean),
                token::Token::Identifier(token::Identifier("y".to_string())),
                token::Token::Sym(token::Symbol::Comma),
                token::Token::Identifier(token::Identifier("z".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            3,
        );
        let expected = (
            vec![
                ClassVarDec {
                    kind: ClassVarKind::Field,
                    type_: Type::Int,
                    var_names: vec![VarName(token::Identifier("x".to_string()))],
                },
                ClassVarDec {
                    kind: ClassVarKind::Static,
                    type_: Type::Boolean,
                    var_names: vec![
                        VarName(token::Identifier("y".to_string())),
                        VarName(token::Identifier("z".to_string())),
                    ],
                },
            ],
            13,
        );
        assert_eq!(input, expected);
    }

    #[test]
    fn test_subroutine_dec_new() {
        /*
        class SquareGame {
            field Square square;
            field int direction;
            constructor SquareGame new() {
                let square = square;
                let direction = direction;
                return square;
             }

             method void dispose() {
                do square.dispose();
                do Memory.deAlloc(square);
                return;
             }

             method void moveSquare() {
                if (direction) { do square.moveUp(); }
                if (direction) { do square.moveDown(); }
                if (direction) { do square.moveLeft(); }
                if (direction) { do square.moveRight(); }
                do Sys.wait(direction);
                return;
             }
        }
        */
        let input = SubroutineDec::new(
            &vec![
                token::Token::Key(token::Keyword::Field),
                token::Token::Identifier(token::Identifier("Square".to_string())),
                token::Token::Identifier(token::Identifier("square".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Field),
                token::Token::Key(token::Keyword::Int),
                token::Token::Identifier(token::Identifier("direction".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Constructor),
                token::Token::Identifier(token::Identifier("SquareGame".to_string())),
                token::Token::Identifier(token::Identifier("new".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("square".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::Identifier(token::Identifier("square".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("direction".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::Identifier(token::Identifier("direction".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Return),
                token::Token::Identifier(token::Identifier("square".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Sym(token::Symbol::RightBrace),
                //  ひとまず1つめの宣言まで
            ],
            0,
        );
        let expected = (
            vec![SubroutineDec {
                kind: SubroutineDecKind::Constructor,
                type_: SubroutineDecType::Type_(Type::ClassName("SquareGame".to_string())),
                subroutine_name: "new".to_string(),
                parameter_list: vec![],
                body: SubroutineBody {
                    var_dec: vec![],
                    statements: vec![],
                },
            }],
            13,
        );
        assert_eq!(input, expected);
    }
}
