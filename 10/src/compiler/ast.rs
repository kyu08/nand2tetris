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
    // parse結果を返す
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
    // parse結果と次のトークンの読み出し位置を返す
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
impl Type {
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Option<Self>, usize) {
        match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Int)) => (Some(Type::Int), index + 1),
            Some(token::Token::Key(token::Keyword::Char)) => (Some(Type::Char), index + 1),
            Some(token::Token::Key(token::Keyword::Boolean)) => (Some(Type::Boolean), index + 1),
            Some(token::Token::Identifier(i)) => (Some(Type::ClassName(i.0.clone())), index + 1),
            _ => (None, index),
        }
    }
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
impl ParameterList {
    // パターンメモ
    // ``: 引数なし
    // `type var_name, type var_name, ..., type var_name`: n個の引数
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Self, usize) {
        let mut index = index;
        let mut param_list = vec![];
        while let (Some(type_), returned_index) = Type::new(tokens, index) {
            index = returned_index;

            let var_name = match tokens.get(index) {
                Some(token::Token::Identifier(i)) => {
                    index += 1;
                    VarName(token::Identifier(i.clone().0))
                }
                _ => panic!("{}", invalid_token(tokens, index)),
            };

            param_list.push((type_, var_name));

            // `,`があるときだけ継続
            match tokens.get(index) {
                Some(token::Token::Sym(token::Symbol::Comma)) => {
                    index += 1;
                }
                _ => break,
            }
        }

        (Self(param_list), index)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SubroutineBody {
    var_dec: Vec<VarDec>,
    statements: Vec<Statement>,
}
impl SubroutineBody {
    // fn new(tokens: &Vec<token::Token>, index: usize) -> (Self, usize) {
    //     // TODO: {があることを確認
    //     // TODO: var_decをn個パース
    //     // TODO: statementsをパース
    //     // TODO: }があることを確認
    //     (Self { var_dec, statements }, index)
    // }
}

#[derive(Debug, PartialEq, Eq)]
struct VarDec {
    type_: Type,
    var_name: Vec<VarName>,
}
impl VarDec {
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Self, usize) {
        let mut index = index;
        let mut var_name = vec![];
        let index = match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Var)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        let (type_, index) = match Type::new(tokens, index) {
            (Some(t), i) => (t, i),
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        let mut index = match tokens.get(index) {
            Some(token::Token::Identifier(token::Identifier(id))) => {
                var_name.push(VarName(token::Identifier(id.clone())));
                index + 1
            }
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        while let Some(token::Token::Sym(token::Symbol::Comma)) = tokens.get(index) {
            index += 1;

            let var_name_hoge_should_rename = match tokens.get(index) {
                Some(token::Token::Identifier(i)) => {
                    index += 1;
                    VarName(token::Identifier(i.clone().0))
                }
                _ => panic!("{}", invalid_token(tokens, index)),
            };

            var_name.push(var_name_hoge_should_rename);
        }

        // 最後にセミコロンがあることをチェック
        match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::SemiColon)) => index += 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        (Self { type_, var_name }, index)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq)]
struct SubroutineName(token::Identifier);
#[derive(Debug, PartialEq, Eq)]
struct VarName(token::Identifier);

/*
 * 文
 */
#[derive(Debug, PartialEq, Eq)]
struct Statements(Vec<Statement>);
impl Statements {
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Self, usize) {
        let mut statements = vec![];
        let mut index = index;
        while let (Some(s), returned_index) = Statement::new(tokens, index) {
            statements.push(s);
            index = returned_index;
        }

        (Statements(statements), index)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Statement {
    Let(LetStatement),
    If(IfStatement),
    While(WhileStatement),
    Do(DoStatement),
    Return(ReturnStatement),
}
impl Statement {
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Option<Self>, usize) {
        match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Let)) => {
                let (l, i) = LetStatement::new(tokens, index);
                (Some(Self::Let(l)), i)
            }
            Some(token::Token::Key(token::Keyword::If)) => {
                let (l, i) = IfStatement::new(tokens, index);
                (Some(Self::If(l)), i)
            }
            // TODO: 随時ほかのvariantを実装する
            _ => (None, index),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LetStatement {
    var_name: VarName,
    index: Option<Expression>,
    right_hand_side: Expression,
}
impl LetStatement {
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Self, usize) {
        let index = match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Let)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (var_name, index) = match tokens.get(index) {
            Some(token::Token::Identifier(i)) => (VarName(i.clone()), index + 1),
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        // TODO: ('[]' expression ']')? のチェックを行う
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::Equal)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (right_hand_side, index) = match Expression::new(tokens, index) {
            (Some(e), index) => (e, index),
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::SemiColon)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        (
            Self {
                var_name,
                index: None,
                right_hand_side,
            },
            index,
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
struct IfStatement {
    condition: Expression,
    positive_case_body: Statements,
    negative_case_body: Option<Statements>,
}
impl IfStatement {
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Self, usize) {
        let index = match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::If)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::LeftParen)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (condition, index) = match Expression::new(tokens, index) {
            (Some(e), index) => (e, index),
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::RightParen)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::LeftBrace)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (positive_case_body, index) = Statements::new(tokens, index);
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::RightBrace)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        match tokens.get(index) {
            // else節があるパターン
            Some(token::Token::Key(token::Keyword::Else)) => {
                let index = index + 1; // elseの分を前に進める

                let index = match tokens.get(index) {
                    Some(token::Token::Sym(token::Symbol::LeftBrace)) => index + 1,
                    _ => panic!("{}", invalid_token(tokens, index)),
                };
                let (negative_case_body, index) = {
                    let (statements, index) = Statements::new(tokens, index);
                    if statements.0.is_empty() {
                        (None, index)
                    } else {
                        (Some(statements), index)
                    }
                };
                let index = match tokens.get(index) {
                    Some(token::Token::Sym(token::Symbol::RightBrace)) => index + 1,
                    _ => panic!("{}", invalid_token(tokens, index)),
                };
                (
                    Self {
                        condition,
                        positive_case_body,
                        negative_case_body,
                    },
                    index,
                )
            }
            // else節がないパターン
            _ => (
                Self {
                    condition,
                    positive_case_body,
                    negative_case_body: None,
                },
                index,
            ),
        }
    }
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
    // 循環参照になってしまうのでBoxでくるんでいる
    term: Box<Term>,
    // TODO: あとでコメントインする
    // op_term: Vec<(Op, Term)>,
}
impl Expression {
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Option<Self>, usize) {
        match Term::new(tokens, index) {
            (Some(t), i) => (Some(Self { term: Box::new(t) }), i),
            _ => (None, index),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Term {
    IntegerConstant(token::IntegerConstant),
    StringConstant(token::StringConstant),
    KeyWordConstant(KeyWordConstant),
    VarName(VarName),
    Expression(Expression),
    SubroutineCall(SubroutineCall),
    // TODO: あとで拡張する
}
impl Term {
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Option<Self>, usize) {
        match tokens.get(index) {
            Some(token::Token::IntegerConstant(i)) => (Some(Term::IntegerConstant(i.clone())), index + 1),
            Some(token::Token::StringConstant(s)) => (Some(Term::StringConstant(s.clone())), index + 1),
            Some(token::Token::Key(token::Keyword::True)) => {
                (Some(Term::KeyWordConstant(KeyWordConstant::True)), index + 1)
            }
            Some(token::Token::Key(token::Keyword::False)) => {
                (Some(Term::KeyWordConstant(KeyWordConstant::False)), index + 1)
            }
            Some(token::Token::Key(token::Keyword::Null)) => {
                (Some(Term::KeyWordConstant(KeyWordConstant::Null)), index + 1)
            }
            Some(token::Token::Key(token::Keyword::This)) => {
                (Some(Term::KeyWordConstant(KeyWordConstant::This)), index + 1)
            }
            Some(token::Token::Identifier(i)) => (Some(Term::VarName(VarName(i.clone()))), index + 1),
            // TODO: SubroutineCallはあとで実装する
            _ => {
                let index = match tokens.get(index) {
                    Some(token::Token::Sym(token::Symbol::LeftParen)) => index + 1,
                    _ => return (None, index), // ExpressionListから見るとtermが空のパターンもあるのでpanicしてはならない
                };
                let (expression, index) = Expression::new(tokens, index);
                let index = match tokens.get(index) {
                    Some(token::Token::Sym(token::Symbol::RightParen)) => index + 1,
                    _ => return (None, index), // ExpressionListから見るとtermが空のパターンもあるのでpanicしてはならない
                };
                (Some(Term::Expression(expression.unwrap())), index)
            }
        }
    }
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
    receiver: Option<Receiver>,
    name: SubroutineName,
    arguments: ExpressionList,
}
impl SubroutineCall {
    fn new(tokens: &Vec<token::Token>, index: usize, class_name: &ClassName) -> (Self, usize) {
        // まずindex + 1を見て`.`があるか調べる
        let exist_receiver = matches!(tokens.get(index + 1), Some(token::Token::Sym(token::Symbol::Dot)));

        if exist_receiver {
            let (receiver, index) = match tokens.get(index) {
                Some(token::Token::Identifier(i)) => {
                    // 自クラスの名前だったらReceiver::ClassName
                    // それ以外の場合はReceiver::VarName
                    if i.0 == class_name.0 .0 {
                        (Some(Receiver::ClassName(class_name.clone())), index + 1)
                    } else {
                        (Some(Receiver::VarName(VarName(i.clone()))), index + 1)
                    }
                }
                _ => panic!("{}", invalid_token(tokens, index)),
            };
            // index番目に`.`があることは確認済みなのでindex + 1を見る
            let (name, index) = match tokens.get(index + 1) {
                Some(token::Token::Identifier(i)) => (SubroutineName(i.clone()), index + 2),
                _ => panic!("{}", invalid_token(tokens, index + 1)),
            };
            let index = match tokens.get(index) {
                Some(token::Token::Sym(token::Symbol::LeftParen)) => index + 1,
                _ => panic!("{}", invalid_token(tokens, index)),
            };
            let (arguments, index) = match ExpressionList::new(tokens, index) {
                (Some(el), returned_index) => (el, returned_index),
                _ => (ExpressionList(vec![]), index),
            };
            let index = match tokens.get(index) {
                Some(token::Token::Sym(token::Symbol::RightParen)) => index + 1,
                _ => panic!("{}", invalid_token(tokens, index)),
            };

            (
                Self {
                    receiver,
                    name,
                    arguments,
                },
                index,
            )
        } else {
            let (subroutine_name, index) = match tokens.get(index) {
                Some(token::Token::Identifier(i)) => (SubroutineName(i.clone()), index + 1),
                _ => panic!("{}", invalid_token(tokens, index)),
            };
            let index = match tokens.get(index) {
                Some(token::Token::Sym(token::Symbol::LeftParen)) => index + 1,
                _ => panic!("{}", invalid_token(tokens, index)),
            };
            let (arguments, index) = match ExpressionList::new(tokens, index) {
                (Some(el), index) => (el, index),
                _ => (ExpressionList(vec![]), index),
            };
            let index = match tokens.get(index) {
                Some(token::Token::Sym(token::Symbol::RightParen)) => index + 1,
                _ => panic!("{}", invalid_token(tokens, index)),
            };

            (
                Self {
                    receiver: None,
                    name: subroutine_name,
                    arguments,
                },
                index,
            )
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Receiver {
    ClassName(ClassName),
    VarName(VarName),
}

#[derive(Debug, PartialEq, Eq)]
struct ExpressionList(Vec<Expression>);
impl ExpressionList {
    // 無
    // expression
    // expression, expression, ..., expression
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Option<Self>, usize) {
        let (expression, index) = match Expression::new(tokens, index) {
            (Some(expression), returned_index) => (expression, returned_index),
            _ => return (None, index),
        };

        let mut index = index;
        let mut expression_list = vec![expression];
        while let Some(token::Token::Sym(token::Symbol::Comma)) = tokens.get(index) {
            index += 1;
            match Expression::new(tokens, index) {
                (Some(expression), returned_index) => {
                    index = returned_index;
                    expression_list.push(expression);
                }
                _ => return (Some(Self(expression_list)), index),
            }
        }

        (Some(Self(expression_list)), index)
    }
}

// TODO: Op
// TODO: UnaryOp

fn invalid_token(tokens: &Vec<token::Token>, index: usize) -> String {
    format!("invalid token {:?}[@{}]", tokens, index)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use token::IntegerConstant;

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
        // TODO: 実装ができたらコメントインする
        // let input = SubroutineDec::new(
        //     &vec![
        //         token::Token::Key(token::Keyword::Field),
        //         token::Token::Identifier(token::Identifier("Square".to_string())),
        //         token::Token::Identifier(token::Identifier("square".to_string())),
        //         token::Token::Sym(token::Symbol::SemiColon),
        //         token::Token::Key(token::Keyword::Field),
        //         token::Token::Key(token::Keyword::Int),
        //         token::Token::Identifier(token::Identifier("direction".to_string())),
        //         token::Token::Sym(token::Symbol::SemiColon),
        //         token::Token::Key(token::Keyword::Constructor),
        //         token::Token::Identifier(token::Identifier("SquareGame".to_string())),
        //         token::Token::Identifier(token::Identifier("new".to_string())),
        //         token::Token::Sym(token::Symbol::LeftParen),
        //         token::Token::Sym(token::Symbol::RightParen),
        //         token::Token::Sym(token::Symbol::LeftBrace),
        //         token::Token::Key(token::Keyword::Let),
        //         token::Token::Identifier(token::Identifier("square".to_string())),
        //         token::Token::Sym(token::Symbol::Equal),
        //         token::Token::Identifier(token::Identifier("square".to_string())),
        //         token::Token::Sym(token::Symbol::SemiColon),
        //         token::Token::Key(token::Keyword::Let),
        //         token::Token::Identifier(token::Identifier("direction".to_string())),
        //         token::Token::Sym(token::Symbol::Equal),
        //         token::Token::Identifier(token::Identifier("direction".to_string())),
        //         token::Token::Sym(token::Symbol::SemiColon),
        //         token::Token::Key(token::Keyword::Return),
        //         token::Token::Identifier(token::Identifier("square".to_string())),
        //         token::Token::Sym(token::Symbol::SemiColon),
        //         token::Token::Sym(token::Symbol::RightBrace),
        //         //  ひとまず1つめの宣言まで
        //     ],
        //     0,
        // );
        // let expected = (
        //     vec![SubroutineDec {
        //         kind: SubroutineDecKind::Constructor,
        //         type_: SubroutineDecType::Type_(Type::ClassName("SquareGame".to_string())),
        //         subroutine_name: "new".to_string(),
        //         parameter_list: vec![],
        //         body: SubroutineBody {
        //             var_dec: vec![],
        //             statements: vec![],
        //         },
        //     }],
        //     13,
        // );
        // assert_eq!(input, expected);
    }

    #[test]
    fn test_type_new() {
        let input = Type::new(&vec![token::Token::Key(token::Keyword::Int)], 0);
        let expected = (Some(Type::Int), 1);
        assert_eq!(input, expected);

        let input = Type::new(&vec![token::Token::Key(token::Keyword::Char)], 0);
        let expected = (Some(Type::Char), 1);
        assert_eq!(input, expected);

        let input = Type::new(&vec![token::Token::Key(token::Keyword::Boolean)], 0);
        let expected = (Some(Type::Boolean), 1);
        assert_eq!(input, expected);

        let input = Type::new(&vec![token::Token::Identifier(token::Identifier("main".to_string()))], 0);
        let expected = (Some(Type::ClassName("main".to_string())), 1);
        assert_eq!(input, expected);
    }

    #[test]
    fn test_var_dec_new() {
        // var MyType foo;
        let input = VarDec::new(
            &vec![
                token::Token::Key(token::Keyword::Var),
                token::Token::Identifier(token::Identifier("MyType".to_string())),
                token::Token::Identifier(token::Identifier("foo".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
        );
        let expected = (
            VarDec {
                type_: Type::ClassName("MyType".to_string()),
                var_name: vec![VarName(token::Identifier("foo".to_string()))],
            },
            4,
        );
        assert_eq!(input, expected);

        // var int x, y, z;
        let input = VarDec::new(
            &vec![
                token::Token::Key(token::Keyword::Var),
                token::Token::Key(token::Keyword::Int),
                token::Token::Identifier(token::Identifier("x".to_string())),
                token::Token::Sym(token::Symbol::Comma),
                token::Token::Identifier(token::Identifier("y".to_string())),
                token::Token::Sym(token::Symbol::Comma),
                token::Token::Identifier(token::Identifier("z".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
        );
        let expected = (
            VarDec {
                type_: Type::Int,
                var_name: vec![
                    VarName(token::Identifier("x".to_string())),
                    VarName(token::Identifier("y".to_string())),
                    VarName(token::Identifier("z".to_string())),
                ],
            },
            8,
        );
        assert_eq!(input, expected);
    }

    #[test]
    fn test_parameter_list_new() {
        /*
          int x, char y
        */
        let input = ParameterList::new(
            &vec![
                token::Token::Key(token::Keyword::Int),
                token::Token::Identifier(token::Identifier("x".to_string())),
                token::Token::Sym(token::Symbol::Comma),
                token::Token::Key(token::Keyword::Char),
                token::Token::Identifier(token::Identifier("y".to_string())),
            ],
            0,
        );
        let expected = (
            ParameterList(vec![
                (Type::Int, VarName(token::Identifier("x".to_string()))),
                (Type::Char, VarName(token::Identifier("y".to_string()))),
            ]),
            5,
        );
        assert_eq!(input, expected);

        /*
            (引数なし)
        */
        let input = ParameterList::new(&vec![], 0);
        let expected = (ParameterList(vec![]), 0);
        assert_eq!(input, expected);
    }

    #[test]
    fn test_expression_list_new() {
        /*
            true, false
        */
        let input = ExpressionList::new(
            &vec![
                token::Token::Key(token::Keyword::True),
                token::Token::Sym(token::Symbol::Comma),
                token::Token::Key(token::Keyword::False),
            ],
            0,
        );
        let expected = (
            Some(ExpressionList(vec![
                Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                },
                Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::False)),
                },
            ])),
            3,
        );
        assert_eq!(input, expected);

        /*
            1
        */
        let input = ExpressionList::new(&vec![token::Token::IntegerConstant(token::IntegerConstant(1))], 0);
        let expected = (
            Some(ExpressionList(vec![Expression {
                term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
            }])),
            1,
        );
        assert_eq!(input, expected);

        /*
            (引数なし)
        */
        let input = ExpressionList::new(&vec![], 0);
        let expected = (None, 0);
        assert_eq!(input, expected);
    }

    #[test]
    fn test_subroutine_call_new() {
        /*
            Main.show(x, y)(レシーバがclass_name)
        */
        let input = SubroutineCall::new(
            &vec![
                token::Token::Identifier(token::Identifier("Main".to_string())),
                token::Token::Sym(token::Symbol::Dot),
                token::Token::Identifier(token::Identifier("show".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Identifier(token::Identifier("x".to_string())),
                token::Token::Sym(token::Symbol::Comma),
                token::Token::Identifier(token::Identifier("y".to_string())),
                token::Token::Sym(token::Symbol::RightParen),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            SubroutineCall {
                receiver: Some(Receiver::ClassName(ClassName(token::Identifier("Main".to_string())))),
                name: SubroutineName(token::Identifier("show".to_string())),
                arguments: ExpressionList(vec![
                    Expression {
                        term: Box::new(Term::VarName(VarName(token::Identifier("x".to_string())))),
                    },
                    Expression {
                        term: Box::new(Term::VarName(VarName(token::Identifier("y".to_string())))),
                    },
                ]),
            },
            8,
        );
        assert_eq!(input, expected);

        /*
            person.show()(レシーバがvar_name)
        */
        let input = SubroutineCall::new(
            &vec![
                token::Token::Identifier(token::Identifier("person".to_string())),
                token::Token::Sym(token::Symbol::Dot),
                token::Token::Identifier(token::Identifier("show".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::RightParen),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            SubroutineCall {
                receiver: Some(Receiver::VarName(VarName(token::Identifier("person".to_string())))),
                name: SubroutineName(token::Identifier("show".to_string())),
                arguments: ExpressionList(vec![]),
            },
            5,
        );
        assert_eq!(input, expected);

        /*
            show(x, y)(レシーバなし)
        */
        let input = SubroutineCall::new(
            &vec![
                token::Token::Identifier(token::Identifier("show".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Identifier(token::Identifier("x".to_string())),
                token::Token::Sym(token::Symbol::Comma),
                token::Token::Identifier(token::Identifier("y".to_string())),
                token::Token::Sym(token::Symbol::RightParen),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            SubroutineCall {
                receiver: None,
                name: SubroutineName(token::Identifier("show".to_string())),
                arguments: ExpressionList(vec![
                    Expression {
                        term: Box::new(Term::VarName(VarName(token::Identifier("x".to_string())))),
                    },
                    Expression {
                        term: Box::new(Term::VarName(VarName(token::Identifier("y".to_string())))),
                    },
                ]),
            },
            6,
        );
        assert_eq!(input, expected);

        /*
            show()(レシーバ、引数なし)
        */
        let input = SubroutineCall::new(
            &vec![
                token::Token::Identifier(token::Identifier("show".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::RightParen),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            SubroutineCall {
                receiver: None,
                name: SubroutineName(token::Identifier("show".to_string())),
                arguments: ExpressionList(vec![]),
            },
            3,
        );
        assert_eq!(input, expected);
    }

    #[test]
    fn test_let_statement_new() {
        /*
            let foo = 1;
        */
        let input = LetStatement::new(
            &vec![
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("foo".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
        );
        let expected = (
            LetStatement {
                var_name: VarName(token::Identifier("foo".to_string())),
                index: None,
                right_hand_side: Expression {
                    term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                },
            },
            5,
        );
        assert_eq!(input, expected);
    }

    #[test]
    fn test_if_statement_new() {
        /*
            if (true) { let foo = 1; let bar = true; }
        */
        let input = IfStatement::new(
            &vec![
                token::Token::Key(token::Keyword::If),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Key(token::Keyword::True),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("foo".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("bar".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::Key(token::Keyword::True),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Sym(token::Symbol::RightBrace),
            ],
            0,
        );
        let expected = (
            IfStatement {
                condition: Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                },
                positive_case_body: Statements(vec![
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("foo".to_string())),
                        index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                        },
                    }),
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("bar".to_string())),
                        index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                        },
                    }),
                ]),
                negative_case_body: None,
            },
            16,
        );
        assert_eq!(input, expected);

        /*
            if (true) { let foo = 1; let bar = true; } else { let baz = 1; let qux = null; }
        */
        let input = IfStatement::new(
            &vec![
                token::Token::Key(token::Keyword::If),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Key(token::Keyword::True),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("foo".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("bar".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::Key(token::Keyword::True),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Sym(token::Symbol::RightBrace),
                token::Token::Key(token::Keyword::Else),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("baz".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("qux".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::Key(token::Keyword::Null),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Sym(token::Symbol::RightBrace),
            ],
            0,
        );
        let expected = (
            IfStatement {
                condition: Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                },
                positive_case_body: Statements(vec![
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("foo".to_string())),
                        index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                        },
                    }),
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("bar".to_string())),
                        index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                        },
                    }),
                ]),
                negative_case_body: Some(Statements(vec![
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("baz".to_string())),
                        index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                        },
                    }),
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("qux".to_string())),
                        index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::KeyWordConstant(KeyWordConstant::Null)),
                        },
                    }),
                ])),
            },
            29,
        );
        assert_eq!(input, expected);
    }

    #[test]
    fn test_statement_new() {
        // let foo = 1;
        let input = Statement::new(
            &vec![
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("foo".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
        );
        let expected = (
            Some(Statement::Let(LetStatement {
                var_name: VarName(token::Identifier("foo".to_string())),
                index: None,
                right_hand_side: Expression {
                    term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                },
            })),
            5,
        );
        assert_eq!(input, expected);

        // if (true) { let foo = 1; }
        let input = Statement::new(
            &vec![
                token::Token::Key(token::Keyword::If),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Key(token::Keyword::True),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("foo".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Sym(token::Symbol::RightBrace),
            ],
            0,
        );
        let expected = (
            Some(Statement::If(IfStatement {
                condition: Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                },
                positive_case_body: Statements(vec![Statement::Let(LetStatement {
                    var_name: VarName(token::Identifier("foo".to_string())),
                    index: None,
                    right_hand_side: Expression {
                        term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                    },
                })]),
                negative_case_body: None,
            })),
            11,
        );
        assert_eq!(input, expected);

        // TODO: while
        // TODO: do
        // TODO: return
    }
}
