section .data
   num1 dw 1234h
   num2 dw 5678h
   result dw 0

section .text
   global _start

_start:
   ; Load numbers into registers
   mov ax, [num1]
   mov bx, [num2]

   ; Perform addition
   add ax, bx
   mov [result], ax

   ; Perform subtraction
   sub ax, bx
   mov [result], ax

   ; Exit
   mov eax, 1
   int 0x80