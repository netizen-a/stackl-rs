[global _start]
_start:
    push 3000
    popreg bp
    push 4000
    popreg lp
    push io_blk
    inp
    nop
    call wait_io_blk
    jmpuser 3000
    halt

wait_io_blk:
    push io_blk
    pushvarind
    push 0x80000000
    eq
    jz wait_io_blk
    ret

[section .data]
    io_blk: dd 8, prog_path, 0
    prog_path: dd './target/test.stackl',0