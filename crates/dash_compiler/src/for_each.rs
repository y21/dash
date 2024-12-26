use std::mem;

use dash_middle::compiler::instruction::AssignKind;
use dash_middle::interner::{Symbol, sym};
use dash_middle::lexer::token::TokenType;
use dash_middle::parser::error::Error;
use dash_middle::parser::expr::{Expr, ExprKind};
use dash_middle::parser::statement::{
    BlockStatement, ScopeId, Statement, StatementKind, VariableBinding, VariableDeclaration, VariableDeclarations,
    WhileLoop,
};
use dash_middle::sourcemap::Span;
use dash_middle::visitor::Visitor;

use crate::builder::InstructionBuilder;
use crate::instruction::compile_local_load;

pub enum ForEachLoopKind {
    ForOf,
    ForIn,
}

/// Helper for desugaring for each-like loops that iterate through an iterator
/// (for..in and for..of), as well as `yield*`
pub struct ForEachDesugarCtxt<'a, 'cx, 'interner> {
    /// The local that stores the iterator
    iterator_local: u16,
    /// The local that stores the intermediate next() call result
    gen_step_local: u16,
    ib: &'a mut InstructionBuilder<'cx, 'interner>,
}

impl<'a, 'cx, 'interner> ForEachDesugarCtxt<'a, 'cx, 'interner> {
    pub fn new(ib: &'a mut InstructionBuilder<'cx, 'interner>, at: Span) -> Result<Self, Error> {
        let iterator_local = ib
            .add_unnameable_local(sym::for_of_iter)
            .map_err(|_| Error::LocalLimitExceeded(at))?;

        let gen_step_local = ib
            .add_unnameable_local(sym::for_of_gen_step)
            .map_err(|_| Error::LocalLimitExceeded(at))?;

        Ok(Self {
            ib,
            iterator_local,
            gen_step_local,
        })
    }

    /// Assigns the iterator expression to the iterator local
    pub fn init_iterator(&mut self, kind: ForEachLoopKind, iterable: Expr) -> Result<(), Error> {
        self.ib.accept_expr(iterable)?;
        match kind {
            ForEachLoopKind::ForOf => self.ib.build_symbol_iterator(),
            ForEachLoopKind::ForIn => self.ib.build_for_in_iterator(),
        }
        self.ib
            .build_local_store(AssignKind::Assignment, self.iterator_local, false);
        self.ib.build_pop();
        Ok(())
    }

    /// Creates an expression that loads the step value
    pub fn step_value_expr(&self) -> Expr {
        Expr {
            span: Span::COMPILER_GENERATED,
            kind: ExprKind::property_access(
                false,
                Expr {
                    span: Span::COMPILER_GENERATED,
                    kind: ExprKind::compiled(compile_local_load(self.gen_step_local, false)),
                },
                Expr {
                    span: Span::COMPILER_GENERATED,
                    kind: ExprKind::identifier(sym::value),
                },
            ),
        }
    }

    /// Prepends a variable definition with the iterator step's value at the "beginning" of a statement by wrapping it in a new block with the given scope.
    pub fn prepend_variable_defn(&self, binding: VariableBinding, body: &mut Statement, scope: ScopeId) {
        // Assign iterator value to binding at the very start of the for loop body
        let next_value = Statement {
            span: Span::COMPILER_GENERATED,
            kind: StatementKind::Variable(VariableDeclarations(vec![VariableDeclaration::new(
                binding,
                Some(self.step_value_expr()),
            )])),
        };

        // Create a new block for the variable declaration
        let old_body = mem::replace(body, Statement::dummy_empty());
        *body = Statement {
            span: Span::COMPILER_GENERATED,
            kind: StatementKind::Block(BlockStatement(vec![next_value, old_body], scope)),
        };
    }

    /// Emits a loop, assuming that `iterator_local` has been initialized with the iterator
    pub fn compile_loop(&mut self, label: Option<Symbol>, body: Box<Statement>) -> Result<(), Error> {
        self.ib.visit_while_loop(Span::COMPILER_GENERATED, label, WhileLoop {
            condition: Expr {
                span: Span::COMPILER_GENERATED,
                kind: ExprKind::unary(TokenType::LogicalNot, Expr {
                    span: Span::COMPILER_GENERATED,
                    kind: ExprKind::property_access(
                        false,
                        Expr {
                            span: Span::COMPILER_GENERATED,
                            kind: ExprKind::assignment_local_space(
                                self.gen_step_local,
                                Expr {
                                    span: Span::COMPILER_GENERATED,
                                    kind: ExprKind::function_call(
                                        Expr {
                                            span: Span::COMPILER_GENERATED,
                                            kind: ExprKind::property_access(
                                                false,
                                                Expr {
                                                    span: Span::COMPILER_GENERATED,
                                                    kind: ExprKind::compiled(compile_local_load(
                                                        self.iterator_local,
                                                        false,
                                                    )),
                                                },
                                                Expr {
                                                    span: Span::COMPILER_GENERATED,
                                                    kind: ExprKind::identifier(sym::next),
                                                },
                                            ),
                                        },
                                        Vec::new(),
                                        false,
                                    ),
                                },
                                TokenType::Assignment,
                            ),
                        },
                        Expr {
                            span: Span::COMPILER_GENERATED,
                            kind: ExprKind::identifier(sym::done),
                        },
                    ),
                }),
            },
            body,
        })?;

        Ok(())
    }

    /// Convenience function for fully desugaring a for..of/for..in loop given an iterable
    pub fn desugar_for_each_kinded_loop(
        &mut self,
        kind: ForEachLoopKind,
        binding: VariableBinding,
        iterable: Expr,
        mut loop_body: Box<Statement>,
        loop_scope: ScopeId,
        label: Option<Symbol>,
    ) -> Result<(), Error> {
        // For-Of Loop Desugaring:
        //
        //    for (const x of [1,2]) console.log(x)
        //
        // becomes
        //
        //    let __forOfIter = [1,2][Symbol.iterator]();
        //    let __forOfGenStep;
        //    let x;
        //    while (!(__forOfGenStep = __forOfIter.next()).done) {
        //        x = __forOfGenStep.value;
        //        console.log(x)
        //    }
        //
        // For-In loops are desugared almost equivalently, except an iterator over the object keys is used

        self.init_iterator(kind, iterable)?;

        self.prepend_variable_defn(binding, &mut loop_body, loop_scope);

        self.compile_loop(label, loop_body)
    }
}
