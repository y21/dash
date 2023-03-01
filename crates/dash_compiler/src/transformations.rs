use std::borrow::Cow;

use dash_middle::lexer::token::TokenType;
use dash_middle::parser::expr::AssignmentExpr;
use dash_middle::parser::expr::AssignmentTarget;
use dash_middle::parser::expr::Expr;
use dash_middle::parser::expr::PropertyAccessExpr;
use dash_middle::parser::statement::BlockStatement;
use dash_middle::parser::statement::Class;
use dash_middle::parser::statement::ClassMemberKind;
use dash_middle::parser::statement::ClassProperty;
use dash_middle::parser::statement::ReturnStatement;
use dash_middle::parser::statement::Statement;

/// Implicitly patches the last expression to be returned from the function
///
/// Or inserts `return undefined;` if there is no last expression
pub fn ast_patch_implicit_return<'a>(ast: &mut Vec<Statement<'a>>) {
    match ast.last_mut() {
        Some(Statement::Return(..)) => {}
        Some(Statement::Expression(..)) => {
            let expr = match ast.pop() {
                Some(Statement::Expression(expr)) => expr,
                _ => unreachable!(),
            };

            ast.push(Statement::Return(ReturnStatement(expr)));
        }
        Some(Statement::Block(BlockStatement(block))) => ast_patch_implicit_return(block),
        _ => ast_insert_implicit_return(ast),
    }
}

pub fn ast_insert_implicit_return<'a>(ast: &mut Vec<Statement<'a>>) {
    ast.push(Statement::Return(ReturnStatement::default()));
}

/// For every field property, insert a `this.fieldName = fieldValue` expression in the constructor
pub fn insert_initializer_in_constructor<'a>(class: &Class<'a>, statements: &mut Vec<Statement<'a>>) {
    let mut prestatements = Vec::new();
    for member in &class.members {
        if let ClassMemberKind::Property(ClassProperty {
            name,
            value: Some(value),
        }) = &member.kind
        {
            prestatements.push(Statement::Expression(Expr::Assignment(AssignmentExpr {
                left: AssignmentTarget::Expr(Box::new(Expr::PropertyAccess(PropertyAccessExpr {
                    computed: false,
                    property: Box::new(Expr::string_literal(Cow::Borrowed(name))),
                    target: Box::new(Expr::identifier(Cow::Borrowed("this"))),
                }))),
                operator: TokenType::Assignment,
                right: Box::new(value.clone()),
            })));
        }
    }
    prestatements.append(statements);
    *statements = prestatements;
}
