use crate::eval;

#[test]
pub fn all() {
    let source = r#"
    function assert(a, b) {
        if (a !== b) {
            throw new Error("Fail: " + a);
        }
    }

    assert("test".bold(), "<b>test</b>");
    assert("test".italics(), "<i>test</i>");
    assert("test".link("test2"), '<a href="test2">test</a>');
    assert("test".sub(), "<sub>test</sub>");
    assert("test".sup(), "<sup>test</sup>");
    assert("test".fontsize(10), '<font size="10">test</font>');
    assert("test".strike(), "<strike>test</strike>");
    assert("test".fontcolor("red"), '<font color="red">test</font>');
    assert("test".anchor("test2"), '<a name="test2">test</a>');
    assert("test".blink(), "<blink>test</blink>");
    assert("test".fixed(), "<tt>test</tt>");
    assert("test".small(), "<small>test</small>");
    "#;

    eval::<()>(source, None).unwrap();
}
