.global main
.extern printf
.align 8
.data
fmt: .asciz "%d\n"
.text
func_1:
    push {fp, lr}
    mov fp, sp
    sub sp, sp, #88
    b block_1
block_2:
    ldr r0, [fp, #-32]
    str r0, [fp, #-72]
     ldr r0, =10
    str r0, [fp, #-48]
    ldr r0, [fp, #-72]
    mov r1, r0
    ldr r0, [fp, #-48]
    cmp r1, r0
    mov r0, #0
    movlt r0, #1
    str r0, [fp, #-64]
    ldr r0, [fp, #-64]
    cmp r0, #0
    beq block_3
    b block_4
block_3:
    ldr r0, [fp, #-32]
    str r0, [fp, #-24]
     ldr r1, [fp, #-24]
    bl func_0
    str r0, [fp, #-8]
    ldr r0, [fp, #-8]
    mov r1, r0
    ldr r0, =fmt
    bl printf
    ldr r0, [fp, #-32]
    str r0, [fp, #-16]
     ldr r0, =1
    str r0, [fp, #-56]
    ldr r0, [fp, #-16]
    mov r1, r0
    ldr r0, [fp, #-56]
    add r0, r1, r0
    str r0, [fp, #-40]
    ldr r0, [fp, #-40]
    str r0, [fp, #-32]
    b block_2
block_1:
     ldr r0, =0
    str r0, [fp, #-80]
    ldr r0, [fp, #-80]
    str r0, [fp, #-32]
    b block_2
block_4:
    b ret_1
ret_1:
    add sp, sp, #88
    pop {fp, lr}
    bx lr
func_0:
    push {fp, lr}
    mov fp, sp
    sub sp, sp, #40
    str r1, [fp, #-16]
    b block_0
block_0:
     ldr r0, =2
    str r0, [fp, #-32]
    ldr r0, [fp, #-16]
    str r0, [fp, #-8]
    ldr r0, [fp, #-32]
    mov r1, r0
    ldr r0, [fp, #-8]
    mul r0, r1, r0
    str r0, [fp, #-24]
    ldr r0, [fp, #-24]
    b ret_0
ret_0:
    add sp, sp, #40
    pop {fp, lr}
    bx lr
main:
    push {lr}
    bl func_1
    pop {lr}
    bx lr
