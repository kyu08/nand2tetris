use crate::analyzer;

#[allow(dead_code)]
struct CompilationEngine {
    // Tokenizerからわたってきた字句解析の結果
    token: Vec<analyzer::token::Tokens>,
}

impl CompilationEngine {
    fn new(token: Vec<analyzer::token::Tokens>) -> Self {
        Self { token }
    }
}

struct Ast {
    classes: Vec<Class>,
}

/*
 * プログラムの構造
 */
struct Class {
    name: String,
    var_dec: Vec<ClassVarDec>,
    sub_routine_dec: Vec<SubRoutineDec>,
}

struct ClassVarDec {
    kind: ClassVarKind,
    type_: Type,
    var_names: Vec<VarName>,
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
struct ClassName(analyzer::token::Identifier);
struct SubRoutineName(analyzer::token::Identifier);
struct VarName(analyzer::token::Identifier);

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
