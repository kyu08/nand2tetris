use crate::analyzer::token;
use rand::seq::SliceRandom;

pub struct Ast {
    class: Class,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SymbolTables {
    // kindごとにVecにSymbolを格納したほうがパフォーマンスはよさそうではある
    class_scope: std::collections::HashMap<String, ClassSymbol>,
    // key: subroutine_name, value: そのsubroutineのsymbol_table
    subroutine_scopes: std::collections::HashMap<String, std::collections::HashMap<String, SubroutineSymbol>>,
    // currentに現在処理中のsubroutine名
    // コンパイラフロントエンドで生成したsymbol_tableを参照するために保持している。
    // 関数に{現在処理中のsubroutine_name}を引き回す方法もなくはないが複雑になりそうだったので意図的にこうしている。
    current_subroutine_name: Option<String>,
    // 一意のラベルを生成するために必要
    file_name: String,
}
impl SymbolTables {
    fn default(file_name: String) -> Self {
        Self {
            class_scope: std::collections::HashMap::new(),
            subroutine_scopes: std::collections::HashMap::new(),
            current_subroutine_name: None,
            file_name,
        }
    }
    fn determine_next_item_index_class(&self, symbol_type: &ClassSymbolType) -> usize {
        let mut next_item_index = 0;
        for v in self.class_scope.values() {
            if v.symbol_type == *symbol_type {
                next_item_index += 1;
            }
        }

        next_item_index
    }
    fn determine_next_item_index_subroutine(
        &self,
        subroutine_name: &String,
        symbol_type: &SubroutineSymbolType,
    ) -> usize {
        let mut next_item_index = 0;
        // この時点ではまだsymbol_tableが存在しない可能性がある
        for v in self.subroutine_scopes.get(subroutine_name).unwrap().values() {
            if v.symbol_type == *symbol_type {
                next_item_index += 1;
            }
        }

        next_item_index
    }
    fn append_class_symbol(&self, name: String, type_: Type, symbol_type: ClassSymbolType) -> Self {
        let index = self.determine_next_item_index_class(&symbol_type);
        let sym = ClassSymbol {
            name: name.clone(),
            type_,
            symbol_type,
            index,
        };
        let mut st = self.clone();
        let _ = st.class_scope.insert(name, sym);
        st
    }
    fn append_subroutine_symbol(&self, name: String, type_: Type, symbol_type: SubroutineSymbolType) -> Self {
        let current_subroutine_name = self.current_subroutine_name.clone().unwrap();
        let index = self.determine_next_item_index_subroutine(&current_subroutine_name, &symbol_type);
        let sym = SubroutineSymbol {
            name: name.clone(),
            type_,
            symbol_type,
            index,
        };
        let mut st = self.clone();
        let _ = st
            .subroutine_scopes
            .get_mut(&current_subroutine_name)
            .unwrap()
            .insert(name, sym);
        st
    }
    fn add_subroutine_symbol_table(&self, subroutine_name: String) -> Self {
        let mut st = self.clone();
        st.subroutine_scopes
            .insert(subroutine_name.clone(), std::collections::HashMap::default());
        st.current_subroutine_name = Some(subroutine_name);
        st
    }
    fn update_current_subroutine_name(&self, subroutine_name: String) -> Self {
        let mut st = self.clone();
        st.current_subroutine_name = Some(subroutine_name);
        st
    }
    fn get(&self, var_name: &str) -> Symbol {
        if let Some(s) = self
            .subroutine_scopes
            .get(&self.current_subroutine_name.clone().unwrap())
            .unwrap()
            .get(var_name)
        {
            Symbol::Subroutine(s.clone())
        } else if let Some(s) = self.class_scope.get(var_name) {
            Symbol::Class(s.clone())
        } else {
            panic!("{:?} not found in {:?}", var_name, self);
        }
    }
    #[allow(dead_code)]
    fn debug_class_symbol_table(&self) {
        println!("class_scope:");
        for v in self.class_scope.values() {
            println!("\t{:?} {:?} {} #{}", v.symbol_type, v.type_, v.name, v.index);
        }
    }
    #[allow(dead_code)]
    fn debug_subroutine_symbol_table(&self, subroutine_name: String) {
        println!("subroutine_scope({}):", subroutine_name);
        for v in self
            .subroutine_scopes
            .get(&self.current_subroutine_name.clone().unwrap())
            .unwrap()
            .values()
        {
            println!("\t{:?} {:?} {} #{}", v.symbol_type, v.type_, v.name, v.index);
        }
    }
    fn get_local_var_count(&self) -> usize {
        self.subroutine_scopes
            .get(&self.current_subroutine_name.clone().unwrap())
            .unwrap()
            .values()
            .filter(|s| s.symbol_type == SubroutineSymbolType::Var)
            .count()
    }
    fn get_field_count(&self) -> usize {
        self.class_scope
            .values()
            .filter(|s| s.symbol_type == ClassSymbolType::Field)
            .count()
    }
}

// symbol_typeをgenericな型として外から受け取るとSubroutineSymbolと構造体定義を共通化できそうにも思えるが
// 不適切な共通化な気がしたのでしていない。(共通化が)必要になったらそのときに検討する。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ClassSymbol {
    name: String,
    type_: Type,
    symbol_type: ClassSymbolType,
    index: usize,
}
#[derive(PartialEq, Eq, Clone, Debug)]
enum ClassSymbolType {
    Static,
    Field,
}
impl ClassSymbolType {
    fn to_segment_name(&self) -> String {
        match self {
            ClassSymbolType::Static => "static",
            ClassSymbolType::Field => "this",
        }
        .to_string()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct SubroutineSymbol {
    name: String,
    type_: Type,
    symbol_type: SubroutineSymbolType,
    index: usize,
}
#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug, Eq)]
enum SubroutineSymbolType {
    Var,
    Arg,
    Pointer,
}
impl SubroutineSymbolType {
    fn to_segment_name(&self) -> String {
        match self {
            SubroutineSymbolType::Var => "local",
            SubroutineSymbolType::Arg => "argument",
            SubroutineSymbolType::Pointer => "pointer",
        }
        .to_string()
    }
}

#[derive(Debug)]
enum Symbol {
    Class(ClassSymbol),
    Subroutine(SubroutineSymbol),
}
impl Symbol {
    fn to_vm(&self) -> String {
        match self {
            Symbol::Class(s) => {
                format!("{} {}", s.symbol_type.to_segment_name(), s.index)
            }
            Symbol::Subroutine(s) => {
                format!("{} {}", s.symbol_type.to_segment_name(), s.index)
            }
        }
    }
    fn push(&self) -> String {
        format!("push {}", self.to_vm())
    }
    fn pop(&self) -> String {
        format!("pop {}", self.to_vm())
    }
    // あるシンボルが任意のclassのインスタンスのとき、その型名を返す
    fn get_class_instance_type(&self) -> Option<ClassName> {
        match self {
            Symbol::Class(c) => match &c.type_ {
                Type::ClassName(c) => Some(ClassName(token::Identifier(c.to_string()))),
                _ => None,
            },
            Symbol::Subroutine(s) => match &s.type_ {
                Type::ClassName(c) => Some(ClassName(token::Identifier(c.to_string()))),
                _ => None,
            },
        }
    }
}

impl Ast {
    pub fn new(tokens: Vec<token::Token>, file_name: String) -> Self {
        let class = match tokens.first() {
            Some(token::Token::Key(token::Keyword::Class)) => match Class::new(&tokens, file_name, 0) {
                Some(class) => class,
                _ => panic!("{}", invalid_token(&tokens, 1)),
            },
            Some(_) => panic!("{}", invalid_token(&tokens, 0)),
            None => panic!("{}", invalid_token(&tokens, 0)),
        };

        Self { class }
    }

    pub fn to_vm(&self) -> String {
        self.class.to_string(&self.class.symbol_tables).join("\n")
    }
}

// シンボルテーブルの仕様メモ
// ## 保持したいデータ
// - name: 変数名
// - category: class_scope(field / static) / subroutine_scope(var / arg / subroutine)
// - index: そのスコープにおけるcategory別の0-originのindex
// - usage:  宣言されているか(field / static / var) / 使用されているか(Jack式に現れる)
//
// ## その他仕様
// - class_scopeのシンボルテーブルとsubroutine_scopeのシンボルテーブルを別々に管理する必要がある
// - subroutineの処理を開始するタイミングでsubroutine_scopeのシンボルテーブルはrefreshされる必要がある

/*
 * プログラムの構造
 */
#[derive(Debug, PartialEq, Eq)]
struct Class {
    name: ClassName,
    var_dec: Vec<ClassVarDec>,
    subroutine_dec: Vec<SubroutineDec>,
    symbol_tables: SymbolTables,
}
impl Class {
    // parse結果を返す。ひとまずindexは返さない
    fn new(tokens: &[token::Token], file_name: String, index: usize) -> Option<Self> {
        let symbol_tables = SymbolTables::default(file_name);
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

        let (var_dec, index, mut symbol_tables) = ClassVarDec::new(tokens, index, symbol_tables);
        let mut subroutine_dec = vec![];
        let mut index = index;
        while let (Some(s), returned_index, returned_symbol_tables) =
            SubroutineDec::new(tokens, index, &name, symbol_tables.clone())
        {
            subroutine_dec.push(s);
            index = returned_index;
            symbol_tables = returned_symbol_tables;
        }

        match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::RightBrace)) => {}
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        // symbol_tables.debug_class_symbol_table();
        Some(Class {
            name,
            var_dec,
            subroutine_dec,
            symbol_tables,
        })
    }

    pub fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        let mut result = vec![];
        // NOTE: 場合によってはクラス変数の初期化処理が必要かもしれない
        // for var_dec in &self.var_dec {
        //     result = [result, var_dec.to_string(&self.symbol_tables)].concat();
        // }
        for subroutine in &self.subroutine_dec {
            result = [result, subroutine.to_string(&self.name, symbol_tables)].concat();
        }

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
    fn new(tokens: &[token::Token], index: usize, mut symbol_tables: SymbolTables) -> (Vec<Self>, usize, SymbolTables) {
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

            for var_name in &var_names {
                // シンボルテーブルを更新
                symbol_tables =
                    symbol_tables.append_class_symbol(var_name.clone().0 .0, type_.clone(), kind.clone().into());
            }
            class_var_decs.push(Self { kind, type_, var_names });
        }

        (class_var_decs, index, symbol_tables)
    }

    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        todo!("実装は不要だと思うが念の為残している");
        // let mut result = vec![];
        // let (open, close) = get_xml_tag("classVarDec".to_string());
        // result.push(open);
        // result.push(self.kind.to_string());
        // result.push(self.type_.to_string());
        // for (index, n) in self.var_names.iter().enumerate() {
        //     if index != 0 {
        //         result.push(to_xml_tag(token::Symbol::Comma));
        //     }
        //     result.push(n.to_string(symbol_tables));
        // }
        // if !&self.var_names.is_empty() {
        //     result.push(to_xml_tag(token::Symbol::SemiColon));
        // }
        //
        // result.push(close);
        // result
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum ClassVarKind {
    Static,
    Field,
}
impl ClassVarKind {
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        todo!();
        // let (open, close) = get_xml_tag("keyword".to_string());
        // format!("{} {} {}", open, format!("{:?}", self).to_lowercase(), close)
    }
}
impl From<ClassVarKind> for ClassSymbolType {
    fn from(kind: ClassVarKind) -> Self {
        match kind {
            ClassVarKind::Static => ClassSymbolType::Static,
            ClassVarKind::Field => ClassSymbolType::Field,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
        todo!();
        // match self {
        //     Type::ClassName(c) => {
        //         todo!();
        //         let (open, close) = get_xml_tag("identifier".to_string());
        //         format!("{} {} {}", open, c, close)
        //     }
        //     _ => {
        //         let (open, close) = get_xml_tag("keyword".to_string());
        //         format!("{} {} {}", open, format!("{:?}", self).to_lowercase(), close)
        //     }
        // }
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
    fn new(
        tokens: &[token::Token],
        index: usize,
        class_name: &ClassName,
        symbol_tables: SymbolTables,
    ) -> (Option<Self>, usize, SymbolTables) {
        let (kind, index) = match SubroutineDecKind::new(tokens, index) {
            (Some(k), i) => (k, i),
            _ => return (None, index, symbol_tables),
        };
        let (type_, index) = SubroutineDecType::new(tokens, index);
        let (subroutine_name, index) = match tokens.get(index) {
            Some(token::Token::Identifier(i)) => (i.clone(), index + 1),
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let symbol_tables = symbol_tables.add_subroutine_symbol_table(subroutine_name.0.clone());

        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::LeftParen)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        // methodだったらThisを追加する
        let symbol_tables = match kind {
            SubroutineDecKind::Method => {
                // メソッドの場合は0番目の引数はthisになる。
                // シンボルテーブルから取得する際はpointerから取得するのでargのthisは取得されない。
                // シンボルテーブルに引数を追加するときに1スタートにする手もあるが、実装の容易性を優先して
                // ダミーのシンボルを追加する方針をとった。
                let symbol_tables = symbol_tables.append_subroutine_symbol(
                    "pseudo_arg_for_receiver".to_string(),
                    Type::ClassName(class_name.0 .0.clone()),
                    SubroutineSymbolType::Arg,
                );
                symbol_tables.append_subroutine_symbol(
                    "this".to_string(),
                    Type::ClassName(class_name.0 .0.clone()),
                    SubroutineSymbolType::Pointer,
                )
            }
            _ => symbol_tables,
        };

        let (parameter_list, index, symbol_tables) = ParameterList::new(tokens, index, symbol_tables);
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::RightParen)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };
        let (body, index, symbol_tables) = SubroutineBody::new(tokens, index, class_name, symbol_tables);

        (
            Some(Self {
                kind,
                type_,
                subroutine_name,
                parameter_list,
                body,
            }),
            index,
            symbol_tables,
        )
    }
    fn to_string(&self, class_name: &ClassName, symbol_tables: &SymbolTables) -> Vec<String> {
        let mut symbol_tables = symbol_tables.update_current_subroutine_name(self.subroutine_name.0.clone());
        let mut result = vec![format!(
            "function {}.{} {}",
            class_name.0.to_string(),
            self.subroutine_name.0,
            symbol_tables.get_local_var_count(),
        )];

        match self.kind {
            SubroutineDecKind::Constructor => {
                // class fieldの初期化
                result = [
                    result,
                    vec![
                        format!("push constant {}", symbol_tables.get_field_count()),
                        "call Memory.alloc 1".to_string(),
                        "pop pointer 0".to_string(),
                    ],
                ]
                .concat();
                symbol_tables = symbol_tables.append_subroutine_symbol(
                    "this".to_string(),
                    Type::ClassName(class_name.0 .0.clone()),
                    SubroutineSymbolType::Pointer,
                );
            }
            SubroutineDecKind::Function => {}
            SubroutineDecKind::Method => {
                // 最初の引数をThisにセット
                result = [result, vec!["push argument 0".to_string(), "pop pointer 0".to_string()]].concat();
            }
        };
        result = [result, self.body.to_string(&symbol_tables)].concat();
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
    fn to_string(&self) -> String {
        todo!();
        // let (open, close) = get_xml_tag("keyword".to_string());
        // format!("{} {} {}", open, format!("{:?}", self).to_lowercase(), close)
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
    fn to_string(&self) -> String {
        todo!();
        // match self {
        //     SubroutineDecType::Void => {
        //         let (open, close) = get_xml_tag("keyword".to_string());
        //         format!("{} {} {}", open, format!("{:?}", self).to_lowercase(), close)
        //     }
        //     SubroutineDecType::Type_(t) => t.to_string(),
        // }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParameterList(Vec<(Type, VarName)>);
impl ParameterList {
    // パターンメモ
    // ``: 引数なし
    // `type var_name, type var_name, ..., type var_name`: n個の引数
    fn new(tokens: &[token::Token], index: usize, mut symbol_tables: SymbolTables) -> (Self, usize, SymbolTables) {
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

            param_list.push((type_.clone(), var_name.clone()));
            symbol_tables = symbol_tables.append_subroutine_symbol(var_name.0 .0, type_, SubroutineSymbolType::Arg);

            // `,`があるときだけ継続
            match tokens.get(index) {
                Some(token::Token::Sym(token::Symbol::Comma)) => {
                    index += 1;
                }
                _ => break,
            }
        }

        (Self(param_list), index, symbol_tables)
    }
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        todo!();
        // let mut result = vec![];
        // let (open, close) = get_xml_tag("parameterList".to_string());
        // result.push(open);
        // for (index, p) in self.0.iter().enumerate() {
        //     if index != 0 {
        //         result.push(to_xml_tag(token::Symbol::Comma));
        //     }
        //     result.push(p.0.to_string());
        //     result.push(p.1.to_string(symbol_tables));
        // }
        // result.push(close);
        // result
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SubroutineBody {
    var_dec: Vec<VarDec>,
    statements: Statements,
}
impl SubroutineBody {
    fn new(
        tokens: &[token::Token],
        index: usize,
        class_name: &ClassName,
        mut symbol_tables: SymbolTables,
    ) -> (Self, usize, SymbolTables) {
        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::LeftBrace)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        let mut var_dec = vec![];
        let mut index = index;
        while let (Some(got), returned_index, returned_symbol_tables) =
            VarDec::new(tokens, index, symbol_tables.clone())
        {
            var_dec.push(got);
            index = returned_index;
            symbol_tables = returned_symbol_tables;
        }

        let (statements, index) = Statements::new(tokens, index, class_name);

        let index = match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::RightBrace)) => index + 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        (Self { var_dec, statements }, index, symbol_tables)
    }
    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        let mut result = vec![];
        if !&self.statements.0.is_empty() {
            for s in &self.statements.0 {
                result = [result, s.to_string(symbol_tables)].concat();
            }
        }
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
struct VarDec {
    type_: Type,
    var_name: Vec<VarName>,
}
impl VarDec {
    fn new(
        tokens: &[token::Token],
        index: usize,
        mut symbol_tables: SymbolTables,
    ) -> (Option<Self>, usize, SymbolTables) {
        let mut var_name = vec![];
        let index = match tokens.get(index) {
            Some(token::Token::Key(token::Keyword::Var)) => index + 1,
            _ => return (None, index, symbol_tables),
        };

        let (type_, index) = match Type::new(tokens, index) {
            (Some(t), i) => (t, i),
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        let mut index = match tokens.get(index) {
            Some(token::Token::Identifier(token::Identifier(id))) => {
                var_name.push(VarName(token::Identifier(id.clone())));
                symbol_tables =
                    symbol_tables.append_subroutine_symbol(id.clone(), type_.clone(), SubroutineSymbolType::Var);
                index + 1
            }
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        while let Some(token::Token::Sym(token::Symbol::Comma)) = tokens.get(index) {
            index += 1;

            let var_name_new = match tokens.get(index) {
                Some(token::Token::Identifier(i)) => {
                    index += 1;
                    VarName(token::Identifier(i.clone().0))
                }
                _ => panic!("{}", invalid_token(tokens, index)),
            };

            var_name.push(var_name_new.clone());
            symbol_tables =
                symbol_tables.append_subroutine_symbol(var_name_new.0 .0, type_.clone(), SubroutineSymbolType::Var);
        }

        // 最後にセミコロンがあることをチェック
        match tokens.get(index) {
            Some(token::Token::Sym(token::Symbol::SemiColon)) => index += 1,
            _ => panic!("{}", invalid_token(tokens, index)),
        };

        (Some(Self { type_, var_name }), index, symbol_tables)
    }
    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        todo!("実装は不要だと思うが念の為残している");
        // let mut result = vec![];
        // let (open, close) = get_xml_tag("varDec".to_string());
        // result.push(open);
        // result.push(to_xml_tag(token::Keyword::Var));
        // result.push(self.type_.to_string());
        // for (index, n) in self.var_name.iter().enumerate() {
        //     if index != 0 {
        //         result.push(to_xml_tag(token::Symbol::Comma));
        //     }
        //     result.push(n.to_string(symbol_tables));
        // }
        // if !&self.var_name.is_empty() {
        //     result.push(to_xml_tag(token::Symbol::SemiColon));
        // }
        //
        // result.push(close);
        // result
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

#[derive(Debug, PartialEq, Eq, Clone)]
struct SubroutineName(token::Identifier);
#[derive(Clone, Debug, PartialEq, Eq)]
struct VarName(token::Identifier);
impl VarName {
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self, symbol_tables: &SymbolTables) -> String {
        let symbol = symbol_tables.get(&self.0 .0);
        symbol.push()
    }
}

/*
 * 文
 */
#[derive(Debug, PartialEq, Eq, Clone)]
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

// labelの一意性を担保するためのsuffixを生成するための関数。
// randomなので厳密には衝突する可能性があり、一意性を必ずしも担保できているとはいえないが、
// 実装量をセーブするためにそこは妥協する。
fn gen_random_6_characters_str() -> String {
    let mut rng = rand::rng();
    let mut v: Vec<u8> = (b'a'..=b'z').collect();
    v.shuffle(&mut rng);
    v = v[0..6].to_vec();
    String::from_utf8(v).unwrap()
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        match self {
            Statement::Let(s) => s.to_string(symbol_tables),
            Statement::If(s) => s.to_string(symbol_tables),
            Statement::While(s) => s.to_string(symbol_tables),
            Statement::Do(s) => s.to_string(symbol_tables),
            Statement::Return(s) => s.to_string(symbol_tables),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        match &self.array_index {
            Some(a) => {
                // 左辺を評価してpush(var_nameのアドレス + 添字)
                let mut result = [
                    vec![symbol_tables.get(&self.var_name.0 .0).push()],
                    a.to_string(symbol_tables),
                    vec!["add".to_string()],
                ]
                .concat();

                // 右辺を評価してpush
                result = [result, self.right_hand_side.to_string(symbol_tables)].concat();

                // 右辺をtemp 0にpop
                result.push("pop temp 0".to_string());
                // 左辺をpointer 1にpop
                result.push("pop pointer 1".to_string());
                // temp 0をthat 0にpop
                result.push("push temp 0".to_string());
                result.push("pop that 0".to_string());

                // NOTE:
                // ## pointer 0とthat 0の違い
                // pointer segmentにはthisとthatのベースアドレスが格納されている。
                // that 0を操作すると実際の値を操作することができる。pointer 0やpointer
                // 1にはあくまでどこにTHISとTHATの実体が位置しているかを指しているだけ。
                result
            }
            None => {
                let right = self.right_hand_side.to_string(symbol_tables);
                let mut result = right;

                let left = symbol_tables.get(&self.var_name.0 .0);
                result.push(left.pop());
                result
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        let (negative_case_label_name, end_if_statement_label_name) = {
            let label_base = format!(
                "{}.{}:{}",
                symbol_tables.file_name,
                symbol_tables.current_subroutine_name.clone().unwrap(),
                gen_random_6_characters_str()
            );

            (format!("{}_if_start", label_base), format!("{}_if_end", label_base))
        };

        let mut result = self.condition.to_string(symbol_tables);
        result.push("not".to_string());
        result.push(format!("if-goto {}", negative_case_label_name));

        for statement in &self.positive_case_body.0 {
            result = [result, statement.to_string(symbol_tables)].concat();
        }
        result.push(format!("goto {}", end_if_statement_label_name));

        result.push(format!("label {}", negative_case_label_name));
        if let Some(negative_case_body) = &self.negative_case_body {
            for statement in &negative_case_body.0 {
                result = [result, statement.to_string(symbol_tables)].concat();
            }
        }

        result.push(format!("label {}", end_if_statement_label_name));
        result
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        let (start_label_name, end_label_name) = {
            let label_base = format!(
                "{}.{}:{}",
                symbol_tables.file_name,
                symbol_tables.current_subroutine_name.clone().unwrap(),
                gen_random_6_characters_str()
            );

            (format!("{}_while_start", label_base), format!("{}_while_end", label_base))
        };

        let mut result = vec![format!("label {}", start_label_name)];

        result = [result, self.condition.to_string(symbol_tables)].concat();
        result.push("not".to_string());
        result.push(format!("if-goto {}", end_label_name));

        for statement in &self.body.0 {
            result = [result, statement.to_string(symbol_tables)].concat();
        }
        result.push(format!("goto {}", start_label_name));

        result.push(format!("label {}", end_label_name));
        result
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        let mut result = self.0.to_string(symbol_tables);
        result.push("pop temp 0".to_string()); // do statementは返り値を無視するためpopする必要がある
        result
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        match &self.0 {
            Some(e) => {
                let mut result = e.to_string(symbol_tables);
                if !result.is_empty() {
                    result.push("return".to_string());
                }
                result
            }
            None => vec!["push constant 0".to_string(), "return".to_string()],
        }
    }
}

/*
 * 式
 */
#[derive(Debug, PartialEq, Eq, Clone)]
struct Expression {
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
    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        let mut result = self.term.to_string(symbol_tables);
        for o in &self.op_term {
            result = [result, o.1.to_string(symbol_tables)].concat();
            result.push(o.0.to_string());
        }

        result
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        match self {
            Term::IntegerConstant(s) => vec![s.to_string()],
            Term::StringConstant(s) => s.to_string(),
            Term::KeyWordConstant(s) => s.to_string(),
            Term::VarName(s) => vec![s.to_string(symbol_tables)],
            Term::ArrayIndexAccess(v, e) => {
                // vとeのアドレスを足し合わせる
                let mut result = vec![v.to_string(symbol_tables)];
                result = [result, e.to_string(symbol_tables)].concat();
                result.push("add".to_string());
                result.push("pop pointer 1".to_string()); // 計算したアドレスをポインタにセット
                result.push("push that 0".to_string()); // そのアドレスの値を取得

                result
            }
            Term::Expression(s) => s.to_string(symbol_tables),
            Term::UnaryOp(u, t) => {
                let mut result = t.to_string(symbol_tables);
                result.push(u.to_string());
                result
            }
            Term::SubroutineCall(s) => s.to_string(symbol_tables),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum KeyWordConstant {
    True,
    False,
    Null,
    This,
}
impl KeyWordConstant {
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> Vec<String> {
        match self {
            KeyWordConstant::True => vec!["push constant 0", "not"],
            KeyWordConstant::False => vec!["push constant 0"],
            KeyWordConstant::Null => vec!["push constant 0"],
            KeyWordConstant::This => vec!["push pointer 0"],
        }
        .into_iter()
        .map(|e| e.to_string())
        .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct SubroutineCall {
    // FIXME: VarNameしかこないはずなのでリファクタできそう。
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
    /// method: {class型の変数}.foo() || foo()
    /// {class型の変数}はsymbol_tablesに格納されている
    /// おそらく小文字スタート
    /// constructor: {class名}.new
    /// function:{大文字始まり}.foo()
    fn which_subroutine_kind(&self) -> SubroutineDecKind {
        match &self.receiver {
            Some(r) => {
                if r.to_string().chars().next().map_or(false, char::is_lowercase) {
                    SubroutineDecKind::Method
                } else if self.name.0 .0.as_str() == "new" {
                    SubroutineDecKind::Constructor
                } else {
                    SubroutineDecKind::Function
                }
            }
            None => SubroutineDecKind::Method,
        }
    }
    fn to_string(&self, symbol_tables: &SymbolTables) -> Vec<String> {
        match self.which_subroutine_kind() {
            SubroutineDecKind::Constructor | SubroutineDecKind::Function => {
                let mut result = vec![];
                for a in &self.arguments.0 {
                    result = [result, a.to_string(symbol_tables)].concat();
                }

                // e.g. call Foo.Bar 2
                result.push(format!(
                    "call {}.{} {}",
                    self.receiver.clone().unwrap().to_string(),
                    self.name.0.to_string(),
                    self.arguments.0.len()
                ));
                result
            }
            SubroutineDecKind::Method => {
                let receiver_symbol_name = match &self.receiver {
                    Some(Receiver::ClassName(_)) => {
                        panic!("このパターンは存在しないはず")
                    }
                    Some(Receiver::VarName(v)) => v.0 .0.clone(),
                    // メソッド内でのメソッド呼び出しでthisが省略されているケース
                    None => "this".to_string(),
                };

                // レシーバをpush
                let mut result = vec![format!("push {}", symbol_tables.get(&receiver_symbol_name).to_vm())];

                // 残りの引数をすべてpush
                for a in &self.arguments.0 {
                    result = [result, a.to_string(symbol_tables)].concat();
                }

                // call foo.Bar n+1
                result.push(format!(
                    "call {}.{} {}",
                    symbol_tables
                        .get(&receiver_symbol_name)
                        .get_class_instance_type()
                        .unwrap()
                        .0
                         .0,
                    self.name.0.to_string(),
                    self.arguments.0.len() + 1
                ));
                result
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
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
        match self {
            Op::Plus => "add".to_string(),
            Op::Minus => "sub".to_string(),
            Op::Multiply => "call Math.multiply 2".to_string(),
            Op::Div => "call Math.divide 2".to_string(),
            Op::Ampersand => "and".to_string(),
            Op::Pipe => "or".to_string(),
            Op::LessThan => "lt".to_string(),
            Op::MoreThan => "gt".to_string(),
            Op::Equal => "eq".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
        match self {
            UnaryOp::Minus => "neg",
            UnaryOp::Tilde => "not",
        }
        .to_string()
    }
}

fn invalid_token(tokens: &[token::Token], index: usize) -> String {
    format!("invalid token {:?}[@{}]", tokens, index)
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
            "Main".to_string(),
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
            symbol_tables: SymbolTables {
                class_scope: {
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        "square".to_string(),
                        ClassSymbol {
                            name: "square".to_string(),
                            type_: Type::ClassName("Square".to_string()),
                            symbol_type: ClassSymbolType::Field,
                            index: 0,
                        },
                    );
                    map.insert(
                        "direction".to_string(),
                        ClassSymbol {
                            name: "direction".to_string(),
                            type_: Type::Int,
                            symbol_type: ClassSymbolType::Field,
                            index: 1,
                        },
                    );
                    map
                },
                subroutine_scopes: std::collections::HashMap::new(),
                current_subroutine_name: None,
                file_name: "Main".to_string(),
            },
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
            SymbolTables::default("Main".to_string()),
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
        assert_eq!(input.0, expected.0);
        assert_eq!(input.1, expected.1);
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
            SymbolTables::default("Main".to_string()),
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
        assert_eq!(input.0, expected.0);
        assert_eq!(input.1, expected.1);

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
            SymbolTables::default("Main".to_string()),
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
        assert_eq!(input.0, expected.0);
        assert_eq!(input.1, expected.1);

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
            SymbolTables::default("Main".to_string()),
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
        assert_eq!(input.0, expected.0);
        assert_eq!(input.1, expected.1);
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
            SymbolTables::default("Main".to_string()),
        );
        let expected = (
            Some(VarDec {
                type_: Type::ClassName("MyType".to_string()),
                var_name: vec![VarName(token::Identifier("foo".to_string()))],
            }),
            4,
        );
        assert_eq!(input.0, expected.0);
        assert_eq!(input.1, expected.1);

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
            SymbolTables::default("Main".to_string()),
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
        assert_eq!(input.0, expected.0);
        assert_eq!(input.1, expected.1);
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
            SymbolTables::default("Main".to_string()),
        );
        let expected = (
            ParameterList(vec![
                (Type::Int, VarName(token::Identifier("x".to_string()))),
                (Type::Char, VarName(token::Identifier("y".to_string()))),
            ]),
            5,
        );
        assert_eq!(input.0, expected.0);
        assert_eq!(input.1, expected.1);

        /*
            (引数なし)
        */
        let input = ParameterList::new(&[], 0, SymbolTables::default("Main".to_string()));
        let expected = (ParameterList(vec![]), 0);
        assert_eq!(input.0, expected.0);
        assert_eq!(input.1, expected.1);
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
            SymbolTables::default("Main".to_string()),
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
        assert_eq!(input.0, expected.0);
        assert_eq!(input.1, expected.1);

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
            SymbolTables::default("Main".to_string()),
        );
        let expected = (
            SubroutineBody {
                var_dec: vec![],
                statements: Statements(vec![]),
            },
            2,
        );
        assert_eq!(input.0, expected.0);
        assert_eq!(input.1, expected.1);
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
