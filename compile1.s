.intel_syntax noprefix
.global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 32
	push r12
	push r13
	push r14
	push r15
	mov r10, rbp
	sub r10, 8
	mov r11, 1
	mov [r10], r11
	mov r10, rbp
	sub r10, 16
	mov r11, 1
	mov [r10], r11
	mov r10, rbp
	sub r10, 24
	mov r11, 0
	mov [r10], r11
.L1:
	mov r10, rbp
	sub r10, 24
	mov r10, [r10]
	mov r11, 10
	cmp r10, r11
	setl r10b
	movzb r10, r10b
	cmp r10, 0
	je .L2
	mov r10, rbp
	sub r10, 32
	mov r11, rbp
	sub r11, 8
	mov r11, [r11]
	mov rbx, rbp
	sub rbx, 16
	mov rbx, [rbx]
	add r11, rbx
	mov [r10], r11
	mov r10, rbp
	sub r10, 8
	mov r11, rbp
	sub r11, 16
	mov r11, [r11]
	mov [r10], r11
	mov r10, rbp
	sub r10, 16
	mov r11, rbp
	sub r11, 32
	mov r11, [r11]
	mov [r10], r11
	mov r10, rbp
	sub r10, 24
	mov r11, rbp
	sub r11, 24
	mov r11, [r11]
	mov rbx, 1
	add r11, rbx
	mov [r10], r11
	jmp .L1
.L2:
	mov r11, rbp
	sub r11, 8
	mov r11, [r11]
	mov rax, r11
	jmp .Lend0
.Lend0:
	pop r15
	pop r14
	pop r13
	pop r12
	mov rsp, rbp
	pop rbp
	ret
