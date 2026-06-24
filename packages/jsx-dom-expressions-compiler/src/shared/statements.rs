use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::{Expression, Statement};
use oxc_ast_visit::VisitMut;
use oxc_span::GetSpan;

use crate::dom::element::AstDomTransform;
use crate::shared::fragment::lower_fragment;

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn process_statements(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>) {
        let current_depth = self.statement_depth;
        self.statement_depth += 1;
        let mut body = ArenaVec::new_in(self.allocator);
        for mut statement in statements.drain(..) {
            if self.error.is_some() {
                body.push(statement);
                continue;
            }

            if let Some(setup) = self.lower_variable_jsx_initializer(&mut statement) {
                body.extend(setup);
                self.collect_static_bindings(&statement);
                if current_depth <= 1 {
                    if let Some(capture) = self.take_this_capture_statement(statement.span()) {
                        body.push(capture);
                        self.clear_this_capture_context();
                    }
                }
                body.push(statement);
                continue;
            }

            if let Some(setup) = self.lower_return_jsx(&mut statement) {
                body.extend(setup);
                if current_depth <= 1 {
                    if let Some(capture) = self.take_this_capture_statement(statement.span()) {
                        body.push(capture);
                        self.clear_this_capture_context();
                    }
                }
                body.push(statement);
                continue;
            }

            self.visit_statement(&mut statement);
            self.collect_static_bindings(&statement);
            if current_depth <= 1 {
                if let Some(capture) = self.take_this_capture_statement(statement.span()) {
                    body.push(capture);
                    self.clear_this_capture_context();
                }
            }
            body.push(statement);
        }
        *statements = body;
        self.statement_depth -= 1;
    }

    fn lower_variable_jsx_initializer(
        &mut self,
        statement: &mut Statement<'a>,
    ) -> Option<std::vec::Vec<Statement<'a>>> {
        let Statement::VariableDeclaration(declaration) = statement else {
            return None;
        };
        if declaration.declarations.len() != 1 {
            return None;
        }

        let init = declaration.declarations[0].init.take()?;
        match init {
            Expression::JSXElement(element) => match self.lower_element_with_setup(&element) {
                Ok((replacement, setup)) => {
                    declaration.declarations[0].init = Some(replacement);
                    Some(setup)
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    declaration.declarations[0].init = Some(Expression::JSXElement(element));
                    Some(std::vec::Vec::new())
                }
            },
            Expression::JSXFragment(fragment) => match lower_fragment(self, &fragment) {
                Ok(replacement) => {
                    declaration.declarations[0].init = Some(replacement);
                    Some(std::vec::Vec::new())
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    declaration.declarations[0].init = Some(Expression::JSXFragment(fragment));
                    Some(std::vec::Vec::new())
                }
            },
            init => {
                declaration.declarations[0].init = Some(init);
                None
            }
        }
    }

    fn lower_return_jsx(
        &mut self,
        statement: &mut Statement<'a>,
    ) -> Option<std::vec::Vec<Statement<'a>>> {
        let Statement::ReturnStatement(return_statement) = statement else {
            return None;
        };
        let argument = return_statement.argument.take()?;
        match argument {
            Expression::JSXElement(element) => match self.lower_element_with_setup(&element) {
                Ok((replacement, setup)) => {
                    return_statement.argument = Some(replacement);
                    Some(setup)
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    return_statement.argument = Some(Expression::JSXElement(element));
                    Some(std::vec::Vec::new())
                }
            },
            Expression::JSXFragment(fragment) => match lower_fragment(self, &fragment) {
                Ok(replacement) => {
                    return_statement.argument = Some(replacement);
                    Some(std::vec::Vec::new())
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    return_statement.argument = Some(Expression::JSXFragment(fragment));
                    Some(std::vec::Vec::new())
                }
            },
            Expression::ParenthesizedExpression(parenthesized) => match parenthesized
                .unbox()
                .expression
            {
                Expression::JSXElement(element) => match self.lower_element_with_setup(&element) {
                    Ok((replacement, setup)) => {
                        return_statement.argument = Some(replacement);
                        Some(setup)
                    }
                    Err(error) => {
                        self.error = Some(error.to_string());
                        return_statement.argument = Some(Expression::JSXElement(element));
                        Some(std::vec::Vec::new())
                    }
                },
                Expression::JSXFragment(fragment) => match lower_fragment(self, &fragment) {
                    Ok(replacement) => {
                        return_statement.argument = Some(replacement);
                        Some(std::vec::Vec::new())
                    }
                    Err(error) => {
                        self.error = Some(error.to_string());
                        return_statement.argument = Some(Expression::JSXFragment(fragment));
                        Some(std::vec::Vec::new())
                    }
                },
                expression => {
                    return_statement.argument = Some(expression);
                    None
                }
            },
            argument => {
                return_statement.argument = Some(argument);
                None
            }
        }
    }

    pub(crate) fn lower_class_field_value(
        &mut self,
        span: oxc_span::Span,
        value: Expression<'a>,
    ) -> Expression<'a> {
        match value {
            Expression::JSXElement(element) => match self.lower_element_with_setup(&element) {
                Ok((replacement, mut setup)) => {
                    if let Some(capture) = self.take_this_capture_statement(span) {
                        setup.insert(0, capture);
                        self.clear_this_capture_context();
                    }
                    self.setup_iife(span, setup, replacement)
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    Expression::JSXElement(element)
                }
            },
            Expression::JSXFragment(fragment) => match lower_fragment(self, &fragment) {
                Ok(replacement) => replacement,
                Err(error) => {
                    self.error = Some(error.to_string());
                    Expression::JSXFragment(fragment)
                }
            },
            Expression::ArrowFunctionExpression(mut arrow) if arrow.expression => {
                if arrow.body.statements.len() != 1 {
                    return Expression::ArrowFunctionExpression(arrow);
                }
                let Some(Statement::ExpressionStatement(statement)) = arrow.body.statements.pop()
                else {
                    return Expression::ArrowFunctionExpression(arrow);
                };
                let expression = statement.unbox().expression;
                match expression {
                    Expression::JSXElement(element) => {
                        match self.lower_element_with_setup(&element) {
                            Ok((replacement, setup)) => {
                                arrow.expression = false;
                                let mut statements = self.ast().vec();
                                if let Some(capture) = self.take_this_capture_statement(span) {
                                    statements.push(capture);
                                    self.clear_this_capture_context();
                                }
                                statements.extend(setup);
                                statements
                                    .push(self.ast().statement_return(span, Some(replacement)));
                                arrow.body.statements = statements;
                                Expression::ArrowFunctionExpression(arrow)
                            }
                            Err(error) => {
                                self.error = Some(error.to_string());
                                Expression::ArrowFunctionExpression(arrow)
                            }
                        }
                    }
                    expression => {
                        arrow.body.statements = self
                            .ast()
                            .vec1(self.ast().statement_expression(span, expression));
                        Expression::ArrowFunctionExpression(arrow)
                    }
                }
            }
            value => {
                let mut value = value;
                self.visit_expression(&mut value);
                value
            }
        }
    }

    fn setup_iife(
        &self,
        span: oxc_span::Span,
        setup: std::vec::Vec<Statement<'a>>,
        result: Expression<'a>,
    ) -> Expression<'a> {
        if setup.is_empty() {
            return result;
        }
        let mut statements = self.ast().vec();
        statements.extend(setup);
        statements.push(self.ast().statement_return(span, Some(result)));
        let arrow = self.arrow_iife(span, statements);
        self.call_expression(span, arrow, std::vec::Vec::new())
    }
}
