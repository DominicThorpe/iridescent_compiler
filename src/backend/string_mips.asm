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



# Takes a pointer to a string in $a0 and puts the length of that string into $a0.
__strlen:
    move $t0, $a0 # load arg into temp register
    move $t1, $zero # initialize length counter to 0

__strlen_loop:
    lb $t2, 0($t0) # get character at the current index
    beq $t2, $zero, __strlen_return # if string[i] == '\0', return

    # otherwise increment count and string ptr
    addi $t0, $t0, 1
    addi $t1, $t1, 1 

    j __strlen_loop # and repeat loop

# return the length of the string
__strlen_return:
    move $a0, $t1
    jr $ra



# Takes a pointer to a string in $a0 and a pointer to another string in $a1, and 
# copies the 1st string into the second. May overwrite other data if string 1 is
# longer than string 2.
__strcopy:
    # move args into saved registers
    move $s0, $a0
    move $s1, $a1

    move $t3, $zero # initialize counter $t3 to 0

    # get length of string 1 and store it in $t2
    move $s2, $ra # save return address
    jal __strlen
    move $ra, $s2 # restore return address
    move $t6, $a0

    # restore saved registers
    move $t0, $s0
    move $t1, $s1

# iterate, copying each character in string 1 to string 2
__strcopy_loop:
    beq $t6, $t3, __strcopy__return # if $t6 == $t3, return 

    lb $t4, 0($t0) # get byte at index in string 1
    sb $t4, 0($t1) # store that byte into string 2

    # increment string ptrs and counter
    addi $t0, $t0, 1
    addi $t1, $t1, 1
    addi $t3, $t3, 1

    j __strcopy_loop # rerun loop

# return from strcopy
__strcopy__return:
    jr $ra



# Takes 2 pointers to strings in $a0 and $a1, and returns a pointer in $a0 to a string
# containing $a0 concatenated to $a1.
__strcat:
    # move arguments into temp registers
    move $s3, $a0
    move $s4, $a1
    
    move $s7, $ra # save return address
    
    jal __strlen # get the length of the 1st string
    move $s5, $a0
    
    move $a0, $s4
    jal __strlen
    move $s6, $a0 # get the length of the 2nd string
    
    add $t5, $s5, $s6 # get the length of the new string
    
    # sbrk syscall to allocate memory for new string
    li $v0, 9
    move $a0, $t5
    syscall
    move $s6, $v0 # address of new string into $t6
    
    # copy 1st string into new string
    move $a0, $s3
    move $a1, $s6
    jal __strcopy
    
    add $t7, $s6, $s5 # get address of new string + length of 1st string
    move $a0, $s4
    move $a1, $t7
    jal __strcopy # copy 2nd string into new string after 1st string
    move $ra, $s7
    
    move $a0, $s6
    jr $ra
    