pub struct Tokenizer {
    // tokenizeをしながら順次ここに追加する
    token: Vec<Token>,
    current_index: u32,
}

impl Tokenizer {
    pub fn new(source_code: String) -> Self {
        for char in source_code.chars() {
            if char.is_whitespace() {
                continue;
            }
            println!("{}", char);
        }
        todo!();
        // ファイル内容の文字を1文字ずつイテレート
        // トークンが見つかったらparse結果を格納するデータ構造に格納
        // EOFまでいったらparse結果をxmlとして出力する
        let token = vec![];

        // TODO: コメント継続中と空行は無視する
        // `"`の中は文字列定数として扱う必要があるので注意
        // 文字を空白~空白までをtokenとして解釈する。ただし途中でsymbolが見つかった場合はその1つ前までをtokenとする。
        //
        let current_index = 0;
        Self { token, current_index }
    }

    pub fn toXml(&self) -> String {
        // TODO: インデントをどうするか問題をあとで考える
        let result: String = "".to_string();
        result
    }
}

enum Token {
    a,
    b,
}

struct CompilationEngine {
    // Tokenizerからわたってきた字句解析結果
    token: Vec<Token>,
    // TODO: その他必要な状態を持たせる
}
