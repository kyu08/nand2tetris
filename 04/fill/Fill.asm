// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/4/Fill.asm

// Runs an infinite loop that listens to the keyboard input. 
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel. When no key is pressed, 
// the screen should be cleared.

(HANDLE_KEYBOARD)
  // 画面サイズは横512*縦256
  // == 32ワード*16ワード
  @i
  M=0

  @KBD
  D=M
  @MAKE_SCREEN_BLACK
  D;JGT
  @MAKE_SCREEN_WHITE
  0;JMP

  (MAKE_SCREEN_BLACK)
    // i == 8192ならば画面を塗りつぶす処理が最後まで完了しているはずなので
    // 内側のループを抜ける

    // NOTE: 本来は@8192にすべきだが、動作が遅すぎて検証できないので@5にしている
    @5
    // @8192
    D=A
    @i
    D=D-M
    @HANDLE_KEYBOARD
    D;JEQ

    // SCREEN + i番目のワードを111...111に設定する
    @i
    D=M

    @SCREEN
    A=A+D
    M=-1

    // i++
    @i
    M=M+1

    @MAKE_SCREEN_BLACK
    0;JMP

  (MAKE_SCREEN_WHITE)
    // i == 8192ならば画面を塗りつぶす処理が最後まで完了しているはずなので
    // 内側のループを抜ける

    @5
    // @8192
    D=A
    @i
    D=D-M
    @HANDLE_KEYBOARD
    D;JEQ

    // SCREEN + i番目のワードを111...111に設定する

    @i
    D=M

    @SCREEN
    A=A+D
    M=0

    // i++
    @i
    M=M+1

    @MAKE_SCREEN_WHITE
    0;JMP

  @HANDLE_KEYBOARD
  0;JMP
