use crate::math::{ F128, FiniteField };
use crate::stark::{ Hasher };
use crate::utils::{ filled_vector };

const TRACE_LENGTH: usize = 16;
const EXTENSION_FACTOR: usize = 16;

mod comparisons;

// FLOW CONTROL OPERATIONS
// ================================================================================================

#[test]
fn noop() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.noop(0);
    assert_eq!(vec![1, 2, 3, 4, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(4, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
fn assert() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.assert(0);
    assert_eq!(vec![2, 3, 4, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(3, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
#[should_panic(expected = "ASSERT failed at step 0")]
fn assert_fail() {
    let mut stack = init_stack(&[2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.assert(0);
}

// INPUT OPERATIONS
// ================================================================================================

#[test]
fn push() {
    let mut stack = init_stack(&[], &[], &[], TRACE_LENGTH);
    stack.push(0, 3);
    assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(1, stack.depth);
    assert_eq!(1, stack.max_depth);
}

#[test]
fn read() {
    let mut stack = init_stack(&[1], &[2, 3], &[], TRACE_LENGTH);

    stack.read(0);
    assert_eq!(vec![2, 1, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(2, stack.max_depth);

    stack.read(1);
    assert_eq!(vec![3, 2, 1, 0, 0, 0, 0, 0], get_stack_state(&stack, 2));

    assert_eq!(3, stack.depth);
    assert_eq!(3, stack.max_depth);
}

#[test]
fn read2() {
    let mut stack = init_stack(&[1], &[2, 4], &[3, 5], TRACE_LENGTH);

    stack.read2(0);
    assert_eq!(vec![3, 2, 1, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(3, stack.depth);
    assert_eq!(3, stack.max_depth);

    stack.read2(1);
    assert_eq!(vec![5, 4, 3, 2, 1, 0, 0, 0], get_stack_state(&stack, 2));

    assert_eq!(5, stack.depth);
    assert_eq!(5, stack.max_depth);
}

// STACK MANIPULATION OPERATIONS
// ================================================================================================

#[test]
fn dup() {
    let mut stack = init_stack(&[1, 2], &[], &[], TRACE_LENGTH);
    stack.dup(0);
    assert_eq!(vec![1, 1, 2, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(3, stack.depth);
    assert_eq!(3, stack.max_depth);
}

#[test]
fn dup2() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.dup2(0);
    assert_eq!(vec![1, 2, 1, 2, 3, 4, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(6, stack.depth);
    assert_eq!(6, stack.max_depth);
}

#[test]
fn dup4() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.dup4(0);
    assert_eq!(vec![1, 2, 3, 4, 1, 2, 3, 4], get_stack_state(&stack, 1));

    assert_eq!(8, stack.depth);
    assert_eq!(8, stack.max_depth);
}

#[test]
fn pad2() {
    let mut stack = init_stack(&[1, 2], &[], &[], TRACE_LENGTH);
    stack.pad2(0);
    assert_eq!(vec![0, 0, 1, 2, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(4, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
fn drop() {
    let mut stack = init_stack(&[1, 2], &[], &[], TRACE_LENGTH);
    stack.drop(0);
    assert_eq!(vec![2, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(1, stack.depth);
    assert_eq!(2, stack.max_depth);
}

#[test]
fn drop4() {
    let mut stack = init_stack(&[1, 2, 3, 4, 5], &[], &[], TRACE_LENGTH);
    stack.drop4(0);
    assert_eq!(vec![5, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(1, stack.depth);
    assert_eq!(5, stack.max_depth);
}

#[test]
fn swap() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.swap(0);
    assert_eq!(vec![2, 1, 3, 4, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(4, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
fn swap2() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.swap2(0);
    assert_eq!(vec![3, 4, 1, 2, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(4, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
fn swap4() {
    let mut stack = init_stack(&[1, 2, 3, 4, 5, 6, 7, 8], &[], &[], TRACE_LENGTH);
    stack.swap4(0);
    assert_eq!(vec![5, 6, 7, 8, 1, 2, 3, 4], get_stack_state(&stack, 1));

    assert_eq!(8, stack.depth);
    assert_eq!(8, stack.max_depth);
}

#[test]
fn roll4() {
    let mut stack = init_stack(&[1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.roll4(0);
    assert_eq!(vec![4, 1, 2, 3, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(4, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
fn roll8() {
    let mut stack = init_stack(&[1, 2, 3, 4, 5, 6, 7, 8], &[], &[], TRACE_LENGTH);
    stack.roll8(0);
    assert_eq!(vec![8, 1, 2, 3, 4, 5, 6, 7], get_stack_state(&stack, 1));

    assert_eq!(8, stack.depth);
    assert_eq!(8, stack.max_depth);
}

// CONDITIONAL OPERATIONS
// ================================================================================================

#[test]
fn choose() {
    // choose on true
    let mut stack = init_stack(&[2, 3, 0], &[], &[], TRACE_LENGTH);
    stack.choose(0);
    assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(1, stack.depth);
    assert_eq!(3, stack.max_depth);

    let mut stack = init_stack(&[2, 3, 0, 4], &[], &[], TRACE_LENGTH);
    stack.choose(0);
    assert_eq!(vec![3, 4, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(4, stack.max_depth);

    // choose on false
    let mut stack = init_stack(&[2, 3, 1, 4], &[], &[], TRACE_LENGTH);
    stack.choose(0);
    assert_eq!(vec![2, 4, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(4, stack.max_depth);
}

#[test]
#[should_panic(expected = "CHOOSE on a non-binary condition at step 0")]
fn choose_fail() {
    let mut stack = init_stack(&[2, 3, 4], &[], &[], TRACE_LENGTH);
    stack.choose(0);
}

#[test]
fn choose2() {
    // choose on true
    let mut stack = init_stack(&[2, 3, 4, 5, 0, 6, 7], &[], &[], TRACE_LENGTH);
    stack.choose2(0);
    assert_eq!(vec![4, 5, 7, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(3, stack.depth);
    assert_eq!(7, stack.max_depth);

    // choose on false
    let mut stack = init_stack(&[2, 3, 4, 5, 1, 6, 7], &[], &[], TRACE_LENGTH);
    stack.choose2(0);
    assert_eq!(vec![2, 3, 7, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(3, stack.depth);
    assert_eq!(7, stack.max_depth);
}

#[test]
#[should_panic(expected = "CHOOSE2 on a non-binary condition at step 0")]
fn choose2_fail() {
    let mut stack = init_stack(&[2, 3, 4, 5, 6, 8, 8], &[], &[], TRACE_LENGTH);
    stack.choose2(0);
}

// ARITHMETIC AND BOOLEAN OPERATIONS
// ================================================================================================

#[test]
fn add() {
    let mut stack = init_stack(&[1, 2], &[], &[], TRACE_LENGTH);
    stack.add(0);
    assert_eq!(vec![3, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(1, stack.depth);
    assert_eq!(2, stack.max_depth);
}

#[test]
fn mul() {
    let mut stack = init_stack(&[2, 3], &[], &[], TRACE_LENGTH);
    stack.mul(0);
    assert_eq!(vec![6, 0, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(1, stack.depth);
    assert_eq!(2, stack.max_depth);
}

#[test]
fn inv() {
    let mut stack = init_stack(&[2, 3], &[], &[], TRACE_LENGTH);
    stack.inv(0);
    assert_eq!(vec![F128::inv(2), 3, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(2, stack.max_depth);
}

#[test]
#[should_panic(expected = "cannot compute INV of 0 at step 0")]
fn inv_zero() {
    let mut stack = init_stack(&[0], &[], &[], TRACE_LENGTH);
    stack.inv(0);
}

#[test]
fn neg() {
    let mut stack = init_stack(&[2, 3], &[], &[], TRACE_LENGTH);
    stack.neg(0);
    assert_eq!(vec![F128::neg(2), 3, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(2, stack.max_depth);
}

#[test]
fn not() {
    let mut stack = init_stack(&[1, 2], &[], &[], TRACE_LENGTH);
    stack.not(0);
    assert_eq!(vec![0, 2, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(2, stack.depth);
    assert_eq!(2, stack.max_depth);

    stack.not(1);
    assert_eq!(vec![1, 2, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 2));

    assert_eq!(2, stack.depth);
    assert_eq!(2, stack.max_depth);
}

#[test]
#[should_panic(expected = "cannot compute NOT of a non-binary value at step 0")]
fn not_fail() {
    let mut stack = init_stack(&[2, 3], &[], &[], TRACE_LENGTH);
    stack.not(0);
}

// CRYPTOGRAPHIC OPERATIONS
// ================================================================================================

#[test]
fn hashr() {
    let mut stack = init_stack(&[0, 0, 1, 2, 3, 4], &[], &[], TRACE_LENGTH);
    let mut expected = vec![0, 0, 1, 2, 3, 4, 0, 0];

    stack.hashr(0);
    <F128 as Hasher>::apply_round(&mut expected[..F128::STATE_WIDTH], 0);
    assert_eq!(expected, get_stack_state(&stack, 1));

    stack.hashr(1);
    <F128 as Hasher>::apply_round(&mut expected[..F128::STATE_WIDTH], 1);
    assert_eq!(expected, get_stack_state(&stack, 2));

    assert_eq!(6, stack.depth);
    assert_eq!(6, stack.max_depth);
}

// HELPER FUNCTIONS
// ================================================================================================

fn init_stack(public_inputs: &[F128], secret_inputs_a: &[F128], secret_inputs_b: &[F128], trace_length: usize) -> super::StackTrace<F128> {
    let mut user_registers: Vec<Vec<F128>> = Vec::with_capacity(super::MIN_USER_STACK_DEPTH);
    for i in 0..super::MIN_USER_STACK_DEPTH {
        let mut register = filled_vector(trace_length, trace_length * EXTENSION_FACTOR, F128::ZERO);
        if i < public_inputs.len() { 
            register[0] = public_inputs[i];
        }
        user_registers.push(register);
    }

    let aux_register = filled_vector(trace_length, trace_length * EXTENSION_FACTOR, F128::ZERO);

    let mut secret_inputs_a = secret_inputs_a.to_vec();
    secret_inputs_a.reverse();
    let mut secret_inputs_b = secret_inputs_b.to_vec();
    secret_inputs_b.reverse();

    return super::StackTrace {
        aux_register,
        user_registers,
        secret_inputs_a,
        secret_inputs_b,
        max_depth: public_inputs.len(),
        depth    : public_inputs.len()
    };
}

fn get_stack_state(stack: &super::StackTrace<F128>, step: usize) -> Vec<F128> {
    let mut state = Vec::with_capacity(stack.user_registers.len());
    for i in 0..stack.user_registers.len() {
        state.push(stack.user_registers[i][step]);
    }
    return state;
}

fn get_aux_state(stack: &super::StackTrace<F128>, step: usize) -> Vec<F128> {
    return vec![stack.aux_register[step]];
}