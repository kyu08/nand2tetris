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
    name: String,
    var_dec: Vec<ClassVarDec>,
    sub_routine_dec: Vec<SubRoutineDec>,
}
impl Class {
    // parse結果と次のトークン読み出し位置を返す
    fn new(tokens: &Vec<token::Token>, index: usize) -> Option<Self> {
        let (name, index) = ClassName::new(&tokens, index);
        let index = {
            if let Some(token::Token::Sym(token::Symbol::LeftBrace)) = tokens.get(index) {
                index + 1
            } else {
                panic!("{}", invalid_token(tokens, 0))
            }
        };

        let (var_dec, index) = ClassVarDec::new(tokens, index);
        None
    }
}

struct ClassVarDec {
    kind: ClassVarKind,
    type_: Type,
    var_names: Vec<VarName>,
}
impl ClassVarDec {
    fn new(tokens: &Vec<token::Token>, index: usize) -> (Vec<Self>, usize) {
        ここから(可能な数だけSelfへのparseを試みる。失敗した位置が次の要素の場所なのでそのindexを返す(+1して返さないように注意))
        (Self { kind, type_, var_names }, index + 1)
    }
}

enum ClassVarKind {
    Static,
    Field,
}

enum Type {
    Int,
    Char,
    Boolean,
    ClassName(String),
}

struct SubRoutineDec {
    kind: SubRoutineDecKind,
    type_: SubRoutineDecType,
    sub_routine_name: String,
    parameter_list: Vec<ParameterList>,
    body: SubRoutineBody,
}

enum SubRoutineDecKind {
    Constructor,
    Function,
    Method,
}

enum SubRoutineDecType {
    Void,
    Type_(Type),
}

struct ParameterList(Vec<(Type, VarName)>);

struct SubRoutineBody {
    var_dec: Vec<VarDec>,
    statements: Vec<Statement>,
}

struct VarDec {
    type_: Type,
    var_name: Vec<VarName>,
}
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

struct SubRoutineName(token::Identifier);
struct VarName(token::Identifier);

/*
 * 文
 */
struct Statements(Vec<Statement>);
enum Statement {
    Let(LetStatement),
    If(IfStatement),
    While(WhileStatement),
    Do(DoStatement),
    Return(ReturnStatement),
}

struct LetStatement {
    var_name: VarName,
    // index: Option<Expression>,
    right_hand_side: Expression,
}

struct IfStatement {
    condition: Expression,
    positive_case_body: Statements,
    negative_case_body: Option<Statements>,
}

struct WhileStatement {
    condition: Expression,
    body: Statements,
}

struct DoStatement(SubRoutineCall);

struct ReturnStatement(Option<Expression>);

/*
 * 式
 */
struct Expression {
    term: Term,
    // TODO: あとでコメントインする
    // op_term: Vec<(Op, Term)>,
}

enum Term {
    KeyWordConstant,
    // TODO: あとで拡張する
}

enum KeyWordConstant {
    True,
    False,
    Null,
    This,
}

struct SubRoutineCall {
    name: String,
    receiver: Option<Receiver>,
    arguments: Vec<ExpressionList>,
}

enum Receiver {
    ClassName(ClassName),
    VarName(VarName),
}

struct ExpressionList(Vec<Expression>);

// TODO: Op
// TODO: UnaryOp
//

fn invalid_token(tokens: &Vec<token::Token>, index: usize) -> String {
    format!("invalid token {:?}[@{}]", tokens, index)
}
