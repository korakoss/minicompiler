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
     ldr r0, =1
    str r0, [fp, #-80]
    ldr r0, [fp, #-80]
    str r0, [fp, #-64]
    b block_1
block_2:
    b block_4
block_4:
    ldr r0, [fp, #-64]
    str r0, [fp, #-0]
     ldr r1, [fp, #-0]
    bl func_0
    str r0, [fp, #-48]
    ldr r0, [fp, #-48]
    cmp r0, #0
    beq block_6
    b block_5
block_3:
block_6:
    ldr r0, [fp, #-64]
    str r0, [fp, #-72]
    ldr r0, [fp, #-72]
    mov r1, r0
    ldr r0, =fmt
    bl printf
    b block_5
block_1:
    ldr r0, [fp, #-64]
    str r0, [fp, #-32]
     ldr r0, =32
    str r0, [fp, #-16]
    ldr r0, [fp, #-32]
    mov r1, r0
    ldr r0, [fp, #-16]
    cmp r1, r0
    mov r0, #0
    movlt r0, #1
    str r0, [fp, #-8]
    ldr r0, [fp, #-8]
    cmp r0, #0
    beq block_2
    b block_3
block_5:
    ldr r0, [fp, #-64]
    str r0, [fp, #-24]
     ldr r0, =1
    str r0, [fp, #-40]
    ldr r0, [fp, #-24]
    mov r1, r0
    ldr r0, [fp, #-40]
    add r0, r1, r0
    str r0, [fp, #-56]
    ldr r0, [fp, #-56]
    str r0, [fp, #-64]
    b block_1
ret_1:
    add sp, sp, #88
    pop {fp, lr}
    bx lr
func_0:
    push {fp, lr}
    mov fp, sp
    sub sp, sp, #192
    str r1, [fp, #-104]
    b block_7
block_20:
    ldr r0, [fp, #-24]
    str r0, [fp, #-112]
     ldr r0, =1
    str r0, [fp, #-72]
    ldr r0, [fp, #-112]
    mov r1, r0
    ldr r0, [fp, #-72]
    add r0, r1, r0
    str r0, [fp, #-8]
    ldr r0, [fp, #-8]
    str r0, [fp, #-24]
    b block_16
block_22:
    b block_20
block_8:
    ldr r0, [fp, #-104]
    str r0, [fp, #-96]
     ldr r0, =2
    str r0, [fp, #-160]
    ldr r0, [fp, #-96]
    mov r1, r0
    ldr r0, [fp, #-160]
    cmp r1, r0
    mov r0, #0
    movlt r0, #1
    str r0, [fp, #-120]
    ldr r0, [fp, #-120]
    cmp r0, #0
    beq block_10
    b block_9
block_12:
    ldr r0, [fp, #-104]
    str r0, [fp, #-32]
     ldr r0, =2
    str r0, [fp, #-64]
    ldr r0, [fp, #-32]
    mov r1, r0
    ldr r0, [fp, #-64]
    cmp r1, r0
    mov r0, #0
    moveq r0, #1
    str r0, [fp, #-184]
    ldr r0, [fp, #-184]
    cmp r0, #0
    beq block_14
    b block_13
block_13:
     ldr r0, =2
    str r0, [fp, #-56]
    ldr r0, [fp, #-56]
    str r0, [fp, #-24]
    b block_16
block_11:
    b block_9
block_14:
    ldr r0, =1
    str r0, [fp, #-48]
    ldr r0, [fp, #-48]
    b ret_0
block_19:
    ldr r0, [fp, #-104]
    str r0, [fp, #-136]
    ldr r0, [fp, #-24]
    str r0, [fp, #-0]
    ldr r0, [fp, #-136]
    mov r1, r0
    ldr r0, [fp, #-0]
    sdiv r2, r1, r0
    mul r2, r0, r2
    sub r0, r1, r2
    str r0, [fp, #-88]
     ldr r0, =0
    str r0, [fp, #-152]
    ldr r0, [fp, #-88]
    mov r1, r0
    ldr r0, [fp, #-152]
    cmp r1, r0
    mov r0, #0
    moveq r0, #1
    str r0, [fp, #-80]
    ldr r0, [fp, #-80]
    cmp r0, #0
    beq block_21
    b block_20
block_23:
block_10:
    ldr r0, =0
    str r0, [fp, #-16]
    ldr r0, [fp, #-16]
    b ret_0
block_17:
    b block_19
block_16:
    ldr r0, [fp, #-24]
    str r0, [fp, #-144]
    ldr r0, [fp, #-104]
    str r0, [fp, #-128]
    ldr r0, [fp, #-144]
    mov r1, r0
    ldr r0, [fp, #-128]
    cmp r1, r0
    mov r0, #0
    movlt r0, #1
    str r0, [fp, #-176]
    ldr r0, [fp, #-176]
    cmp r0, #0
    beq block_17
    b block_18
block_18:
    ldr r0, =1
    str r0, [fp, #-40]
    ldr r0, [fp, #-40]
    b ret_0
block_21:
    ldr r0, =0
    str r0, [fp, #-168]
    ldr r0, [fp, #-168]
    b ret_0
block_7:
    b block_8
block_15:
    b block_13
block_9:
    b block_12
ret_0:
    add sp, sp, #192
    pop {fp, lr}
    bx lr
main:
    push {lr}
    bl func_1
    pop {lr}
    bx lr
