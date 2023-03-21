# string for MIPS, used for string-related utility functions needed for Iridescent to work

# Compares strings and returns 1 if they are equal, 0 otherwise. Takes a pointer to the strings
# in $a0 and $a1, result goes in $a0.
__strcmp:
    # load args into temp registers
    move $t0, $a0
    move $t1, $a1

# iteratively check if the current character at $a0 + i == $a1 + i, or if either are '\0' which
# denotes the end of the string.
__strcmp_loop:
    # get characters at $t0 + i and $t1 + i
    lb $t3, 0($t0)
    lb $t4, 0($t1)

    bne $t3, $t4, __strcmp_return_false # if string_a[i] != string_b[i], return false
    beq $t3, $zero, __strcmp_return_true # if string_a[i] == '\0', return true

    # else increment array index and repeat the loop
    addi $t0, $t0, 1
    addi $t1, $t1, 1
    j __strcmp_loop 

# return false by setting $a0 and $a1 to 0
__strcmp_return_false:
    move $a0, $zero
    move $a1, $zero
    jr $ra

# return true by setting $a0 to 1 and $a1 to 0
__strcmp_return_true:
    addi $a0, $zero, 1
    move $a1, $zero
    jr $ra
