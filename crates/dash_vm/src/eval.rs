use std::fmt;

use dash_compiler::error::CompileError;
use dash_compiler::FunctionCompiler;
use dash_lexer::Lexer;
use dash_middle::compiler::StaticImportKind;
use dash_middle::lexer;
use dash_middle::parser;
use dash_middle::util;
use dash_optimizer::type_infer::TypeInferCtx;
use dash_optimizer::OptLevel;
use dash_parser::Parser;

use crate::frame::Frame;
use crate::localscope::LocalScope;
use crate::throw;
use crate::value::object::NamedObject;
use crate::value::object::Object;
use crate::value::object::PropertyValue;
use crate::value::Value;
use crate::Vm;
use dash_compiler::from_string::CompileStrError;

#[derive(Debug)]
pub enum EvalError<'a> {
    Lexer(Vec<lexer::error::Error<'a>>),
    Parser(Vec<parser::error::Error<'a>>),
    Compiler(CompileError),
    Exception(Value),
}

impl<'a> fmt::Display for EvalError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::Lexer(errors) => {
                util::fmt_group(f, errors, "\n\n")?;
            }
            EvalError::Parser(errors) => {
                util::fmt_group(f, errors, "\n\n")?;
            }
            EvalError::Compiler(error) => {
                writeln!(f, "{error:?}")?;
            }
            EvalError::Exception(value) => {
                writeln!(f, "Exception: {value:?}")?;
            }
        }

        Ok(())
    }
}

impl Vm {
    pub fn eval<'a>(&mut self, input: &'a str, opt: OptLevel) -> Result<Value, EvalError<'a>> {
        let tokens = Lexer::new(input).scan_all().map_err(EvalError::Lexer)?;
        let (ast, counter) = Parser::new(input, tokens).parse_all().map_err(EvalError::Parser)?;

        let tcx = TypeInferCtx::new(counter);
        let cr = FunctionCompiler::new(opt, tcx)
            .compile_ast(ast, true)
            .map_err(EvalError::Compiler)?;
        let frame = Frame::from_compile_result(cr);
        let val = self.execute_frame(frame).map_err(EvalError::Exception)?;
        Ok(val.into_value())
    }

    pub fn evaluate_module(
        sc: &mut LocalScope,
        input: &str,
        import_ty: StaticImportKind,
        opt: OptLevel,
    ) -> Result<Value, Value> {
        let re = match FunctionCompiler::compile_str(input, opt) {
            Ok(re) => re,
            Err(CompileStrError::Compiler(ce)) => throw!(sc, SyntaxError, "Compile error: {:?}", ce),
            Err(CompileStrError::Parser(pe)) => throw!(sc, SyntaxError, "Parse error: {:?}", pe),
            Err(CompileStrError::Lexer(le)) => throw!(sc, SyntaxError, "Lex error: {:?}", le),
        };

        let frame = Frame::from_compile_result(re);

        let exports = sc.execute_module(frame)?;

        let export_obj = match import_ty {
            StaticImportKind::Default => {
                let export_obj = match exports.default {
                    Some(obj) => obj,
                    None => {
                        let o = NamedObject::new(sc);
                        Value::Object(sc.register(o))
                    }
                };

                export_obj
            }
            StaticImportKind::All => {
                let export_obj = NamedObject::new(sc);

                if let Some(default) = exports.default {
                    export_obj.set_property(sc, "default".into(), PropertyValue::static_default(default))?;
                }

                Value::Object(sc.register(export_obj))
            }
        };

        for (k, v) in exports.named {
            export_obj.set_property(sc, String::from(k.as_ref()).into(), PropertyValue::static_default(v))?;
        }

        Ok(export_obj)
    }
}
