[global _start]
[systrap systrap]
_start:
    push 3000
    popreg bp
    push 4000
    popreg lp
    push io_blk
    inp
    call wait_io_blk
    jmpuser 0
    halt

wait_io_blk:
    push io_blk
    pushvarind
    push 0x80000000
    eq
    jz wait_io_blk
    ret

systrap:
    pushvar -12
    pushreg bp
    add
    outs
    pop
    rti
[section .data]
    io_blk: dd 8, prog_path, 0
    prog_path: dd './target/user.stackl',0