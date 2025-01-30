## TODO
- [x] トークナイザを実装
- [x] トークナイザのテストをパスすることを確認
    - [x] `test_data/ExpressionLessSquare/SquareT.xml`
        ```sh
        cargo run test_data/ExpressionLessSquare/ \
            && diff -w -B test_data/ExpressionLessSquare/MainT.xml test_data/ExpressionLessSquare/Main.gen.xml \
            && diff -w -B test_data/ExpressionLessSquare/SquareT.xml test_data/ExpressionLessSquare/Square.gen.xml \
            && diff -w -B test_data/ExpressionLessSquare/SquareGameT.xml test_data/ExpressionLessSquare/SquareGame.gen.xml 
        ```
    - [x] `test_data/ExpressionLessSquare/SquareGameT.xml`
    - [x] `test_data/ExpressionLessSquare/MainT.xml`
    - [x] `test_data/ArrayTest/MainT.xml`
        ```sh
        cargo run test_data/ArrayTest/ \
            && diff -w -B test_data/ArrayTest/MainT.xml test_data/ArrayTest/Main.gen.xml
        ```
    - [x] `test_data/Square/SquareT.xml`
        ```sh
        cargo run test_data/Square/ \
            && diff -w -B test_data/Square/MainT.xml test_data/Square/Main.gen.xml \
            && diff -w -B test_data/Square/SquareT.xml test_data/Square/Square.gen.xml \
            && diff -w -B test_data/Square/SquareGameT.xml test_data/Square/SquareGame.gen.xml 
        ```
    - [x] `test_data/Square/SquareGameT.xml`
    - [x] `test_data/Square/MainT.xml`
- [ ] コンパイルエンジンを実装(式と配列以外)
    - [x] `ExpressionList`
    - [x] `SubroutineCall`
    - [x] `Term`
    - [x] `Expression`
    - [x] `LetStatement`
    - [x] `IfStatement`
    - [x] `WhileStatement`
    - [x] `DoStatement`
    - [x] `ReturnStatement`
    - [x] `Statement`
        - [x] `LetStatement`
        - [x] `IfStatement`
        - [x] `WhileStatement`
        - [x] `DoStatement`
        - [x] `ReturnStatement`
    - [x] `Statements`
    - [x] `SubroutineBody`
    - [ ] `SubroutineDec`
    - [ ] `Class`
- [ ] 以下のテストをパスすることを確認
    - [ ] `test_data/ExpressionLessSquare/Main.xml`
    - [ ] `test_data/ExpressionLessSquare/SquareGame.xml`
    - [ ] `test_data/ExpressionLessSquare/Square.xml`
- [ ] コンパイルエンジンの残りの部分を実装(式と配列)
- [ ] 以下のテストをパスすることを確認
    - [ ] `test_data/ArrayTest/Main.xml`
    - [ ] `test_data/Square/Main.xml`
    - [ ] `test_data/Square/SquareGame.xml`
    - [ ] `test_data/Square/Square.xml`
