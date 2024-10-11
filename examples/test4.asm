[global _start]
_start:
    call geti
    call print
exit:
    halt
print:
    push buf   ; stack[0] = 10
    pushvarind
    dup
    jz exit    ; if buf == 0 then jump to exit
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
    ret
geti:
    push io_blk_read
    inp
geti_wait:
    push io_blk_read
    pushvarind
    push 0x80000000
    eq
    jz geti_wait
    ret

exit_loop:
    ret

[section .data]
    io_blk_hw dd 3, buf, 0
    io_blk_read dd 7, buf, 0
    buf dd 0
    hw dd 'Hello World!\n',0

