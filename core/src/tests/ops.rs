use crate::eval;

macro_rules! assert_eval_num {
    ($left:expr, $right:expr) => {{
        let result = $left.unwrap().unwrap().borrow().as_number();
        assert_eq!(result, ($right) as f64);
    }};
}

#[test]
pub fn num_add() {
    assert_eval_num!(eval::<()>("12 + 34", None), 46);
}

#[test]
pub fn num_sub() {
    assert_eval_num!(eval::<()>("12 - 34", None), -22);
}

#[test]
pub fn num_mul() {
    assert_eval_num!(eval::<()>("12 * 34", None), 12 * 34);
}

#[test]
pub fn num_div() {
    assert_eval_num!(eval::<()>("12 / 34", None), 34f64 / 16f64);
}

#[test]
pub fn num_rem() {
    assert_eval_num!(eval::<()>("12 % 34", None), 12 % 34);
}

#[test]
pub fn num_pow() {
    assert_eval_num!(eval::<()>("12 ** 34", None), 4f64.powf(6f64));
}

#[test]
pub fn num_left_shift() {
    assert_eval_num!(eval::<()>("4 << 8", None), 4 << 8);
}

#[test]
pub fn num_right_shift() {
    assert_eval_num!(eval::<()>("32 >> 2", None), 32 >> 2);
}

/*#[test]
pub fn num_unsigned_right_shift() {
    assert_eval_num!(eval("32 >> 2"), 32 >> 2);
}*/

#[test]
pub fn num_bitwise_and() {
    assert_eval_num!(eval::<()>("6 & 2", None), 6 & 2);
}

#[test]
pub fn num_bitwise_or() {
    assert_eval_num!(eval::<()>("6 | 2", None), 6 | 2);
}

#[test]
pub fn num_bitwise_xor() {
    assert_eval_num!(eval::<()>("6 ^ 2", None), 6 ^ 2);
}
