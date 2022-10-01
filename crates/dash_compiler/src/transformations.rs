use dash_middle::parser::statement::BlockStatement;
use dash_middle::parser::statement::FunctionDeclaration;
use dash_middle::parser::statement::ReturnStatement;
use dash_middle::parser::statement::Statement;
use dash_middle::parser::statement::VariableBinding;
use dash_middle::parser::statement::VariableDeclaration;
use dash_middle::parser::statement::VariableDeclarationKind;

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
pub fn find_hoisted_declarations<'a>(ast: &Vec<Statement<'a>>) -> Vec<VariableBinding<'a>> {
    let mut vars = Vec::new();
    for node in ast {
        match node {
            Statement::Variable(VariableDeclaration {
                binding:
                    binding @ VariableBinding {
                        kind: VariableDeclarationKind::Var,
                        ..
                    },
                ..
            }) => {
                vars.push(binding.clone());
            }
            Statement::Function(FunctionDeclaration { name: Some(name), .. }) => {
                vars.push(VariableBinding {
                    name,
                    kind: VariableDeclarationKind::Var,
                    ty: None,
                });
            }
            // TODO: recursively visit all nodes to hoist nested variable declarations
            _ => {}
        }
    }
    vars
}
