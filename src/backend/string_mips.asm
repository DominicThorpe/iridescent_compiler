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



# Takes a number in $a0 and outputs a pointer to its string representation in $a0, which
# will be no more than 12 bytes.
__tostring_int:
    move $t0, $a0 # move argument into $t0

    addi $v0, $zero, 9 # syscall code for sbrk
    addi $a0, $zero, 12 # allocate 12 bytes
    syscall # allocate 12 bytes of memory for max 12 digits of int

    # move address of target string into $t9 and a copy into $t1
    move $t1, $v0
    move $t9, $v0

    # check if sign bit set, add negative sign if it is, else skip
    andi $t2, $t0, 0x80000000 # get sign bit
    beqz $t2, __tostring_int_positive # skip adding '-' if positive

    addi $t3, $t3, 0x2D # load 0x2D (ASCII '-') into $t1
    sb $t3, 0($t1) # save '-' into the start of the string
    addi $t1, $t1, 1 # go to next byte in string

    abs $t0, $t0 # make int positive

__tostring_int_positive:
	li $t3, 1000000000

__tostring_int_loop:
	div $t0, $t3 # divide number by current digit divisor
	mflo $t4 # move billion digit into $t4
	mfhi $t0 # replace original number with remainder
	
	addi $t4, $t4, 0x30 # add 48 to quotient to convert number to ASCII representation
	sb $t4, 0($t1)
	addi $t1, $t1, 1 # move on to the next byte in the string
	
	div $t3, $t3, 10 # get next digit divisor by dividing current divisor by 10
	beqz $t3, __tostring_int_end # if there are no more digits, print the result
	
	j __tostring_int_loop # otherwise, repeat the loop


__tostring_int_end:
	move $a0, $t1
	jr $ra



# Takes a number > 0 and < 255 in $a0 and outputs a pointer to its string representation in 
# $a0, which will be no more than 4 bytes.
__tostring_byte:
    move $t0, $a0 # move argument into $t0

    addi $v0, $zero, 9 # syscall code for sbrk
    addi $a0, $zero, 4 # allocate 4 bytes
    syscall # allocate 4 bytes of memory for max 3 digits of int + '\0'

    # move address of target string into $t9 and a copy into $t1
    move $t1, $v0
    move $t9, $v0
	li $t3, 100

__tostring_byte_loop:
	div $t0, $t3 # divide number by current digit divisor
	mflo $t4 # move billion digit into $t4
	mfhi $t0 # replace original number with remainder
	
	addi $t4, $t4, 0x30 # add 48 to quotient to convert number to ASCII representation
	sb $t4, 0($t1)
	addi $t1, $t1, 1 # move on to the next byte in the string
	
	div $t3, $t3, 10 # get next digit divisor by dividing current divisor by 10
	beqz $t3, __tostring_byte_end # if there are no more digits, print the result
	
	j __tostring_byte_loop # otherwise, repeat the loop


__tostring_byte_end:
	move $a0, $t9
	jr $ra



# Takes a pointer to the string representation of a 32 bit integer in $a0 and returns that 
# number as an int in $a0.
__fromstring_int:
	# get the length of the string whicb is the number of digits
	move $s0, $ra
	move $s1, $a0 # save value of argument
	jal __strlen
	move $t5, $a0 # put length of string into $t5
	move $ra, $s0

    move $t0, $s1 # restore argument into $t0
    move $t1, $zero # clear $t1 so it can hold the result
    	
    # check if sign bit set, add negative sign if it is, else skip
	lb $t2, 0($t0) # get first character
	li $t8, 0x2D # load ASCII value for '-' into $t8
    seq $t9, $t2, $t8 # set $t9 if negative

    beqz $t9, __fromstring_int_positive # skip next part if positive
    addi $t0, $t0, 1 # else skip '-' character at start of string
    addi $t5, $t5, -1 # decrement length of the string
    	
   
__fromstring_int_positive:
	# multiply coefficient by 10 ^ num digits in number (stored in $t5)
	li $t3, 1
	addi $t5, $t5, -1
	beqz $t5, __fromstring_int_loop # skip loop if only 1 digit


__fromstring_int_coefficient_loop:
	beqz $t5, __fromstring_int_loop # break out of loop after num iterations = num digits
	mul $t3, $t3, 10 # multiply coefficient by 10
	addi $t5, $t5, -1 # decrement digit counter
	j __fromstring_int_coefficient_loop


__fromstring_int_loop:
	lb $t4, 0($t0) # load digit character into $t4
	addi $t4, $t4, -48 # subtract 48 from digit ASCII value to get number
	mul $t4, $t4, $t3 # multiply current string digit by digit coefficient
	add $t1, $t1, $t4 # add it to the total
	
	div $t3, $t3, 10 # get next digit divisor by dividing current divisor by 10
	add $t0, $t0, 1
	beqz $t3, __fromstring_int_handle_sign # if there are no more digits, handle the sign
	
	j __fromstring_int_loop # otherwise, repeat the loop


__fromstring_int_handle_sign:
	# handling negativity
	beqz $t9, __fromstring_int_end # skip if number is positive
	neg $t1, $t1 # else negate
	j __fromstring_int_end


__fromstring_int_end:	
	move $a0, $t1
	jr $ra
    