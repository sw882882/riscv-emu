  .section .text
  .globl _start
_start:
  li x5, 3
  li x6, 0
loop:
  addi x6, x6, 1
  addi x5, x5, -1
  bne  x5, x0, loop
done:
  j done

