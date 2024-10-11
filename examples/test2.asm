[global _start]
_start:
    jmp print
exit:
    halt
print:
    push in
    inp
loop:
    push in
    pushvarind
    push 0x80000000
    ne
    jz exit
    jmp loop
[section .data]
    in dd 3, hw, 0
    hw dd 'Hello World!\n',0
