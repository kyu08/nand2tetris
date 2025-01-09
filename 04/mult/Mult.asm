// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/4/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
// The algorithm is based on repetitive addition.

// 0に対して(R0-1)回R1を足すことで掛け算を実現する

  // n = 0
  @n
  M=0

  // R2 = 0
  @R2
  M=0
(LOOP)
  // if n == R0 then goto END
  @n
  D=M
  @R0
  D=D-M
  @END
  D;JEQ

  // R2 += R1
  @R2
  D=M

  @R1
  D=D+M

  // R2 = D
  @R2
  M=D

  // n++
  @n
  M=M+1

  // go to LOOP
  @LOOP
  0;JMP
(END)
  @END
  0;JMP
