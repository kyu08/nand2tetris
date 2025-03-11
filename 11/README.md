## TODO
- [x] 第10章のコードをコピー
- [x] シンボルテーブルに変数を加える実装
   	- [x] 実装する
   	- [x] コード出力時にシンボルテーブルの情報を出力する(field, static, var, arg, class, subroutine)
   	- [x] subroutineってsymbol_tableに追加する必要はあるんだっけ？	-> なさそうなのでひとまずスルー
   	- [x] symbol_tableの実装がうまく動いてそうなことを確認
- [x] コード生成の実装
   	- [x] 実装する
   	- [x] テスト
       	- [x] Seven
           	- [x] 式をコンパイル
           	- [x] do statementをコンパイル
           	- [x] subroutineをコンパイル
           	- [x] 不要なxmlを削除
       	- [x] ConvertToBin
       	- [x] Square
            - `Out of segment space in Square.new.10`というエラーが出て終了する
            - [x] 以下もデグレしてないか改めて確認
                - [x] Seven
                - [x] ConvertToBin
       	- [x] Average
            - [x] 文字列のコンパイルを実装
            - [x] 配列のコンパイルを実装
            - [x] テストのパスを確認
       	- [x] Pong
       	- [x] ComplexArrays


# 現在処理中のsubroutineの`symbol_table`をどう管理するを考えたときのメモ
## 1. currentに今処理中のsymbol_tableを持つ案
- pros
   	- 呼び出し側はcurrent_subroutineを読んで見つからなかったらclass_scopeを読みに行けばよいという意味でシンプル
- cons
   	- currentの更新作業が必要になる。
- 更新するタイミングは？
   	- subroutineの終わりにNoneに変更 & 次のsubroutineの先頭でSome(symbol_table)で更新する

## 2. mapのmapにする案
- pros
	- currentの更新が不要な点でシンプル
-  cons
	- VarName::to_string()とかにもcurrent_subroutine_nameを渡す必要が生じる。VarNameはClassVarDecでも呼ばれるのでOption<String>で
 渡す必要がある。複雑になるしこの構造に起因してバグが起きそう。

## 3. subroutineの変わり目で必要なsymbol_tableだけ渡す
- pros
	- 呼び出し側から見ると他のsubroutineのsymbol_tableが見えないのでバグが起きづらい
- cons
	- Class.symbol_tablesとは別のデータ構造を用意する必要がある。

## 結論
`1. currentに今処理中のsymbol_tableを持つ案`が一番バグが起きづらそう。
