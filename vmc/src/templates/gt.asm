@SP
AM=M-1
D=M
A=A-1
D=D-M
@{label_true}
D;JLE
@0
D=A
@{label_end}
D;JMP
({label_true})
@0
D=!A
({label_end})
@SP
A=M-1
M=D
