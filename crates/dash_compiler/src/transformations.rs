use dash_middle::interner::sym;
use dash_middle::lexer::token::TokenType;
use dash_middle::parser::expr::AssignmentExpr;
use dash_middle::parser::expr::AssignmentTarget;
use dash_middle::parser::expr::Expr;
use dash_middle::parser::expr::ExprKind;
use dash_middle::parser::expr::PropertyAccessExpr;
use dash_middle::parser::statement::BlockStatement;
use dash_middle::parser::statement::Class;
use dash_middle::parser::statement::ClassMemberKind;
use dash_middle::parser::statement::ClassProperty;
use dash_middle::parser::statement::Loop;
use dash_middle::parser::statement::ReturnStatement;
use dash_middle::parser::statement::Statement;
use dash_middle::parser::statement::StatementKind;
use dash_middle::sourcemap::Span;

/// Implicitly patches the last expression to be returned from the function
///
/// Or inserts `return undefined;` if there is no last expression
pub fn ast_patch_implicit_return(ast: &mut Vec<Statement>) {
    match ast.last_mut().map(|s| &mut s.kind) {
        Some(StatementKind::Return(..)) => {}
        Some(StatementKind::Expression(..)) => {
            let expr = match ast.pop().map(|s| s.kind) {
                Some(StatementKind::Expression(expr)) => expr,
                _ => unreachable!(),
            };

            ast.push(Statement {
                span: expr.span,
                kind: StatementKind::Return(ReturnStatement(expr)),
            });
        }
        Some(StatementKind::Block(BlockStatement(block))) => ast_patch_implicit_return(block),
        _ => ast_insert_implicit_return(ast),
    }
}

pub fn ast_insert_implicit_return(ast: &mut Vec<Statement>) {
    ast.push(Statement::dummy_return());
}

/// For every field property, insert a `this.fieldName = fieldValue` expression in the constructor
pub fn insert_initializer_in_constructor(class: &Class, statements: &mut Vec<Statement>) {
    let mut prestatements = Vec::new();
    for member in &class.members {
        if let ClassMemberKind::Property(ClassProperty {
            name,
            value: Some(value),
        }) = &member.kind
        {
            prestatements.push(Statement {
                span: Span::COMPILER_GENERATED,
                kind: StatementKind::Expression(Expr {
                    span: Span::COMPILER_GENERATED,
                    kind: ExprKind::Assignment(AssignmentExpr {
                        left: AssignmentTarget::Expr(Box::new(Expr {
                            span: Span::COMPILER_GENERATED,
                            kind: ExprKind::PropertyAccess(PropertyAccessExpr {
                                computed: false,
                                property: Box::new(Expr {
                                    span: Span::COMPILER_GENERATED,
                                    kind: ExprKind::identifier(*name),
                                }),
                                target: Box::new(Expr {
                                    span: Span::COMPILER_GENERATED,
                                    kind: ExprKind::identifier(sym::THIS),
                                }),
                            }),
                        })),
                        operator: TokenType::Assignment,
                        right: Box::new(value.clone()),
                    }),
                }),
            });
        }
    }
    prestatements.append(statements);
    *statements = prestatements;
}

/// Hoists all function declarations as well as moving the assignment to the nearest enclosing block.
/// Example:
/// ```js
/// x();
/// function x() { return 56; }
/// ```
/// becomes
/// ```js
/// var x;
/// x = function x() { return 56; };
/// x();
/// ```
///
/// Example 2:
/// ```js
/// x();
/// if (false) {
///     function x() { return 56; }
/// }
/// ```
/// becomes
/// ```js
/// var x;
/// x();
/// if (false) {
///    x = function x() { return 56; };
/// }
/// ```
pub fn hoist_declarations(ast: &mut Vec<Statement>) {
    let mut prepend_function_assigns = Vec::new();

    fn hoist_function_declaration(prepend_function_assigns: &mut Vec<Statement>, stmt: &mut Statement) {
        match &mut stmt.kind {
            StatementKind::Function(function_decl) => {
                let name = function_decl.name.expect("Function statement did not have a name");
                let function_stmt = match std::mem::replace(stmt, Statement::dummy_empty()).kind {
                    StatementKind::Function(function_decl) => function_decl,
                    _ => unreachable!(),
                };

                prepend_function_assigns.push(Statement {
                    span: stmt.span,
                    kind: StatementKind::Expression(Expr {
                        span: stmt.span,
                        kind: ExprKind::assignment(
                            Expr {
                                span: stmt.span,
                                kind: ExprKind::identifier(name),
                            },
                            Expr {
                                span: stmt.span,
                                kind: ExprKind::function(function_stmt),
                            },
                            TokenType::Assignment,
                        ),
                    }),
                });
            }
            StatementKind::Block(BlockStatement(stmts)) => {
                let mut prepend = Vec::new();
                for stmt in stmts.iter_mut() {
                    hoist_function_declaration(&mut prepend, stmt);
                }

                if !stmts.is_empty() {
                    stmts.insert(
                        0,
                        Statement {
                            span: Span::COMPILER_GENERATED,
                            kind: StatementKind::Block(BlockStatement(prepend)),
                        },
                    );
                }
            }
            StatementKind::If(if_stmt) => {
                hoist_function_declaration(prepend_function_assigns, &mut if_stmt.then);
                if let Some(else_stmt) = &mut if_stmt.el {
                    hoist_function_declaration(prepend_function_assigns, else_stmt);
                }
            }
            StatementKind::Loop(Loop::For(for_stmt)) => {
                hoist_function_declaration(prepend_function_assigns, &mut for_stmt.body);
                if let Some(init_stmt) = &mut for_stmt.init {
                    hoist_function_declaration(prepend_function_assigns, init_stmt);
                }
            }
            StatementKind::Loop(Loop::ForIn(for_stmt)) => {
                hoist_function_declaration(prepend_function_assigns, &mut for_stmt.body);
            }
            StatementKind::Loop(Loop::ForOf(for_stmt)) => {
                hoist_function_declaration(prepend_function_assigns, &mut for_stmt.body);
            }
            StatementKind::Loop(Loop::While(while_stmt)) => {
                hoist_function_declaration(prepend_function_assigns, &mut while_stmt.body);
            }
            StatementKind::Try(tc_stmt) => {
                hoist_function_declaration(prepend_function_assigns, &mut tc_stmt.try_);
                hoist_function_declaration(prepend_function_assigns, &mut tc_stmt.catch.body);
                if let Some(finally_stmt) = &mut tc_stmt.finally {
                    hoist_function_declaration(prepend_function_assigns, finally_stmt);
                }
            }
            _ => {}
        }
    }

    for node in ast.iter_mut() {
        hoist_function_declaration(&mut prepend_function_assigns, node);
    }

    if !ast.is_empty() {
        ast.insert(
            0,
            Statement {
                span: Span::COMPILER_GENERATED,
                kind: StatementKind::Block(BlockStatement(prepend_function_assigns)),
            },
        );
    }
}
