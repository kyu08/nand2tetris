## TODO
- [x] トークナイザを実装
- [ ] トークナイザのテストをパスすることを確認
    - [x] `test_data/ExpressionLessSquare/SquareT.xml`
        ```sh
        cargo run test_data/ExpressionLessSquare/ \
            && diff -w -B test_data/ExpressionLessSquare/MainT.xml test_data/ExpressionLessSquare/Main.gen.xml \
            && diff -w -B test_data/ExpressionLessSquare/SquareT.xml test_data/ExpressionLessSquare/Square.gen.xml \
            && diff -w -B test_data/ExpressionLessSquare/SquareGameT.xml test_data/ExpressionLessSquare/SquareGame.gen.xml 
        ```
    - [ ] `test_data/ExpressionLessSquare/SquareGameT.xml`
    - [ ] `test_data/ExpressionLessSquare/MainT.xml`
    - [ ] `test_data/ArrayTest/MainT.xml`
    - [ ] `test_data/Square/SquareT.xml`
    - [ ] `test_data/Square/SquareGameT.xml`
    - [ ] `test_data/Square/MainT.xml`
- [ ] コンパイルエンジンを実装
- [ ] コンパイルエンジンのテストをパスすることを確認
    - [ ] `test_data/ExpressionLessSquare/Main.xml`
    - [ ] `test_data/ExpressionLessSquare/SquareGame.xml`
    - [ ] `test_data/ExpressionLessSquare/Square.xml`
    - [ ] `test_data/ArrayTest/Main.xml`
    - [ ] `test_data/Square/Main.xml`
    - [ ] `test_data/Square/SquareGame.xml`
    - [ ] `test_data/Square/Square.xml`
