# math64 for MIPS, used to perform 64 bit mathematics


# integer divides 64 bit integer in $a0 and $a1 by divisor in $a1 and $a2. Result is
# put into $a0 and $a1. 
__divint64:
    # clear result registers
	li $t4, 0
	li $t5, 0

    # end if the divisor is 0
	seq $t1, $a0, $zero
	seq $t2, $a1, $zero
	and $t1, $t1, $t2
	bnez $t1, __divint64_end

    # move args into temp registers
	move $t0, $a0
	move $t1, $a1
	move $t2, $a2
	move $t3, $a3

__divint64_loop:
    # iteratively subtract divisor from target until it target < divisor
    # then return
	subu $t0, $t0, $t2
	subu $t1, $t1, $t3
	addiu $t5, $t5, 1
	sltiu $t7, $t5, 1
	addu $t4, $t4, $t7
	sleu $t6, $t2, $t0
	sleu $t7, $t3, $t1
	and $t6, $t6, $t7
	bnez $t6, __divint64_loop

__divint64_end:
    # move results into $a0 and $a1
	move $a0, $t0
	move $a1, $t5
	jr $ra # return



# logical left shifts $a0 and $a1 by the amount in $a2
__sllint64:
	move $t0, $a1 # copy upper 32 bits into $t0

    # different subroutine if shift > 64 or > 32
	bgeu $a2, 64, __sllint64_return0
	bgeu $a2, 32, __sllint64_over32

    # if shift < 32, shift both registers
	sllv $a0, $a0, $a2
	sllv $a1, $a1, $a2

    # move lower x bits of $a1 into upper x bits of $a0
	addi $t1, $zero, 32
	sub $t2, $t1, $a2
	srlv $t0, $t0, $t2
	or $a0, $a0, $t0

	jr $ra # return 

__sllint64_over32:
    # if shift >= 32, clear $a1 and move lower x bits of $a1 into
    # upper 32 bits of $a0 
	move $a0, $a1
	li $a1, 0
	subi $a2, $a2, 32
	sllv $a0, $a0, $a2
	jr $ra # return

__sllint64_return0:
    # if shift >= 64, return 0
	li $a0, 0
	li $a1, 0
	jr $ra # return



# logical right shifts $a0 and $a1 by the amount in $a2
__srlint64:
	move $t0, $a0 # copy lower 32 bits into $t0

    # different subroutine if shift > 64 or > 32
	bgeu $a2, 64, __srlint64_return0
	bgeu $a2, 32, __srlint64_over32

    # if shift < 32, shift both registers
	srlv $a0, $a0, $a2
	srlv $a1, $a1, $a2

    # move lower x bits of $a1 into upper x bits of $a0
	addi $t1, $zero, 32
	sub $t2, $t1, $a2
	sllv $t0, $t0, $t2
	or $a1, $a1, $t0

	jr $ra # return 

__srlint64_over32:
    # if shift >= 32, clear $a0 and move upper x bits of $a0 into
    # lower 32 bits of $a1 
	move $a1, $a0
	li $a0, 0
	subi $a2, $a2, 32
	srlv $a1, $a1, $a2
	jr $ra # return

__srlint64_return0:
    # if shift >= 64, return 0
	li $a0, 0
	li $a1, 0
	jr $ra # return



# arithmetic right shifts $a0 and $a1 by the amount in $a2
__sraint64:
	move $t0, $a0 # copy lower 32 bits into $t0

    # different subroutine if shift > 64 or > 32
	bgeu $a2, 64, __sraint64_over64
	bgeu $a2, 32, __sraint64_over32

    # if shift < 32, shift both registers
	srav $a0, $a0, $a2
	srav $a1, $a1, $a2

    # move lower x bits of $a1 into upper x bits of $a0
	addi $t1, $zero, 32
	sub $t2, $t1, $a2
	sllv $t0, $t0, $t2
	or $a1, $a1, $t0

	jr $ra # return 

__sraint64_over32:
    # if shift >= 32, replace $a0 with -1 and move upper x bits of $a0 into
    # lower 32 bits of $a1 
	move $a1, $a0
	li $a0, -1
	subi $a2, $a2, 32
	srav $a1, $a1, $a2
	jr $ra # return
	
__sraint64_over64:
	andi $t0, $a0, 0x80000000
	beqz $t0, __sraint64_return0
	j __sraint64_return_minus1
	
__sraint64_return_minus1:
	li $a0, -1
	li $a1, -1
	jr $ra

__sraint64_return0:
    # if shift >= 64, and sign bit is not set, return 0
	li $a0, 0
	li $a1, 0
	jr $ra # return

# end of math64 library


