use dash_compiler::FunctionCompiler;
use dash_lexer::Lexer;
use dash_middle::compiler::StaticImportKind;
use dash_optimizer::type_infer::TypeInferCtx;
use dash_optimizer::OptLevel;
use dash_parser::Parser;

use crate::frame::Frame;
use crate::gc::interner::sym;
use crate::localscope::LocalScope;
use crate::value::object::{NamedObject, Object, PropertyValue};
use crate::value::{Root, Unrooted, Value};
use crate::{throw, Vm};

#[derive(Debug)]
pub enum EvalError {
    Middle(Vec<dash_middle::parser::error::Error>),
    Exception(Unrooted),
}

impl Vm {
    pub fn eval(&mut self, input: &str, opt: OptLevel) -> Result<Unrooted, EvalError> {
        let tokens = Lexer::new(&mut self.interner, input)
            .scan_all()
            .map_err(EvalError::Middle)?;
        let (ast, counter) = Parser::new(&mut self.interner, input, tokens)
            .parse_all()
            .map_err(EvalError::Middle)?;

        let tcx = TypeInferCtx::new(counter);
        let cr = FunctionCompiler::new(input, opt, tcx, &mut self.interner)
            .compile_ast(ast, true)
            .map_err(|err| EvalError::Middle(vec![err]))?;
        let mut frame = Frame::from_compile_result(cr);
        frame.set_sp(self.stack_size());
        let val = self.execute_frame(frame).map_err(EvalError::Exception)?;
        Ok(val.into_value())
    }

    pub fn evaluate_module(
        sc: &mut LocalScope,
        input: &str,
        import_ty: StaticImportKind,
        opt: OptLevel,
    ) -> Result<Unrooted, Unrooted> {
        let re = match FunctionCompiler::compile_str(&mut sc.interner, input, opt) {
            Ok(re) => re,
            Err(err) => throw!(sc, SyntaxError, "Middle error: {:?}", err),
        };

        let frame = Frame::from_compile_result(re);

        let exports = sc.execute_module(frame)?;

        let export_obj = match import_ty {
            StaticImportKind::Default => match exports.default {
                Some(obj) => obj.root(sc),
                None => {
                    let o = NamedObject::new(sc);
                    Value::Object(sc.register(o))
                }
            },
            StaticImportKind::All => {
                let export_obj = NamedObject::new(sc);

                if let Some(default) = exports.default {
                    let default = default.root(sc);
                    export_obj.set_property(sc, sym::default.into(), PropertyValue::static_default(default))?;
                }

                Value::Object(sc.register(export_obj))
            }
        };

        for (k, v) in exports.named {
            let v = v.root(sc);
            export_obj.set_property(sc, k.into(), PropertyValue::static_default(v))?;
        }

        Ok(export_obj.into())
    }
}
