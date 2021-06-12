use crate::eval;

#[test]
pub fn recursion() {
    let result = eval::<()>(
        r#"
    function recurse(a, b) {
        if (a === 0) {
            return b;
        }
    
        return recurse(a - 1, b * 2);
    }
    
    recurse(50, 2);
    "#,
        None,
    )
    .unwrap()
    .unwrap()
    .borrow()
    .as_number();

    assert_eq!(result, 2251799813685248f64);
}

#[test]
pub fn loop_break() {
    let result = eval::<()>(
        r#"
        let i = 0;
        for (;;) {
            if (++i % 2 === 0 && i > 50) {
                break;
            }
        }
        i
    "#,
        None,
    )
    .unwrap()
    .unwrap()
    .borrow()
    .as_number();

    assert_eq!(result, 52f64);
}

#[test]
pub fn single_line_comments() {
    let result = eval::<()>(
        r#"
        // hello
        1+2
    "#,
        None,
    )
    .unwrap()
    .unwrap()
    .borrow()
    .as_number();

    assert_eq!(result, 3f64);
}

#[test]
pub fn multi_line_comments() {
    let result = eval::<()>(
        r#"
        /*
        this is a comment
        that is spread across
        several
        lines
        */

        /**/

        3+3
    "#,
        None,
    )
    .unwrap()
    .unwrap()
    .borrow()
    .as_number();

    assert_eq!(result, 6f64);
}

#[test]
pub fn else_if() {
    let result = eval::<()>(
        r#"if (false) {
            console.log("no");
          } else if (false) {
            console.log("no");
          } else {
              
          }
          let x = 6; x
    "#,
        None,
    )
    .unwrap()
    .unwrap()
    .borrow()
    .as_number();

    assert_eq!(result, 6f64);
}

#[test]
pub fn conditional() {
    let result = eval::<()>(
        r#"
        typeof true ? 6 : 1
    "#,
        None,
    )
    .unwrap()
    .unwrap()
    .borrow()
    .as_number();

    assert_eq!(result, 6f64);
}

#[test]
pub fn property_lookup_this_binding() {
    let result = eval::<()>(
        r#"
        true.constructor === Boolean ? 6 : false
    "#,
        None,
    )
    .unwrap()
    .unwrap()
    .borrow()
    .as_number();

    assert_eq!(result, 6f64);
}
