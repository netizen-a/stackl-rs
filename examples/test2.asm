[global _start]
_start:
    jmp print
exit:
    halt
print:
    push in
    inp
    jmp wait
[section .data]
    in dd 3, hw, 0
    hw dd 'Hello World!\n',0
[section .text]
wait:
    push 100000
loop:
    push 1
    sub
    dup
    push 0
    eq
    jz loop
    jmp exit
