use crate::vm::stack::Stack;

#[test]
pub fn take() {
    let mut stack = Stack::<u8, 16>::new();
    stack.push(1);
    stack.push(2);
    stack.push(3);

    let mut stack2 = stack.take();
    assert_eq!(stack2.pop(), 3);
    assert_eq!(stack2.pop(), 2);
    assert_eq!(stack2.pop(), 1);

    // Check that both stack pointers are at zero
    assert!(stack2.is_empty());
    assert!(stack.is_empty());
}
