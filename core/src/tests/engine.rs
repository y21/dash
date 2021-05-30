use crate::eval;

#[test]
pub fn recursion() {
    let result = eval(
        r#"
    function recurse(a, b) {
        if (a === 0) {
            return b;
        }
    
        return recurse(a - 1, b * 2);
    }
    
    recurse(50, 2);
    "#,
    )
    .unwrap()
    .unwrap()
    .borrow()
    .as_number();

    assert_eq!(result, 2251799813685248f64);
}

#[test]
pub fn loop_break() {
    let result = eval(
        r#"
        let i = 0;
        for (;;) {
            if (++i % 2 === 0 && i > 50) {
                break;
            }
        }
        i
    "#,
    )
    .unwrap()
    .unwrap()
    .borrow()
    .as_number();

    assert_eq!(result, 52f64);
}
