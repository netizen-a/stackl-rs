[section .text]
[global _start]
_start:
    push 10   ; stack[0] = 10
loop:
    push hw
    outs
    pop
    push 1    ; stack[1] = 1
    sub       ; stack[0] = stack[0] - stack[1]
    dup       ; stack[1] = stack[0]
    push 0    ; stack[2] = 0
    eq        ; stack[2] = stack[1] == stack[2]
    jz loop   ; if stack[2] == 0 then goto loop;
    halt
[section .data]
    hw dd 'hello world!\n',0
    