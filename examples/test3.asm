[global _start]
_start:
    call readline
    call print
exit:
    halt
print:
    push io_blk_hw
    inp
print_wait:
    push io_blk_hw
    pushvarind
    push -2147483648
    eq
    jz print_wait
    ret
readline
    push io_blk_read
    inp
readline_wait:
    push io_blk_read
    pushvarind
    push -2147483648
    eq
    jz readline_wait
    ret

exit_loop:
    ret

[section .data]
    io_blk_hw dd 3, buf, 0
    io_blk_read dd 6, buf, 0
    buf db dup 256 (0)

