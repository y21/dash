use std::borrow::Cow;

use dash_middle::lexer::token::TokenType;
use dash_middle::parser::expr::Expr;
use dash_middle::parser::statement::BlockStatement;
use dash_middle::parser::statement::ImportKind;
use dash_middle::parser::statement::Loop;
use dash_middle::parser::statement::ReturnStatement;
use dash_middle::parser::statement::SpecifierKind;
use dash_middle::parser::statement::Statement;
use dash_middle::parser::statement::VariableBinding;
use dash_middle::parser::statement::VariableDeclaration;
use dash_middle::parser::statement::VariableDeclarationKind;
use dash_middle::parser::statement::VariableDeclarationName;
use dash_middle::parser::statement::VariableDeclarations;

/// Implicitly inserts a `return` statement for the last expression
pub fn ast_insert_return<'a>(ast: &mut Vec<Statement<'a>>) {
    match ast.last_mut() {
        Some(Statement::Return(..)) => {}
        Some(Statement::Expression(..)) => {
            let expr = match ast.pop() {
                Some(Statement::Expression(expr)) => expr,
                _ => unreachable!(),
            };

            ast.push(Statement::Return(ReturnStatement(expr)));
        }
        Some(Statement::Block(BlockStatement(block))) => ast_insert_return(block),
        _ => ast.push(Statement::Return(ReturnStatement::default())),
    }
}

/// Returns a vector of variable declarations that must be hoisted
/// and also hoists all function declarations as well as moving the assignment to the nearest enclosing block.
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
pub fn hoist_declarations<'a>(ast: &mut Vec<Statement<'a>>) -> Vec<VariableBinding<'a>> {
    let mut vars = Vec::new();

    let mut prepend_function_assigns = Vec::new();

    fn hoist_function_declaration<'a>(
        variable_bindings: &mut Vec<VariableBinding<'a>>,
        prepend_function_assigns: &mut Vec<Statement<'a>>,
        stmt: &mut Statement<'a>,
    ) {
        match stmt {
            Statement::Import(
                ImportKind::AllAs(SpecifierKind::Ident(name), ..)
                | ImportKind::DefaultAs(SpecifierKind::Ident(name), ..),
            ) => {
                variable_bindings.push(VariableBinding {
                    ty: None,
                    kind: VariableDeclarationKind::Const,
                    name: VariableDeclarationName::Identifier(*name),
                });
            }
            Statement::Function(function_decl) => {
                let name = function_decl.name.expect("Function statement did not have a name");
                let function_stmt = match std::mem::replace(stmt, Statement::Empty) {
                    Statement::Function(function_decl) => function_decl,
                    _ => unreachable!(),
                };

                variable_bindings.push(VariableBinding {
                    kind: VariableDeclarationKind::Var,
                    name: VariableDeclarationName::Identifier(name),
                    ty: None,
                });
                prepend_function_assigns.push(Statement::Expression(Expr::assignment(
                    Expr::identifier(Cow::Borrowed(name)),
                    Expr::Function(function_stmt),
                    TokenType::Assignment,
                )));
            }
            Statement::Variable(VariableDeclarations(declarations)) => {
                for VariableDeclaration { binding, .. } in declarations {
                    variable_bindings.push(binding.clone());
                }
            }
            Statement::Class(class_decl) => {
                let name = class_decl.name.expect("Class statement did not have a name");

                variable_bindings.push(VariableBinding {
                    kind: VariableDeclarationKind::Var,
                    name: VariableDeclarationName::Identifier(name),
                    ty: None,
                });
            }
            Statement::Block(BlockStatement(stmts)) => {
                let mut prepend = Vec::new();
                for stmt in stmts.iter_mut() {
                    hoist_function_declaration(variable_bindings, &mut prepend, stmt);
                }

                if !stmts.is_empty() {
                    stmts.insert(0, Statement::Block(BlockStatement(prepend)));
                }
            }
            Statement::If(if_stmt) => {
                hoist_function_declaration(variable_bindings, prepend_function_assigns, &mut if_stmt.then);
                if let Some(else_stmt) = &mut if_stmt.el {
                    hoist_function_declaration(variable_bindings, prepend_function_assigns, else_stmt);
                }
            }
            Statement::Loop(Loop::For(for_stmt)) => {
                hoist_function_declaration(variable_bindings, prepend_function_assigns, &mut for_stmt.body);
                if let Some(init_stmt) = &mut for_stmt.init {
                    hoist_function_declaration(variable_bindings, prepend_function_assigns, init_stmt);
                }
            }
            Statement::Loop(Loop::ForIn(for_stmt)) => {
                hoist_function_declaration(variable_bindings, prepend_function_assigns, &mut for_stmt.body);
            }
            Statement::Loop(Loop::ForOf(for_stmt)) => {
                hoist_function_declaration(variable_bindings, prepend_function_assigns, &mut for_stmt.body);
            }
            Statement::Loop(Loop::While(while_stmt)) => {
                hoist_function_declaration(variable_bindings, prepend_function_assigns, &mut while_stmt.body);
            }
            Statement::Try(tc_stmt) => {
                hoist_function_declaration(variable_bindings, prepend_function_assigns, &mut tc_stmt.try_);
                hoist_function_declaration(variable_bindings, prepend_function_assigns, &mut tc_stmt.catch.body);
                if let Some(finally_stmt) = &mut tc_stmt.finally {
                    hoist_function_declaration(variable_bindings, prepend_function_assigns, finally_stmt);
                }
            }
            _ => {}
        }
    }

    for node in ast.iter_mut() {
        hoist_function_declaration(&mut vars, &mut prepend_function_assigns, node);
    }

    if !ast.is_empty() {
        ast.insert(0, Statement::Block(BlockStatement(prepend_function_assigns)));
    }

    vars
}
