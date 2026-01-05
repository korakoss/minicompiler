.global main
.extern printf
.align 4
.data
fmt: .asciz "%d\n"
.text
func_1:
    push {fp, lr}
    mov fp, sp
    sub sp, sp, #88
    b block_0
block_0:
     ldr r0, #1
    str r0, [fp, #-64]
    ldr r0, [fp, #-64]
    str r0, [fp, #-0]
    b block_1
block_4:
    ldr r0, [fp, #-0]
    str r0, [fp, #-8]
     ldr r1, [fp, #-8]
    bl func_0
    str r0, [fp, #-48]
    ldr r0, [fp, #-48]
    cmp r0, #0
    beq block_6
    b block_5
block_6:
    ldr r0, [fp, #-0]
    str r0, [fp, #-16]
    ldr r0, [fp, #-16]
    mov r1, r0
    ldr r0, =fmt
    bl printf
    b block_5
block_2:
    b block_4
block_1:
    ldr r0, [fp, #-0]
    str r0, [fp, #-80]
     ldr r0, #32
    str r0, [fp, #-32]
    ldr r0, [fp, #-80]
    mov r1, r0
    ldr r0, [fp, #-32]
    cmp r1, r0
    mov r0, #0
    movlt r0, #1
    str r0, [fp, #-40]
    ldr r0, [fp, #-40]
    cmp r0, #0
    beq block_2
    b block_3
block_5:
    ldr r0, [fp, #-0]
    str r0, [fp, #-24]
     ldr r0, #1
    str r0, [fp, #-72]
    ldr r0, [fp, #-24]
    mov r1, r0
    ldr r0, [fp, #-72]
    add r0, r1, r0
    str r0, [fp, #-56]
    ldr r0, [fp, #-56]
    str r0, [fp, #-0]
    b block_1
    add sp, sp, #88
    pop {fp, lr}
    bx lr
func_0:
    push {fp, lr}
    mov fp, sp
    sub sp, sp, #192
    str r1, [fp, #-80]
    b block_7
block_9:
    b block_11
block_19:
    ldr r0, =0
    str r0, [fp, #-32]
    ldr r0, [fp, #-32]
    bx lr
block_16:
    ldr r0, =1
    str r0, [fp, #-176]
    ldr r0, [fp, #-176]
    bx lr
block_10:
    ldr r0, =0
    str r0, [fp, #-24]
    ldr r0, [fp, #-24]
    bx lr
block_11:
    ldr r0, [fp, #-80]
    str r0, [fp, #-168]
     ldr r0, #2
    str r0, [fp, #-8]
    ldr r0, [fp, #-168]
    mov r1, r0
    ldr r0, [fp, #-8]
    cmp r1, r0
    mov r0, #0
    moveq r0, #1
    str r0, [fp, #-184]
    ldr r0, [fp, #-184]
    cmp r0, #0
    beq block_13
    b block_12
block_14:
    ldr r0, [fp, #-16]
    str r0, [fp, #-88]
    ldr r0, [fp, #-80]
    str r0, [fp, #-160]
    ldr r0, [fp, #-88]
    mov r1, r0
    ldr r0, [fp, #-160]
    cmp r1, r0
    mov r0, #0
    movlt r0, #1
    str r0, [fp, #-120]
    ldr r0, [fp, #-120]
    cmp r0, #0
    beq block_15
    b block_16
block_13:
    ldr r0, =1
    str r0, [fp, #-112]
    ldr r0, [fp, #-112]
    bx lr
block_15:
    b block_17
block_12:
     ldr r0, #2
    str r0, [fp, #-96]
    ldr r0, [fp, #-96]
    str r0, [fp, #-16]
    b block_14
block_8:
    ldr r0, [fp, #-80]
    str r0, [fp, #-56]
     ldr r0, #2
    str r0, [fp, #-104]
    ldr r0, [fp, #-56]
    mov r1, r0
    ldr r0, [fp, #-104]
    cmp r1, r0
    mov r0, #0
    movlt r0, #1
    str r0, [fp, #-144]
    ldr r0, [fp, #-144]
    cmp r0, #0
    beq block_10
    b block_9
block_7:
    b block_8
block_17:
    ldr r0, [fp, #-80]
    str r0, [fp, #-72]
    ldr r0, [fp, #-16]
    str r0, [fp, #-0]
    ldr r0, [fp, #-72]
    mov r1, r0
    ldr r0, [fp, #-0]
    sdiv r2, r1, r0
    mul r2, r0, r2
    sub r0, r1, r2
    str r0, [fp, #-48]
     ldr r0, #0
    str r0, [fp, #-136]
    ldr r0, [fp, #-48]
    mov r1, r0
    ldr r0, [fp, #-136]
    cmp r1, r0
    mov r0, #0
    moveq r0, #1
    str r0, [fp, #-128]
    ldr r0, [fp, #-128]
    cmp r0, #0
    beq block_19
    b block_18
block_18:
    ldr r0, [fp, #-16]
    str r0, [fp, #-152]
     ldr r0, #1
    str r0, [fp, #-64]
    ldr r0, [fp, #-152]
    mov r1, r0
    ldr r0, [fp, #-64]
    add r0, r1, r0
    str r0, [fp, #-40]
    ldr r0, [fp, #-40]
    str r0, [fp, #-16]
    b block_14
    add sp, sp, #192
    pop {fp, lr}
    bx lr
main:
    push {lr}
    bl func_1
    pop {lr}
    bx lr
