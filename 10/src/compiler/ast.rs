use crate::analyzer::token;

pub struct Ast {
    class: Class,
}

impl Ast {
    pub fn new(tokens: Vec<token::Token>) -> Self {
        let class = match tokens.first() {
            Some(token::Token::Key(token::Keyword::Class)) => match Class::new(&tokens, 0) {
                Some(class) => class,
                _ => panic!("{}", invalid_token(&tokens, 1)),
            },
            Some(_) => panic!("{}", invalid_token(&tokens, 0)),
            None => panic!("{}", invalid_token(&tokens, 0)),
        };

        Self { class }
    }

    pub fn to_xml(&self) -> String {
        self.class.to_string().join("\n")
    }
}

/*
 * プログラムの構造
 */
#[derive(Debug, PartialEq, Eq)]
struct Class {
    name: ClassName,
    var_dec: Vec<ClassVarDec>,
    subroutine_dec: Vec<SubroutineDec>,
}
impl Class {
    // parse結果を返す。ひとまずindexは返さない
    fn new(tokens: &[token::Token], index: usize) -> Option<Self> {
        let index = match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Class)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, 0)),
        };
        let (name, index) = ClassName::new(tokens, index);
        let index = {
            if let Some(token::Token::Sym(token::Symbol::LeftBrace)) = tokens.get(index) {
                index + 1
            } else {
                panic!("{}", invalid_token(tokens, 0))
            }
        };

        let (var_dec, index) = ClassVarDec::new(tokens, index);
        let mut subroutine_dec = vec![];
        let mut index = index;
        while let (Some(s), returned_index) = SubroutineDec::new(tokens, index, &name) {
            subroutine_dec.push(s);
            index = returned_index;
        }

        match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::RightBrace)) => {}
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        Some(Class {
            name,
            var_dec,
            subroutine_dec,
        })
    }

    pub fn to_string(&self) -> Vec<String> {
        let mut result = vec![];
        let (class_open, class_close) = get_xml_tag("class".to_string());
        result.push(class_open);
        result.push(to_xml_tag(token::Keyword::Class));
        result.push(self.name.0.to_string());
        result.push(to_xml_tag(token::Symbol::LeftBrace));
        for var_dec in &self.var_dec {
            result = [result, var_dec.to_string()].concat();
        }
        for subroutine in &self.subroutine_dec {
            result = [result, subroutine.to_string()].concat();
        }

        result.push(to_xml_tag(token::Symbol::RightBrace));
        result.push(class_close);
        result
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
    // FIXME: SubroutineDec::newは単数を返すのにこっちはVecを返すのは一貫性がないので直してもいいかもしれない
    fn new(tokens: &[token::Token], index: usize) -> (Vec<Self>, usize) {
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

        (class_var_decs, index)
    }

    fn to_string(&self) -> Vec<String> {
        let mut result = vec![];
        let (open, close) = get_xml_tag("classVarDec".to_string());
        result.push(open);
        result.push(self.kind.to_string());
        result.push(self.type_.to_string());
        for (index, n) in self.var_names.iter().enumerate() {
            if index != 0 {
                result.push(to_xml_tag(token::Symbol::Comma));
            }
            result.push(n.to_string());
        }
        if !&self.var_names.is_empty() {
            result.push(to_xml_tag(token::Symbol::SemiColon));
        }

        result.push(close);
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ClassVarKind {
    Static,
    Field,
}
impl ClassVarKind {
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        let (open, close) = get_xml_tag("keyword".to_string());
        format!("{} {} {}", open, format!("{:?}", self).to_lowercase(), close)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Type {
    Int,
    Char,
    Boolean,
    ClassName(String),
}
impl Type {
    fn new(tokens: &[token::Token], index: usize) -> (Option<Self>, usize) {
        match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Int)) => (Some(Type::Int), index + 1),
            Some(token::Token::Key(token::Keyword::Char)) => (Some(Type::Char), index + 1),
            Some(token::Token::Key(token::Keyword::Boolean)) => (Some(Type::Boolean), index + 1),
            Some(token::Token::Identifier(i)) => (Some(Type::ClassName(i.0.clone())), index + 1),
            _ => (None, index),
        }
    }
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        match self {
            Type::ClassName(c) => {
                let (open, close) = get_xml_tag("identifier".to_string());
                format!("{} {} {}", open, c, close)
            }
            _ => {
                let (open, close) = get_xml_tag("keyword".to_string());
                format!("{} {} {}", open, format!("{:?}", self).to_lowercase(), close)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SubroutineDec {
    kind: SubroutineDecKind,
    type_: SubroutineDecType,
    subroutine_name: token::Identifier,
    parameter_list: ParameterList,
    body: SubroutineBody,
}
impl SubroutineDec {
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Option<Self>, usize) {
        let (kind, index) = match SubroutineDecKind::new(tokens, index) {
            (Some(k), i) => (k, i),
            _ => return (None, index),
        };
        let (type_, index) = SubroutineDecType::new(tokens, index);
        let (subroutine_name, index) = match tokens.get(index) {
            Some(token::Token::Identifier(i)) => (i.clone(), index + 1),
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::LeftParen)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (parameter_list, index) = ParameterList::new(tokens, index);
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::RightParen)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (body, index) = SubroutineBody::new(tokens, index, class_name);

        (
            Some(Self {
                kind,
                type_,
                subroutine_name,
                parameter_list,
                body,
            }),
            index,
        )
    }
    fn to_string(&self) -> Vec<String> {
        let mut result = vec![];
        let (open, close) = get_xml_tag("subroutineDec".to_string());
        result.push(open);
        result.push(self.kind.to_string());
        result.push(self.type_.to_string());
        result.push(self.subroutine_name.to_string());
        result.push(to_xml_tag(token::Symbol::LeftParen));
        result = [result, self.parameter_list.to_string()].concat();
        result.push(to_xml_tag(token::Symbol::RightParen));
        result = [result, self.body.to_string()].concat();
        result.push(close);
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SubroutineDecKind {
    Constructor,
    Function,
    Method,
}
impl SubroutineDecKind {
    fn new(tokens: &[token::Token], index: usize) -> (Option<Self>, usize) {
        match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Constructor)) => (Some(SubroutineDecKind::Constructor), index + 1),
            Some(token::Token::Key(token::Keyword::Function)) => (Some(SubroutineDecKind::Function), index + 1),
            Some(token::Token::Key(token::Keyword::Method)) => (Some(SubroutineDecKind::Method), index + 1),
            _ => (None, index),
        }
    }
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        let (open, close) = get_xml_tag("keyword".to_string());
        format!("{} {} {}", open, format!("{:?}", self).to_lowercase(), close)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SubroutineDecType {
    Void,
    Type_(Type),
}
impl SubroutineDecType {
    fn new(tokens: &[token::Token], index: usize) -> (Self, usize) {
        match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Void)) => (SubroutineDecType::Void, index + 1),
            _ => match Type::new(tokens, index) {
                (Some(t), index) => (SubroutineDecType::Type_(t), index),
                _ => panic!("{}", invalid_token(tokens, index)),
            },
        }
    }
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        match self {
            SubroutineDecType::Void => {
                let (open, close) = get_xml_tag("keyword".to_string());
                format!("{} {} {}", open, format!("{:?}", self).to_lowercase(), close)
            }
            SubroutineDecType::Type_(t) => t.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParameterList(Vec<(Type, VarName)>);
impl ParameterList {
    // パターンメモ
    // ``: 引数なし
    // `type var_name, type var_name, ..., type var_name`: n個の引数
    fn new(tokens: &[token::Token], index: usize) -> (Self, usize) {
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
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> Vec<String> {
        let mut result = vec![];
        let (open, close) = get_xml_tag("parameterList".to_string());
        result.push(open);
        for (index, p) in self.0.iter().enumerate() {
            if index != 0 {
                result.push(to_xml_tag(token::Symbol::Comma));
            }
            result.push(p.0.to_string());
            result.push(p.1.to_string());
        }
        result.push(close);
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SubroutineBody {
    var_dec: Vec<VarDec>,
    statements: Statements,
}
impl SubroutineBody {
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Self, usize) {
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::LeftBrace)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        let mut var_dec = vec![];
        let mut index = index;
        while let (Some(got), returned_index) = VarDec::new(tokens, index) {
            var_dec.push(got);
            index = returned_index;
        }

        let (statements, index) = Statements::new(tokens, index, class_name);

        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::RightBrace)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        (Self { var_dec, statements }, index)
    }
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> Vec<String> {
        let mut result = vec![];
        let (open, close) = get_xml_tag("subroutineBody".to_string());
        result.push(open);
        result.push(to_xml_tag(token::Symbol::LeftBrace));
        for v in &self.var_dec {
            result = [result, v.to_string()].concat();
        }
        if !&self.statements.0.is_empty() {
            let (open, close) = get_xml_tag("statements".to_string());
            result.push(open);
            for s in &self.statements.0 {
                result = [result, s.to_string()].concat();
            }
            result.push(close);
        }
        result.push(to_xml_tag(token::Symbol::RightBrace));
        result.push(close);
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
struct VarDec {
    type_: Type,
    var_name: Vec<VarName>,
}
impl VarDec {
    fn new(tokens: &[token::Token], index: usize) -> (Option<Self>, usize) {
        let mut var_name = vec![];
        let index = match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Var)) => index + 1,
            _ => return (None, index),
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

        (Some(Self { type_, var_name }), index)
    }
    fn to_string(&self) -> Vec<String> {
        let mut result = vec![];
        let (open, close) = get_xml_tag("varDec".to_string());
        result.push(open);
        result.push(to_xml_tag(token::Keyword::Var));
        result.push(self.type_.to_string());
        for (index, n) in self.var_name.iter().enumerate() {
            if index != 0 {
                result.push(to_xml_tag(token::Symbol::Comma));
            }
            result.push(n.to_string());
        }
        if !&self.var_name.is_empty() {
            result.push(to_xml_tag(token::Symbol::SemiColon));
        }

        result.push(close);
        result
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct ClassName(token::Identifier);
impl ClassName {
    fn new(tokens: &[token::Token], index: usize) -> (Self, usize) {
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
impl VarName {
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

/*
 * 文
 */
#[derive(Debug, PartialEq, Eq)]
struct Statements(Vec<Statement>);
impl Statements {
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Self, usize) {
        let mut statements = vec![];
        let mut index = index;
        while let (Some(s), returned_index) = Statement::new(tokens, index, class_name) {
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
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Option<Self>, usize) {
        match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Let)) => {
                let (l, i) = LetStatement::new(tokens, index, class_name);
                (Some(Self::Let(l)), i)
            }
            Some(token::Token::Key(token::Keyword::If)) => {
                let (l, i) = IfStatement::new(tokens, index, class_name);
                (Some(Self::If(l)), i)
            }
            Some(token::Token::Key(token::Keyword::While)) => {
                let (l, i) = WhileStatement::new(tokens, index, class_name);
                (Some(Self::While(l)), i)
            }
            Some(token::Token::Key(token::Keyword::Do)) => {
                let (l, i) = DoStatement::new(tokens, index, class_name);
                (Some(Self::Do(l)), i)
            }
            Some(token::Token::Key(token::Keyword::Return)) => {
                let (l, i) = ReturnStatement::new(tokens, index, class_name);
                (Some(Self::Return(l)), i)
            }
            _ => (None, index),
        }
    }
    fn to_string(&self) -> Vec<String> {
        match self {
            Statement::Let(s) => s.to_string(),
            Statement::If(s) => s.to_string(),
            Statement::While(s) => s.to_string(),
            Statement::Do(s) => s.to_string(),
            Statement::Return(s) => s.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LetStatement {
    var_name: VarName,
    array_index: Option<Expression>,
    right_hand_side: Expression,
}
impl LetStatement {
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Self, usize) {
        let index = match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Let)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (var_name, mut index) = match tokens.get(index) {
            Some(token::Token::Identifier(i)) => (VarName(i.clone()), index + 1),
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        let mut array_index = None;
        if let Some(token::Token::Sym(token::Symbol::LeftBracket)) = tokens.get(index) {
            if let (Some(e), returned_index) = Expression::new(tokens, index + 1, class_name) {
                match tokens.get(returned_index) {
                    Some(token::Token::Sym(token::Symbol::RightBracket)) => {
                        array_index = Some(e);
                        index = returned_index + 1;
                    }
                    _ => panic!("{}", invalid_token(tokens, returned_index)),
                }
            }
        }

        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::Equal)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (right_hand_side, index) = match Expression::new(tokens, index, class_name) {
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
                array_index,
                right_hand_side,
            },
            index,
        )
    }
    fn to_string(&self) -> Vec<String> {
        let (open, close) = get_xml_tag("letStatement".to_string());
        let mut result = vec![open];
        result.push(to_xml_tag(token::Keyword::Let));
        result.push(self.var_name.to_string());

        // index
        if let Some(a) = &self.array_index {
            result.push(to_xml_tag(token::Symbol::LeftBracket));
            result = [result, a.to_string()].concat();
            result.push(to_xml_tag(token::Symbol::RightBracket));
        }

        result.push(to_xml_tag(token::Symbol::Equal));
        result = [result, self.right_hand_side.to_string()].concat();
        result.push(to_xml_tag(token::Symbol::SemiColon));
        result.push(close);
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
struct IfStatement {
    condition: Expression,
    positive_case_body: Statements,
    negative_case_body: Option<Statements>,
}
impl IfStatement {
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Self, usize) {
        let index = match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::If)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::LeftParen)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (condition, index) = match Expression::new(tokens, index, class_name) {
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
        let (positive_case_body, index) = Statements::new(tokens, index, class_name);
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
                let (negative_case_body, index) = Statements::new(tokens, index, class_name);
                let index = match tokens.get(index) {
                    Some(token::Token::Sym(token::Symbol::RightBrace)) => index + 1,
                    _ => panic!("{}", invalid_token(tokens, index)),
                };
                (
                    Self {
                        condition,
                        positive_case_body,
                        negative_case_body: Some(negative_case_body),
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
    fn to_string(&self) -> Vec<String> {
        let (open, close) = get_xml_tag("ifStatement".to_string());
        let mut result = vec![open];
        result.push(to_xml_tag(token::Keyword::If));
        result.push(to_xml_tag(token::Symbol::LeftParen));
        result = [result, self.condition.to_string()].concat();
        result.push(to_xml_tag(token::Symbol::RightParen));
        result.push(to_xml_tag(token::Symbol::LeftBrace));

        let (statement_open, statement_close) = get_xml_tag("statements".to_string());
        result.push(statement_open);
        for p in &self.positive_case_body.0 {
            result = [result, p.to_string()].concat();
        }
        result.push(statement_close);
        result.push(to_xml_tag(token::Symbol::RightBrace));

        // panic!("{:?}", self);

        match &self.negative_case_body {
            Some(n_) => {
                result.push(to_xml_tag(token::Keyword::Else));
                result.push(to_xml_tag(token::Symbol::LeftBrace));
                let (statement_open, statement_close) = get_xml_tag("statements".to_string());
                result.push(statement_open);
                for n in &n_.0 {
                    result = [result, n.to_string()].concat();
                }
                result.push(statement_close);
                result.push(to_xml_tag(token::Symbol::RightBrace));
            }
            None => {}
        }

        result.push(close);
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
struct WhileStatement {
    condition: Expression,
    body: Statements,
}
impl WhileStatement {
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Self, usize) {
        let index = match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::While)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::LeftParen)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (condition, index) = match Expression::new(tokens, index, class_name) {
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
        let (body, index) = Statements::new(tokens, index, class_name);
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::RightBrace)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        (Self { condition, body }, index)
    }
    fn to_string(&self) -> Vec<String> {
        let (open, close) = get_xml_tag("whileStatement".to_string());
        let mut result = vec![open];
        result.push(to_xml_tag(token::Keyword::While));
        result.push(to_xml_tag(token::Symbol::LeftParen));
        result = [result, self.condition.to_string()].concat();
        result.push(to_xml_tag(token::Symbol::RightParen));
        result.push(to_xml_tag(token::Symbol::LeftBrace));

        let (statement_open, statement_close) = get_xml_tag("statements".to_string());
        result.push(statement_open);
        for p in &self.body.0 {
            result = [result, p.to_string()].concat();
        }
        result.push(statement_close);
        result.push(to_xml_tag(token::Symbol::RightBrace));
        result.push(close);
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DoStatement(SubroutineCall);
impl DoStatement {
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Self, usize) {
        let index = match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Do)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (subroutine_call, index) = SubroutineCall::new(tokens, index, class_name);
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::SemiColon)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        (Self(subroutine_call.unwrap()), index)
    }
    fn to_string(&self) -> Vec<String> {
        let (open, close) = get_xml_tag("doStatement".to_string());
        let mut result = vec![open];
        result.push(to_xml_tag(token::Keyword::Do));
        result = [result, self.0.to_string()].concat();
        result.push(to_xml_tag(token::Symbol::SemiColon));
        result.push(close);
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ReturnStatement(Option<Expression>);
impl ReturnStatement {
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Self, usize) {
        let index = match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Return)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (expression, index) = Expression::new(tokens, index, class_name);
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::SemiColon)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        (Self(expression), index)
    }
    fn to_string(&self) -> Vec<String> {
        let (open, close) = get_xml_tag("returnStatement".to_string());
        let mut result = vec![open];
        result.push(to_xml_tag(token::Keyword::Return));
        if let Some(e) = &self.0 {
            result = [result, e.to_string()].concat()
        }
        result.push(to_xml_tag(token::Symbol::SemiColon));
        result.push(close);
        result
    }
}

/*
 * 式
 */
#[derive(Debug, PartialEq, Eq)]
struct Expression {
    // 再帰参照になってしまうのでBoxでくるんでいる
    term: Box<Term>,
    op_term: Vec<(Op, Term)>,
}
impl Expression {
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Option<Self>, usize) {
        let (term, mut index) = match Term::new(tokens, index, class_name) {
            (Some(t), i) => (t, i),
            _ => return (None, index),
        };

        let mut op_term = vec![];
        while let (Some(o), op_index) = Op::new(tokens, index) {
            index = op_index;
            if let (Some(t), term_index) = Term::new(tokens, op_index, class_name) {
                index = term_index;
                op_term.push((o, t));
            }
        }

        (
            Some(Expression {
                term: Box::new(term),
                op_term,
            }),
            index,
        )
    }
    fn to_string(&self) -> Vec<String> {
        let mut result = vec![];
        let (open, close) = get_xml_tag("expression".to_string());
        result.push(open);
        result = [result, self.term.to_string()].concat();
        for o in &self.op_term {
            result.push(o.0.to_string());
            result = [result, o.1.to_string()].concat();
        }

        result.push(close);
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Term {
    IntegerConstant(token::IntegerConstant),
    StringConstant(token::StringConstant),
    KeyWordConstant(KeyWordConstant),
    VarName(VarName),
    ArrayIndexAccess(VarName, Expression),
    Expression(Expression),
    UnaryOp(UnaryOp, Box<Term>),
    SubroutineCall(SubroutineCall),
}
impl Term {
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Option<Self>, usize) {
        if let (Some(s), index) = SubroutineCall::new(tokens, index, class_name) {
            return (Some(Term::SubroutineCall(s)), index);
        }
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
            Some(token::Token::Identifier(i)) => {
                // VarName[index]のパターン
                match tokens.get(index + 1) {
                    Some(token::Token::Sym(token::Symbol::LeftBracket)) => {
                        match Expression::new(tokens, index + 2, class_name) {
                            (Some(ex), index) => match tokens.get(index) {
                                Some(token::Token::Sym(token::Symbol::RightBracket)) => {
                                    // index[len-2]みたいなパターンもあるので。Expression::new()
                                    // から返ってきたindexを使う必要があることに注意
                                    (Some(Term::ArrayIndexAccess(VarName(i.clone()), ex)), index + 1)
                                }
                                _ => panic!("{}", invalid_token(tokens, index)),
                            },
                            // たまたま Identifier, [, なにか, ]という並びになる可能性もある。なのでVarName[index]としてパースできなかったら
                            // VarNameとして扱っておく
                            _ => (Some(Term::VarName(VarName(i.clone()))), index + 1),
                        }
                    }

                    // VarNameのパターン
                    _ => (Some(Term::VarName(VarName(i.clone()))), index + 1),
                }
            }
            Some(token::Token::Sym(token::Symbol::LeftParen)) => {
                let index = match tokens.get(index) {
                    Some(token::Token::Sym(token::Symbol::LeftParen)) => index + 1,
                    _ => return (None, index), // ExpressionListから見るとtermが空のパターンもあるのでpanicしてはならない
                };
                let (expression, index) = Expression::new(tokens, index, class_name);

                let index = match tokens.get(index) {
                    Some(token::Token::Sym(token::Symbol::RightParen)) => index + 1,
                    _ => return (None, index), // ExpressionListから見るとtermが空のパターンもあるのでpanicしてはならない
                };
                (Some(Term::Expression(expression.unwrap())), index)
            }

            // 単に1token読んだだけではわからないパターン
            _ => {
                if let (Some(u), index) = UnaryOp::new(tokens, index) {
                    match Term::new(tokens, index, class_name) {
                        (Some(t), index) => {
                            return (Some(Term::UnaryOp(u, Box::new(t))), index);
                        }
                        _ => panic!("{}", invalid_token(tokens, index)),
                    }
                };
                (None, index)
            }
        }
    }
    fn to_string(&self) -> Vec<String> {
        let mut result = vec![];
        let (open, close) = get_xml_tag("term".to_string());
        result.push(open);

        let content = match self {
            Term::IntegerConstant(s) => vec![s.to_string()],
            Term::StringConstant(s) => vec![s.to_string()],
            Term::KeyWordConstant(s) => vec![s.to_string()],
            Term::VarName(s) => vec![s.to_string()],
            Term::ArrayIndexAccess(v, e) => {
                let mut result = vec![v.to_string()];
                result.push(to_xml_tag(token::Symbol::LeftBracket));
                result = [result, e.to_string()].concat();
                result.push(to_xml_tag(token::Symbol::RightBracket));
                result
            }
            Term::Expression(s) => {
                let mut result = vec![to_xml_tag(token::Symbol::LeftParen)];
                result = [result, s.to_string()].concat();
                result.push(to_xml_tag(token::Symbol::RightParen));
                result
            }
            Term::UnaryOp(u, t) => {
                let mut result = vec![u.to_string()];
                result = [result, t.to_string()].concat();
                result
            }
            Term::SubroutineCall(s) => s.to_string(),
        };
        result = [result, content].concat();

        result.push(close);
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
enum KeyWordConstant {
    True,
    False,
    Null,
    This,
}
impl KeyWordConstant {
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        let (open, close) = get_xml_tag("keyword".to_string());
        let content = match self {
            KeyWordConstant::True => "true".to_string(),
            KeyWordConstant::False => "false".to_string(),
            KeyWordConstant::Null => "null".to_string(),
            KeyWordConstant::This => "this".to_string(),
        };

        format!("{} {} {}", open, content, close)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SubroutineCall {
    receiver: Option<Receiver>,
    name: SubroutineName,
    arguments: ExpressionList,
}
impl SubroutineCall {
    // NOTE: _class_nameは必要なくなったがあとで必要になるかもなのでいったん残しておく
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Option<Self>, usize) {
        let exist_receiver = matches!(tokens.get(index + 1), Some(token::Token::Sym(token::Symbol::Dot)));

        if exist_receiver {
            let (receiver, index) = match tokens.get(index) {
                Some(token::Token::Identifier(i)) => {
                    // 1文字目が大文字ならclass_nameだと判定する
                    if i.0.chars().next().map_or(false, char::is_uppercase) {
                        (Some(Receiver::ClassName(ClassName(i.clone()))), index + 1)
                    } else {
                        (Some(Receiver::VarName(VarName(i.clone()))), index + 1)
                    }
                }
                _ => (None, index),
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
            let (arguments, index) = match ExpressionList::new(tokens, index, class_name) {
                (Some(el), returned_index) => (el, returned_index),
                _ => (ExpressionList(vec![]), index),
            };
            let index = match tokens.get(index) {
                Some(token::Token::Sym(token::Symbol::RightParen)) => index + 1,
                _ => panic!("{}", invalid_token(tokens, index)),
            };

            (
                Some(Self {
                    receiver,
                    name,
                    arguments,
                }),
                index,
            )
        } else {
            let (subroutine_name, index) = match tokens.get(index) {
                Some(token::Token::Identifier(i)) => (SubroutineName(i.clone()), index + 1),
                _ => return (None, index),
            };
            let index = match tokens.get(index) {
                Some(token::Token::Sym(token::Symbol::LeftParen)) => index + 1,
                _ => return (None, index),
            };
            let (arguments, index) = match ExpressionList::new(tokens, index, class_name) {
                (Some(el), index) => (el, index),
                _ => (ExpressionList(vec![]), index),
            };
            let index = match tokens.get(index) {
                Some(token::Token::Sym(token::Symbol::RightParen)) => index + 1,
                _ => return (None, index),
            };

            (
                Some(Self {
                    receiver: None,
                    name: subroutine_name,
                    arguments,
                }),
                index,
            )
        }
    }
    fn to_string(&self) -> Vec<String> {
        let mut result = vec![];
        if let Some(r) = &self.receiver {
            result.push(r.to_string());
            result.push(to_xml_tag(token::Symbol::Dot));
        }
        result.push(self.name.0.to_string());
        result.push(to_xml_tag(token::Symbol::LeftParen));

        let (open, close) = get_xml_tag("expressionList".to_string());
        result.push(open);
        for (index, a) in self.arguments.0.iter().enumerate() {
            if index != 0 {
                result.push(to_xml_tag(token::Symbol::Comma));
            }
            result = [result, a.to_string()].concat();
        }
        result.push(close);

        result.push(to_xml_tag(token::Symbol::RightParen));
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Receiver {
    ClassName(ClassName),
    VarName(VarName),
}
impl Receiver {
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        match self {
            Receiver::ClassName(c) => c.0.to_string(),
            Receiver::VarName(v) => v.0.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ExpressionList(Vec<Expression>);
impl ExpressionList {
    // 無
    // expression
    // expression, expression, ..., expression
    fn new(tokens: &[token::Token], index: usize, class_name: &ClassName) -> (Option<Self>, usize) {
        let (expression, index) = match Expression::new(tokens, index, class_name) {
            (Some(expression), returned_index) => (expression, returned_index),
            _ => return (None, index),
        };

        let mut index = index;
        let mut expression_list = vec![expression];
        while let Some(token::Token::Sym(token::Symbol::Comma)) = tokens.get(index) {
            index += 1;
            match Expression::new(tokens, index, class_name) {
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

#[derive(Debug, PartialEq, Eq)]
enum Op {
    Plus,
    Minus,
    Multiply,
    Div,
    Ampersand,
    Pipe,
    LessThan,
    MoreThan,
    Equal,
}
impl Op {
    fn new(tokens: &[token::Token], index: usize) -> (Option<Self>, usize) {
        match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::Plus)) => (Some(Op::Plus), index + 1),
            Some(token::Token::Sym(token::Symbol::Minus)) => (Some(Op::Minus), index + 1),
            Some(token::Token::Sym(token::Symbol::Asterisk)) => (Some(Op::Multiply), index + 1),
            Some(token::Token::Sym(token::Symbol::Slash)) => (Some(Op::Div), index + 1),
            Some(token::Token::Sym(token::Symbol::Ampersand)) => (Some(Op::Ampersand), index + 1),
            Some(token::Token::Sym(token::Symbol::Pipe)) => (Some(Op::Pipe), index + 1),
            Some(token::Token::Sym(token::Symbol::LessThan)) => (Some(Op::LessThan), index + 1),
            Some(token::Token::Sym(token::Symbol::MoreThan)) => (Some(Op::MoreThan), index + 1),
            Some(token::Token::Sym(token::Symbol::Equal)) => (Some(Op::Equal), index + 1),
            _ => (None, index),
        }
    }
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        let content = match self {
            Op::Plus => "+".to_string(),
            Op::Minus => "-".to_string(),
            Op::Multiply => "*".to_string(),
            Op::Div => "/".to_string(),
            Op::Ampersand => "&amp;".to_string(),
            Op::Pipe => "|".to_string(),
            Op::LessThan => "&lt;".to_string(),
            Op::MoreThan => "&gt;".to_string(),
            Op::Equal => "=".to_string(),
        };
        let (open, close) = get_xml_tag("symbol".to_string());
        format!("{} {} {}", open, content, close)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum UnaryOp {
    Minus,
    Tilde,
}
impl UnaryOp {
    fn new(tokens: &[token::Token], index: usize) -> (Option<Self>, usize) {
        match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::Minus)) => (Some(UnaryOp::Minus), index + 1),
            Some(token::Token::Sym(token::Symbol::Tilde)) => (Some(UnaryOp::Tilde), index + 1),
            _ => (None, index),
        }
    }
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        let content = match self {
            UnaryOp::Minus => "-",
            UnaryOp::Tilde => "~",
        };
        let (open, close) = get_xml_tag("symbol".to_string());
        format!("{} {} {}", open, content, close)
    }
}

fn invalid_token(tokens: &[token::Token], index: usize) -> String {
    format!("invalid token {:?}[@{}]", tokens, index)
}

fn get_xml_tag(tag_name: String) -> (String, String) {
    (format!("<{}>", tag_name), format!("</{}>", tag_name))
}
fn to_xml_tag<T: std::fmt::Debug>(value: T) -> String {
    let type_name_slice = std::any::type_name_of_val(&value).split("::").collect::<Vec<&str>>();
    let tag_name = to_lowercase_at_1(type_name_slice[type_name_slice.len() - 1]);
    format!("<{}> {} </{}>", &tag_name, format!("{:?}", value).to_lowercase(), &tag_name)
}

fn to_lowercase_at_1(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for (i, c) in s.chars().enumerate() {
        if i == 0 {
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use token::IntegerConstant;

    #[test]
    fn test_class_new() {
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
        }
        */
        let input = Class::new(
            &vec![
                token::Token::Key(token::Keyword::Class),
                token::Token::Identifier(token::Identifier("SquareGame".to_string())),
                token::Token::Sym(token::Symbol::LeftBrace),
                // field Square square;
                token::Token::Key(token::Keyword::Field),
                token::Token::Identifier(token::Identifier("Square".to_string())),
                token::Token::Identifier(token::Identifier("square".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
                // field int direction;
                token::Token::Key(token::Keyword::Field),
                token::Token::Key(token::Keyword::Int),
                token::Token::Identifier(token::Identifier("direction".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
                // constructor
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
                // dispose
                token::Token::Key(token::Keyword::Method),
                token::Token::Key(token::Keyword::Void),
                token::Token::Identifier(token::Identifier("dispose".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Key(token::Keyword::Do),
                token::Token::Identifier(token::Identifier("square".to_string())),
                token::Token::Sym(token::Symbol::Dot),
                token::Token::Identifier(token::Identifier("dispose".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Do),
                token::Token::Identifier(token::Identifier("Memory".to_string())),
                token::Token::Sym(token::Symbol::Dot),
                token::Token::Identifier(token::Identifier("deAlloc".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Identifier(token::Identifier("square".to_string())),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Return),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Sym(token::Symbol::RightBrace),
                token::Token::Sym(token::Symbol::RightBrace),
            ],
            0,
        );
        let expected = Some(Class {
            name: ClassName(token::Identifier("SquareGame".to_string())),
            var_dec: vec![
                ClassVarDec {
                    kind: ClassVarKind::Field,
                    type_: Type::ClassName("Square".to_string()),
                    var_names: vec![VarName(token::Identifier("square".to_string()))],
                },
                ClassVarDec {
                    kind: ClassVarKind::Field,
                    type_: Type::Int,
                    var_names: vec![VarName(token::Identifier("direction".to_string()))],
                },
            ],
            subroutine_dec: vec![
                SubroutineDec {
                    kind: SubroutineDecKind::Constructor,
                    type_: SubroutineDecType::Type_(Type::ClassName("SquareGame".to_string())),
                    subroutine_name: token::Identifier("new".to_string()),
                    parameter_list: ParameterList(vec![]),
                    body: SubroutineBody {
                        var_dec: vec![],
                        statements: Statements(vec![
                            Statement::Let(LetStatement {
                                var_name: VarName(token::Identifier("square".to_string())),
                                array_index: None,
                                right_hand_side: Expression {
                                    term: Box::new(Term::VarName(VarName(token::Identifier("square".to_string())))),
                                    op_term: vec![],
                                },
                            }),
                            Statement::Let(LetStatement {
                                var_name: VarName(token::Identifier("direction".to_string())),
                                array_index: None,
                                right_hand_side: Expression {
                                    term: Box::new(Term::VarName(VarName(token::Identifier("direction".to_string())))),
                                    op_term: vec![],
                                },
                            }),
                            Statement::Return(ReturnStatement(Some(Expression {
                                term: Box::new(Term::VarName(VarName(token::Identifier("square".to_string())))),
                                op_term: vec![],
                            }))),
                        ]),
                    },
                },
                SubroutineDec {
                    kind: SubroutineDecKind::Method,
                    type_: SubroutineDecType::Void,
                    subroutine_name: token::Identifier("dispose".to_string()),
                    parameter_list: ParameterList(vec![]),
                    body: SubroutineBody {
                        var_dec: vec![],
                        statements: Statements(vec![
                            Statement::Do(DoStatement(SubroutineCall {
                                receiver: Some(Receiver::VarName(VarName(token::Identifier("square".to_string())))),
                                name: SubroutineName(token::Identifier("dispose".to_string())),
                                arguments: ExpressionList(vec![]),
                            })),
                            Statement::Do(DoStatement(SubroutineCall {
                                receiver: Some(Receiver::ClassName(ClassName(token::Identifier("Memory".to_string())))),
                                name: SubroutineName(token::Identifier("deAlloc".to_string())),
                                arguments: ExpressionList(vec![Expression {
                                    term: Box::new(Term::VarName(VarName(token::Identifier("square".to_string())))),
                                    op_term: vec![],
                                }]),
                            })),
                            Statement::Return(ReturnStatement(None)),
                        ]),
                    },
                },
            ],
        });
        assert_eq!(input, expected);
    }

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
            constructor SquareGame new(int x, int y) {
                var boolean b;

                let square = square;
                let direction = direction;
                return square;
            }
        */
        let input = SubroutineDec::new(
            &vec![
                token::Token::Key(token::Keyword::Constructor),
                token::Token::Identifier(token::Identifier("SquareGame".to_string())),
                token::Token::Identifier(token::Identifier("new".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Key(token::Keyword::Int),
                token::Token::Identifier(token::Identifier("x".to_string())),
                token::Token::Sym(token::Symbol::Comma),
                token::Token::Key(token::Keyword::Int),
                token::Token::Identifier(token::Identifier("y".to_string())),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Key(token::Keyword::Var),
                token::Token::Key(token::Keyword::Boolean),
                token::Token::Identifier(token::Identifier("b".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
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
            ],
            0,
            &ClassName(token::Identifier("SquareGame".to_string())),
        );
        let expected = (
            Some(SubroutineDec {
                kind: SubroutineDecKind::Constructor,
                type_: SubroutineDecType::Type_(Type::ClassName("SquareGame".to_string())),
                subroutine_name: token::Identifier("new".to_string()),
                parameter_list: ParameterList(vec![
                    (Type::Int, VarName(token::Identifier("x".to_string()))),
                    (Type::Int, VarName(token::Identifier("y".to_string()))),
                ]),
                body: SubroutineBody {
                    var_dec: vec![VarDec {
                        type_: Type::Boolean,
                        var_name: vec![VarName(token::Identifier("b".to_string()))],
                    }],
                    statements: Statements(vec![
                        Statement::Let(LetStatement {
                            var_name: VarName(token::Identifier("square".to_string())),
                            array_index: None,
                            right_hand_side: Expression {
                                term: Box::new(Term::VarName(VarName(token::Identifier("square".to_string())))),
                                op_term: vec![],
                            },
                        }),
                        Statement::Let(LetStatement {
                            var_name: VarName(token::Identifier("direction".to_string())),
                            array_index: None,
                            right_hand_side: Expression {
                                term: Box::new(Term::VarName(VarName(token::Identifier("direction".to_string())))),
                                op_term: vec![],
                            },
                        }),
                        Statement::Return(ReturnStatement(Some(Expression {
                            term: Box::new(Term::VarName(VarName(token::Identifier("square".to_string())))),
                            op_term: vec![],
                        }))),
                    ]),
                },
            }),
            29,
        );
        assert_eq!(input, expected);

        /*
             function void dispose() {
                do square.dispose();
                do Memory.deAlloc(square);
                return;
             }
        */
        let input = SubroutineDec::new(
            &vec![
                token::Token::Key(token::Keyword::Function),
                token::Token::Key(token::Keyword::Void),
                token::Token::Identifier(token::Identifier("dispose".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Key(token::Keyword::Do),
                token::Token::Identifier(token::Identifier("square".to_string())),
                token::Token::Sym(token::Symbol::Dot),
                token::Token::Identifier(token::Identifier("dispose".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Do),
                token::Token::Identifier(token::Identifier("Memory".to_string())),
                token::Token::Sym(token::Symbol::Dot),
                token::Token::Identifier(token::Identifier("deAlloc".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Identifier(token::Identifier("square".to_string())),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Return),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Sym(token::Symbol::RightBrace),
            ],
            0,
            &ClassName(token::Identifier("SquareGame".to_string())),
        );
        let expected = (
            Some(SubroutineDec {
                kind: SubroutineDecKind::Function,
                type_: SubroutineDecType::Void,
                subroutine_name: token::Identifier("dispose".to_string()),
                parameter_list: ParameterList(vec![]),
                body: SubroutineBody {
                    var_dec: vec![],
                    statements: Statements(vec![
                        Statement::Do(DoStatement(SubroutineCall {
                            receiver: Some(Receiver::VarName(VarName(token::Identifier("square".to_string())))),
                            name: SubroutineName(token::Identifier("dispose".to_string())),
                            arguments: ExpressionList(vec![]),
                        })),
                        Statement::Do(DoStatement(SubroutineCall {
                            receiver: Some(Receiver::ClassName(ClassName(token::Identifier("Memory".to_string())))),
                            name: SubroutineName(token::Identifier("deAlloc".to_string())),
                            arguments: ExpressionList(vec![Expression {
                                term: Box::new(Term::VarName(VarName(token::Identifier("square".to_string())))),
                                op_term: vec![],
                            }]),
                        })),
                        Statement::Return(ReturnStatement(None)),
                    ]),
                },
            }),
            24,
        );
        assert_eq!(input, expected);

        /*
             method void moveSquare() {
                if (direction) { do square.moveUp(); }
                do Sys.wait(direction);
                return;
             }
        }
        */
        let input = SubroutineDec::new(
            &vec![
                token::Token::Key(token::Keyword::Method),
                token::Token::Key(token::Keyword::Void),
                token::Token::Identifier(token::Identifier("moveSquare".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Key(token::Keyword::If),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Identifier(token::Identifier("direction".to_string())),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Key(token::Keyword::Do),
                token::Token::Identifier(token::Identifier("square".to_string())),
                token::Token::Sym(token::Symbol::Dot),
                token::Token::Identifier(token::Identifier("moveUp".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Sym(token::Symbol::RightBrace),
                token::Token::Key(token::Keyword::Do),
                token::Token::Identifier(token::Identifier("Sys".to_string())),
                token::Token::Sym(token::Symbol::Dot),
                token::Token::Identifier(token::Identifier("wait".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Identifier(token::Identifier("direction".to_string())),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Return),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Sym(token::Symbol::RightBrace),
            ],
            0,
            &ClassName(token::Identifier("SquareGame".to_string())),
        );
        let expected = (
            Some(SubroutineDec {
                kind: SubroutineDecKind::Method,
                type_: SubroutineDecType::Void,
                subroutine_name: token::Identifier("moveSquare".to_string()),
                parameter_list: ParameterList(vec![]),
                body: SubroutineBody {
                    var_dec: vec![],
                    statements: Statements(vec![
                        Statement::If(IfStatement {
                            condition: Expression {
                                term: Box::new(Term::VarName(VarName(token::Identifier("direction".to_string())))),
                                op_term: vec![],
                            },
                            positive_case_body: Statements(vec![Statement::Do(DoStatement(SubroutineCall {
                                receiver: Some(Receiver::VarName(VarName(token::Identifier("square".to_string())))),
                                name: SubroutineName(token::Identifier("moveUp".to_string())),
                                arguments: ExpressionList(vec![]),
                            }))]),
                            negative_case_body: None,
                        }),
                        Statement::Do(DoStatement(SubroutineCall {
                            receiver: Some(Receiver::ClassName(ClassName(token::Identifier("Sys".to_string())))),
                            name: SubroutineName(token::Identifier("wait".to_string())),
                            arguments: ExpressionList(vec![Expression {
                                term: Box::new(Term::VarName(VarName(token::Identifier("direction".to_string())))),
                                op_term: vec![],
                            }]),
                        })),
                        Statement::Return(ReturnStatement(None)),
                    ]),
                },
            }),
            30,
        );
        assert_eq!(input, expected);
    }

    #[test]
    fn test_type_new() {
        let input = Type::new(&[token::Token::Key(token::Keyword::Int)], 0);
        let expected = (Some(Type::Int), 1);
        assert_eq!(input, expected);

        let input = Type::new(&[token::Token::Key(token::Keyword::Char)], 0);
        let expected = (Some(Type::Char), 1);
        assert_eq!(input, expected);

        let input = Type::new(&[token::Token::Key(token::Keyword::Boolean)], 0);
        let expected = (Some(Type::Boolean), 1);
        assert_eq!(input, expected);

        let input = Type::new(&[token::Token::Identifier(token::Identifier("main".to_string()))], 0);
        let expected = (Some(Type::ClassName("main".to_string())), 1);
        assert_eq!(input, expected);
    }

    #[test]
    fn test_var_dec_new() {
        // var MyType foo;
        let input = VarDec::new(
            &[
                token::Token::Key(token::Keyword::Var),
                token::Token::Identifier(token::Identifier("MyType".to_string())),
                token::Token::Identifier(token::Identifier("foo".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
        );
        let expected = (
            Some(VarDec {
                type_: Type::ClassName("MyType".to_string()),
                var_name: vec![VarName(token::Identifier("foo".to_string()))],
            }),
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
            Some(VarDec {
                type_: Type::Int,
                var_name: vec![
                    VarName(token::Identifier("x".to_string())),
                    VarName(token::Identifier("y".to_string())),
                    VarName(token::Identifier("z".to_string())),
                ],
            }),
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
            &[
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
        let input = ParameterList::new(&[], 0);
        let expected = (ParameterList(vec![]), 0);
        assert_eq!(input, expected);
    }

    #[test]
    fn test_expression_list_new() {
        /*
            true, false
        */
        let input = ExpressionList::new(
            &[
                token::Token::Key(token::Keyword::True),
                token::Token::Sym(token::Symbol::Comma),
                token::Token::Key(token::Keyword::False),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(ExpressionList(vec![
                Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                    op_term: vec![],
                },
                Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::False)),
                    op_term: vec![],
                },
            ])),
            3,
        );
        assert_eq!(input, expected);

        /*
            1
        */
        let input = ExpressionList::new(
            &[token::Token::IntegerConstant(token::IntegerConstant(1))],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(ExpressionList(vec![Expression {
                term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                op_term: vec![],
            }])),
            1,
        );
        assert_eq!(input, expected);

        /*
            (引数なし)
        */
        let input = ExpressionList::new(&[], 0, &ClassName(token::Identifier("Main".to_string())));
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
            Some(SubroutineCall {
                receiver: Some(Receiver::ClassName(ClassName(token::Identifier("Main".to_string())))),
                name: SubroutineName(token::Identifier("show".to_string())),
                arguments: ExpressionList(vec![
                    Expression {
                        term: Box::new(Term::VarName(VarName(token::Identifier("x".to_string())))),
                        op_term: vec![],
                    },
                    Expression {
                        term: Box::new(Term::VarName(VarName(token::Identifier("y".to_string())))),
                        op_term: vec![],
                    },
                ]),
            }),
            8,
        );
        assert_eq!(input, expected);

        /*
            person.show()(レシーバがvar_name)
        */
        let input = SubroutineCall::new(
            &[
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
            Some(SubroutineCall {
                receiver: Some(Receiver::VarName(VarName(token::Identifier("person".to_string())))),
                name: SubroutineName(token::Identifier("show".to_string())),
                arguments: ExpressionList(vec![]),
            }),
            5,
        );
        assert_eq!(input, expected);

        /*
            show(x, y)(レシーバなし)
        */
        let input = SubroutineCall::new(
            &[
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
            Some(SubroutineCall {
                receiver: None,
                name: SubroutineName(token::Identifier("show".to_string())),
                arguments: ExpressionList(vec![
                    Expression {
                        term: Box::new(Term::VarName(VarName(token::Identifier("x".to_string())))),
                        op_term: vec![],
                    },
                    Expression {
                        term: Box::new(Term::VarName(VarName(token::Identifier("y".to_string())))),
                        op_term: vec![],
                    },
                ]),
            }),
            6,
        );
        assert_eq!(input, expected);

        /*
            show()(レシーバ、引数なし)
        */
        let input = SubroutineCall::new(
            &[
                token::Token::Identifier(token::Identifier("show".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::RightParen),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(SubroutineCall {
                receiver: None,
                name: SubroutineName(token::Identifier("show".to_string())),
                arguments: ExpressionList(vec![]),
            }),
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
            &[
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("foo".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            LetStatement {
                var_name: VarName(token::Identifier("foo".to_string())),
                array_index: None,
                right_hand_side: Expression {
                    term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                    op_term: vec![],
                },
            },
            5,
        );
        assert_eq!(input, expected);

        // let length = Keyboard.readInt("HOW MANY NUMBERS? ");
        let input = LetStatement::new(
            &[
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("length".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::Identifier(token::Identifier("Keyboard".to_string())),
                token::Token::Sym(token::Symbol::Dot),
                token::Token::Identifier(token::Identifier("readInt".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::StringConstant(token::StringConstant("HOW MANY NUMBERS? ".to_string())),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            LetStatement {
                var_name: VarName(token::Identifier("length".to_string())),
                array_index: None,
                right_hand_side: Expression {
                    term: Box::new(Term::SubroutineCall(SubroutineCall {
                        receiver: Some(Receiver::ClassName(ClassName(token::Identifier("Keyboard".to_string())))),
                        name: SubroutineName(token::Identifier("readInt".to_string())),
                        arguments: ExpressionList(vec![Expression {
                            term: Box::new(Term::StringConstant(token::StringConstant(
                                "HOW MANY NUMBERS? ".to_string(),
                            ))),
                            op_term: vec![],
                        }]),
                    })),
                    op_term: vec![],
                },
            },
            10,
        );
        assert_eq!(input, expected);

        // let a[i] = Keyboard.readInt("HOW MANY NUMBERS? ");
        let input = LetStatement::new(
            &[
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("a".to_string())),
                token::Token::Sym(token::Symbol::LeftBracket),
                token::Token::Identifier(token::Identifier("i".to_string())),
                token::Token::Sym(token::Symbol::RightBracket),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::Identifier(token::Identifier("Keyboard".to_string())),
                token::Token::Sym(token::Symbol::Dot),
                token::Token::Identifier(token::Identifier("readInt".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::StringConstant(token::StringConstant("HOW MANY NUMBERS? ".to_string())),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            LetStatement {
                var_name: VarName(token::Identifier("a".to_string())),
                array_index: Some(Expression {
                    term: Box::new(Term::VarName(VarName(token::Identifier("i".to_string())))),
                    op_term: vec![],
                }),
                right_hand_side: Expression {
                    term: Box::new(Term::SubroutineCall(SubroutineCall {
                        receiver: Some(Receiver::ClassName(ClassName(token::Identifier("Keyboard".to_string())))),
                        name: SubroutineName(token::Identifier("readInt".to_string())),
                        arguments: ExpressionList(vec![Expression {
                            term: Box::new(Term::StringConstant(token::StringConstant(
                                "HOW MANY NUMBERS? ".to_string(),
                            ))),
                            op_term: vec![],
                        }]),
                    })),
                    op_term: vec![],
                },
            },
            13,
        );
        assert_eq!(input, expected);

        // let sum = sum + a[i];
        let input = LetStatement::new(
            &[
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("sum".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::Identifier(token::Identifier("sum".to_string())),
                token::Token::Sym(token::Symbol::Plus),
                token::Token::Identifier(token::Identifier("a".to_string())),
                token::Token::Sym(token::Symbol::LeftBracket),
                token::Token::Identifier(token::Identifier("i".to_string())),
                token::Token::Sym(token::Symbol::RightBracket),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            LetStatement {
                var_name: VarName(token::Identifier("sum".to_string())),
                array_index: None,
                right_hand_side: Expression {
                    term: Box::new(Term::VarName(VarName(token::Identifier("sum".to_string())))),
                    op_term: vec![(
                        Op::Plus,
                        Term::ArrayIndexAccess(
                            VarName(token::Identifier("a".to_string())),
                            Expression {
                                term: Box::new(Term::VarName(VarName(token::Identifier("i".to_string())))),
                                op_term: vec![],
                            },
                        ),
                    )],
                },
            },
            10,
        );
        assert_eq!(input, expected);

        // let j = j / (-2);
        let input = LetStatement::new(
            &[
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("j".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::Identifier(token::Identifier("j".to_string())),
                token::Token::Sym(token::Symbol::Slash),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::Minus),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            LetStatement {
                var_name: VarName(token::Identifier("j".to_string())),
                array_index: None,
                right_hand_side: Expression {
                    term: Box::new(Term::VarName(VarName(token::Identifier("j".to_string())))),
                    op_term: vec![(
                        Op::Div,
                        Term::Expression(Expression {
                            term: Box::new(Term::UnaryOp(
                                UnaryOp::Minus,
                                Box::new(Term::IntegerConstant(token::IntegerConstant(1))),
                            )),
                            op_term: vec![],
                        }),
                    )],
                },
            },
            10,
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
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            IfStatement {
                condition: Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                    op_term: vec![],
                },
                positive_case_body: Statements(vec![
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("foo".to_string())),
                        array_index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                            op_term: vec![],
                        },
                    }),
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("bar".to_string())),
                        array_index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                            op_term: vec![],
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
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            IfStatement {
                condition: Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                    op_term: vec![],
                },
                positive_case_body: Statements(vec![
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("foo".to_string())),
                        array_index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                            op_term: vec![],
                        },
                    }),
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("bar".to_string())),
                        array_index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                            op_term: vec![],
                        },
                    }),
                ]),
                negative_case_body: Some(Statements(vec![
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("baz".to_string())),
                        array_index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                            op_term: vec![],
                        },
                    }),
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("qux".to_string())),
                        array_index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::KeyWordConstant(KeyWordConstant::Null)),
                            op_term: vec![],
                        },
                    }),
                ])),
            },
            29,
        );
        assert_eq!(input, expected);

        /*
            if (true) {} else {}
        */
        let input = IfStatement::new(
            &vec![
                token::Token::Key(token::Keyword::If),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Key(token::Keyword::True),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Sym(token::Symbol::RightBrace),
                token::Token::Key(token::Keyword::Else),
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Sym(token::Symbol::RightBrace),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            IfStatement {
                condition: Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                    op_term: vec![],
                },
                positive_case_body: Statements(vec![]),
                negative_case_body: Some(Statements(vec![])),
            },
            9,
        );
        assert_eq!(input, expected);
    }

    #[test]
    fn test_while_statement_new() {
        /*
            while (true) { let foo = 1; let bar = true; }
        */
        let input = WhileStatement::new(
            &vec![
                token::Token::Key(token::Keyword::While),
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
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            WhileStatement {
                condition: Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                    op_term: vec![],
                },
                body: Statements(vec![
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("foo".to_string())),
                        array_index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                            op_term: vec![],
                        },
                    }),
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("bar".to_string())),
                        array_index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                            op_term: vec![],
                        },
                    }),
                ]),
            },
            16,
        );
        assert_eq!(input, expected);
    }

    #[test]
    fn test_do_statement_new() {
        /*
            do game.run();
        */
        let input = DoStatement::new(
            &vec![
                token::Token::Key(token::Keyword::Do),
                token::Token::Identifier(token::Identifier("game".to_string())),
                token::Token::Sym(token::Symbol::Dot),
                token::Token::Identifier(token::Identifier("run".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            DoStatement(SubroutineCall {
                receiver: Some(Receiver::VarName(VarName(token::Identifier("game".to_string())))),
                name: SubroutineName(token::Identifier("run".to_string())),
                arguments: ExpressionList(vec![]),
            }),
            7,
        );
        assert_eq!(input, expected);
    }

    #[test]
    fn test_return_statement_new() {
        /*
            return;
        */
        let input = ReturnStatement::new(
            &[
                token::Token::Key(token::Keyword::Return),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (ReturnStatement(None), 2);
        assert_eq!(input, expected);

        /*
            return true;
        */
        let input = ReturnStatement::new(
            &[
                token::Token::Key(token::Keyword::Return),
                token::Token::Key(token::Keyword::True),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            ReturnStatement(Some(Expression {
                term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                op_term: vec![],
            })),
            3,
        );
        assert_eq!(input, expected);
    }

    #[test]
    fn test_subroutine_body_new() {
        /* var_decが複数 && statementが複数
            {
                var int x, y;
                let foo = true;
                return x;
            }
        */
        let input = SubroutineBody::new(
            &vec![
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Key(token::Keyword::Var),
                token::Token::Key(token::Keyword::Int),
                token::Token::Identifier(token::Identifier("x".to_string())),
                token::Token::Sym(token::Symbol::Comma),
                token::Token::Identifier(token::Identifier("y".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("foo".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::Key(token::Keyword::True),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Key(token::Keyword::Return),
                token::Token::Identifier(token::Identifier("x".to_string())),
                token::Token::Sym(token::Symbol::SemiColon),
                token::Token::Sym(token::Symbol::RightBrace),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            SubroutineBody {
                var_dec: vec![VarDec {
                    type_: Type::Int,
                    var_name: vec![
                        VarName(token::Identifier("x".to_string())),
                        VarName(token::Identifier("y".to_string())),
                    ],
                }],
                statements: Statements(vec![
                    Statement::Let(LetStatement {
                        var_name: VarName(token::Identifier("foo".to_string())),
                        array_index: None,
                        right_hand_side: Expression {
                            term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                            op_term: vec![],
                        },
                    }),
                    Statement::Return(ReturnStatement(Some(Expression {
                        term: Box::new(Term::VarName(VarName(token::Identifier("x".to_string())))),
                        op_term: vec![],
                    }))),
                ]),
            },
            16,
        );
        assert_eq!(input, expected);

        /*  var_decもstatementsもなしのパターン
            {}
        */
        let input = SubroutineBody::new(
            &[
                token::Token::Sym(token::Symbol::LeftBrace),
                token::Token::Sym(token::Symbol::RightBrace),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            SubroutineBody {
                var_dec: vec![],
                statements: Statements(vec![]),
            },
            2,
        );
        assert_eq!(input, expected);
    }

    #[test]
    fn test_statement_new() {
        // let foo = 1;
        let input = Statement::new(
            &[
                token::Token::Key(token::Keyword::Let),
                token::Token::Identifier(token::Identifier("foo".to_string())),
                token::Token::Sym(token::Symbol::Equal),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(Statement::Let(LetStatement {
                var_name: VarName(token::Identifier("foo".to_string())),
                array_index: None,
                right_hand_side: Expression {
                    term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                    op_term: vec![],
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
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(Statement::If(IfStatement {
                condition: Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                    op_term: vec![],
                },
                positive_case_body: Statements(vec![Statement::Let(LetStatement {
                    var_name: VarName(token::Identifier("foo".to_string())),
                    array_index: None,
                    right_hand_side: Expression {
                        term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                        op_term: vec![],
                    },
                })]),
                negative_case_body: None,
            })),
            11,
        );
        assert_eq!(input, expected);

        // while (true) { let foo = 1; }
        let input = Statement::new(
            &vec![
                token::Token::Key(token::Keyword::While),
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
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(Statement::While(WhileStatement {
                condition: Expression {
                    term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                    op_term: vec![],
                },
                body: Statements(vec![Statement::Let(LetStatement {
                    var_name: VarName(token::Identifier("foo".to_string())),
                    array_index: None,
                    right_hand_side: Expression {
                        term: Box::new(Term::IntegerConstant(IntegerConstant(1))),
                        op_term: vec![],
                    },
                })]),
            })),
            11,
        );
        assert_eq!(input, expected);

        // do game.run();
        let input = Statement::new(
            &vec![
                token::Token::Key(token::Keyword::Do),
                token::Token::Identifier(token::Identifier("game".to_string())),
                token::Token::Sym(token::Symbol::Dot),
                token::Token::Identifier(token::Identifier("run".to_string())),
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::RightParen),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(Statement::Do(DoStatement(SubroutineCall {
                receiver: Some(Receiver::VarName(VarName(token::Identifier("game".to_string())))),
                name: SubroutineName(token::Identifier("run".to_string())),
                arguments: ExpressionList(vec![]),
            }))),
            7,
        );
        assert_eq!(input, expected);

        // return true;
        let input = Statement::new(
            &[
                token::Token::Key(token::Keyword::Return),
                token::Token::Key(token::Keyword::True),
                token::Token::Sym(token::Symbol::SemiColon),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(Statement::Return(ReturnStatement(Some(Expression {
                term: Box::new(Term::KeyWordConstant(KeyWordConstant::True)),
                op_term: vec![],
            })))),
            3,
        );
        assert_eq!(input, expected);
    }

    #[test]
    fn test_expression_new() {
        /*
            term
        */
        let input = Expression::new(
            &[token::Token::IntegerConstant(token::IntegerConstant(1))],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(Expression {
                term: Box::new(Term::IntegerConstant(token::IntegerConstant(1))),
                op_term: vec![],
            }),
            1,
        );
        assert_eq!(input, expected);

        /*
            term op term
        */
        let input = Expression::new(
            &[
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::Plus),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(Expression {
                term: Box::new(Term::IntegerConstant(token::IntegerConstant(1))),
                op_term: vec![(Op::Plus, Term::IntegerConstant(token::IntegerConstant(1)))],
            }),
            3,
        );
        assert_eq!(input, expected);

        /*
            term op term op term
        */
        let input = Expression::new(
            &[
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::Plus),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::Minus),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(Expression {
                term: Box::new(Term::IntegerConstant(token::IntegerConstant(1))),
                op_term: vec![
                    (Op::Plus, Term::IntegerConstant(token::IntegerConstant(1))),
                    (Op::Minus, Term::IntegerConstant(token::IntegerConstant(1))),
                ],
            }),
            5,
        );
        assert_eq!(input, expected);

        /*
            term op term op term
        */
        let input = Expression::new(
            &[
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::Plus),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
                token::Token::Sym(token::Symbol::Minus),
                token::Token::IntegerConstant(token::IntegerConstant(1)),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(Expression {
                term: Box::new(Term::IntegerConstant(token::IntegerConstant(1))),
                op_term: vec![
                    (Op::Plus, Term::IntegerConstant(token::IntegerConstant(1))),
                    (Op::Minus, Term::IntegerConstant(token::IntegerConstant(1))),
                ],
            }),
            5,
        );
        assert_eq!(input, expected);

        // (-2)
        let input = Expression::new(
            &[
                token::Token::Sym(token::Symbol::LeftParen),
                token::Token::Sym(token::Symbol::Minus),
                token::Token::IntegerConstant(token::IntegerConstant(2)),
                token::Token::Sym(token::Symbol::RightParen),
            ],
            0,
            &ClassName(token::Identifier("Main".to_string())),
        );
        let expected = (
            Some(Expression {
                term: Box::new(Term::Expression(Expression {
                    term: Box::new(Term::UnaryOp(
                        UnaryOp::Minus,
                        Box::new(Term::IntegerConstant(token::IntegerConstant(2))),
                    )),
                    op_term: vec![],
                })),
                op_term: vec![],
            }),
            4,
        );
        assert_eq!(input, expected);
    }
}
