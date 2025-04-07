use dash_compiler::FunctionCompiler;
use dash_lexer::Lexer;
use dash_middle::compiler::StaticImportKind;
use dash_middle::interner::sym;
use dash_optimizer::OptLevel;
use dash_optimizer::type_infer::name_res;
use dash_parser::Parser;

use crate::frame::Frame;
use crate::localscope::LocalScope;
use crate::value::object::{OrdObject, Object, PropertyValue};
use crate::value::propertykey::ToPropertyKey;
use crate::value::{Root, Unrooted, Value};
use crate::{Vm, throw};

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
        let (ast, scope_counter, local_counter) = Parser::new(&mut self.interner, input, tokens)
            .parse_all()
            .map_err(EvalError::Middle)?;

        let nameres = name_res(&ast, scope_counter.len(), local_counter.len());

        let cr = FunctionCompiler::new(input, opt, nameres, scope_counter, &mut self.interner)
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
                    let o = OrdObject::new(sc);
                    Value::object(sc.register(o))
                }
            },
            StaticImportKind::All => {
                let export_obj = OrdObject::new(sc);

                if let Some(default) = exports.default {
                    let default = default.root(sc);
                    export_obj.set_property(sym::default.to_key(sc), PropertyValue::static_default(default), sc)?;
                }

                Value::object(sc.register(export_obj))
            }
        };

        for (k, v) in exports.named {
            let v = v.root(sc);
            export_obj.set_property(k.to_key(sc), PropertyValue::static_default(v), sc)?;
        }

        Ok(export_obj.into())
    }
}
