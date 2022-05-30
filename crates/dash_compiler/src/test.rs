use super::CompileResult;
use super::FunctionCompiler;

fn compile(source: &str) -> CompileResult {
    let ast = Parser::from_str(source)
        .expect("Lex error")
        .parse_all()
        .expect("Parse error");

    FunctionCompiler::new().compile_ast(ast).expect("Compile error")
}

#[test]
pub fn empty() {
    let c = compile("");
    assert_eq!(c.instructions, [CONSTANT, 0, RET]);
    assert_eq!(&*c.cp, [Constant::Undefined]);
}

#[test]
fn binary_math() {
    macro_rules! case {
        ($($st:expr, $instr:expr);*) => {
            $(
                let c = compile(concat!("1234 ", $st, " 5678"));
                assert_eq!(c.instructions, [CONSTANT, 0, CONSTANT, 1, $instr, RET]);
                assert_eq!(&*c.cp, [Constant::Number(1234.0), Constant::Number(5678.0)]);
            )*
        };
    }

    case! {
        "+", ADD;
        "-", SUB;
        "*", MUL;
        "/", DIV;
        "%", REM;
        "**", POW;
        ">", GT;
        ">=", GE;
        "<", LT;
        "<=", LE;
        "==", EQ;
        "!=", NE
    };
}
