use oxc_allocator::CloneIn;
use oxc_ast::ast::{AssignmentOperator, AssignmentTarget, Expression, JSXExpression, Statement};
use oxc_span::Span;

use crate::dom::element::{jsx_expression_to_expression, AstDomTransform};
use crate::shared::constants::delegated_events;

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn event_statement(
        &mut self,
        span: Span,
        element_id: &str,
        name: &str,
        expression: &JSXExpression<'a>,
    ) -> Statement<'a> {
        let event_name = to_event_name(name);
        let handler = jsx_expression_to_expression(expression, self.allocator);

        let delegated = self.should_delegate_event(&event_name);
        if delegated {
            if let Some((handler, data)) = event_array_data(self, &handler) {
                self.register_delegated_event(&event_name);
                return self.delegated_event_statement(
                    span,
                    element_id,
                    &event_name,
                    handler,
                    Some(data),
                );
            }
            if let Some(handler) = single_event_array_handler(self, &handler) {
                self.register_delegated_event(&event_name);
                return self.delegated_event_statement(
                    span,
                    element_id,
                    &event_name,
                    handler,
                    None,
                );
            }
            if is_function_handler(&handler) || self.is_const_identifier(&handler) {
                self.register_delegated_event(&event_name);
                return self.delegated_event_statement(
                    span,
                    element_id,
                    &event_name,
                    handler,
                    None,
                );
            }
        }

        self.add_event_listener_statement(span, element_id, &event_name, handler, delegated)
    }

    fn should_delegate_event(&self, event_name: &str) -> bool {
        self.delegate_events
            && (delegated_events(event_name)
                || self
                    .delegated_events
                    .iter()
                    .any(|delegated| delegated == event_name))
    }

    fn register_delegated_event(&mut self, event_name: &str) {
        self.template_state.uses_delegate_events = true;
        if !self
            .template_state
            .delegated_events
            .iter()
            .any(|event| event == event_name)
        {
            self.template_state
                .delegated_events
                .push(event_name.to_string());
        }
    }

    fn delegated_event_assignment(
        &self,
        span: Span,
        element_id: &str,
        event_name: &str,
        handler: Expression<'a>,
        data: Option<Expression<'a>>,
    ) -> Statement<'a> {
        let target =
            self.static_member_assignment_target(span, element_id, &format!("$${event_name}"));
        if let Some(data) = data {
            let data_target = self.static_member_assignment_target(
                span,
                element_id,
                &format!("$${event_name}Data"),
            );
            let mut statements = self.ast().vec();
            statements.push(
                self.ast().statement_expression(
                    span,
                    self.ast().expression_assignment(
                        span,
                        AssignmentOperator::Assign,
                        target,
                        handler,
                    ),
                ),
            );
            statements.push(self.ast().statement_expression(
                span,
                self.ast().expression_assignment(
                    span,
                    AssignmentOperator::Assign,
                    data_target,
                    data,
                ),
            ));
            return self.ast().statement_block(span, statements);
        }
        self.ast().statement_expression(
            span,
            self.ast()
                .expression_assignment(span, AssignmentOperator::Assign, target, handler),
        )
    }

    fn delegated_event_statement(
        &mut self,
        span: Span,
        element_id: &str,
        event_name: &str,
        handler: Expression<'a>,
        data: Option<Expression<'a>>,
    ) -> Statement<'a> {
        let assignment =
            self.delegated_event_assignment(span, element_id, event_name, handler, data);
        // Delegated events on SSR'd markup may have fired before hydration;
        // the template root replays them once via `runHydrationEvents()`
        // after all setup (Babel's `hasHydratableEvent` bubbling).
        if self.hydratable {
            self.has_hydratable_event = true;
        }
        assignment
    }

    fn add_event_listener_statement(
        &mut self,
        span: Span,
        element_id: &str,
        event_name: &str,
        mut handler: Expression<'a>,
        delegated: bool,
    ) -> Statement<'a> {
        let event_name_expression =
            self.ast()
                .expression_string_literal(span, self.ast().atom(event_name), None);
        if delegated {
            self.register_delegated_event(event_name);
            self.template_state.uses_add_event_listener = true;
            let mut args = vec![
                self.identifier_expression(span, element_id),
                event_name_expression,
                handler,
            ];
            args.push(self.ast().expression_boolean_literal(span, true));
            return self
                .ast()
                .statement_expression(span, self.call_identifier(span, "_$addEvent", args));
        }

        let mut force_native_listener = false;
        if let Some((event_handler, data)) = event_array_data(self, &handler) {
            handler = self.event_handler_with_data(span, event_handler, data);
        } else if let Some(event_handler) = single_event_array_handler(self, &handler) {
            handler = event_handler;
            force_native_listener = true;
        }

        let callee = self.static_member_expression(span, element_id, "addEventListener");
        if !force_native_listener && self.should_use_add_event_helper(&handler) {
            return self.add_event_helper_statement(span, element_id, event_name, handler, false);
        }
        self.ast().statement_expression(
            span,
            self.call_expression(span, callee, vec![event_name_expression, handler]),
        )
    }

    fn add_event_helper_statement(
        &mut self,
        span: Span,
        element_id: &str,
        event_name: &str,
        handler: Expression<'a>,
        delegated: bool,
    ) -> Statement<'a> {
        self.template_state.uses_add_event_listener = true;
        let mut args = vec![
            self.identifier_expression(span, element_id),
            self.ast()
                .expression_string_literal(span, self.ast().atom(event_name), None),
            handler,
        ];
        if delegated {
            args.push(self.ast().expression_boolean_literal(span, true));
        }
        self.ast()
            .statement_expression(span, self.call_identifier(span, "_$addEvent", args))
    }

    fn should_use_add_event_helper(&self, handler: &Expression<'_>) -> bool {
        let Expression::Identifier(identifier) = handler else {
            return false;
        };
        !self
            .function_bindings
            .iter()
            .any(|binding| binding == identifier.name.as_str())
    }

    fn is_const_identifier(&self, handler: &Expression<'_>) -> bool {
        let Expression::Identifier(identifier) = handler else {
            return false;
        };
        self.const_bindings
            .iter()
            .any(|binding| binding == identifier.name.as_str())
    }

    fn event_handler_with_data(
        &self,
        span: Span,
        handler: Expression<'a>,
        data: Expression<'a>,
    ) -> Expression<'a> {
        let event_name = "e";
        let event_param = self.ast().formal_parameter(
            span,
            self.ast().vec(),
            self.ast()
                .binding_pattern_binding_identifier(span, self.ast().ident(event_name)),
            oxc_ast::NONE,
            oxc_ast::NONE,
            false,
            None,
            false,
            false,
        );
        let params = self.ast().formal_parameters(
            span,
            oxc_ast::ast::FormalParameterKind::ArrowFormalParameters,
            self.ast().vec1(event_param),
            oxc_ast::NONE,
        );
        let call = self.call_expression(
            span,
            handler,
            vec![data, self.identifier_expression(span, event_name)],
        );
        let body = self.ast().function_body(
            span,
            self.ast().vec(),
            self.ast()
                .vec1(self.ast().statement_return(span, Some(call))),
        );
        self.ast().expression_arrow_function(
            span,
            false,
            false,
            oxc_ast::NONE,
            params,
            oxc_ast::NONE,
            body,
        )
    }

    fn static_member_assignment_target(
        &self,
        span: Span,
        object: &str,
        property: &str,
    ) -> AssignmentTarget<'a> {
        AssignmentTarget::StaticMemberExpression(self.ast().alloc_static_member_expression(
            span,
            self.identifier_expression(span, object),
            self.ast().identifier_name(span, self.ast().ident(property)),
            false,
        ))
    }
}

fn is_function_handler(handler: &Expression<'_>) -> bool {
    matches!(
        handler,
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    )
}

fn to_event_name(name: &str) -> String {
    name[2..].to_ascii_lowercase()
}

fn event_array_data<'a>(
    ctx: &AstDomTransform<'a, '_>,
    handler: &Expression<'a>,
) -> Option<(Expression<'a>, Expression<'a>)> {
    let Expression::ArrayExpression(array) = handler else {
        return None;
    };
    let mut elements = array.elements.iter();
    let first = elements.next()?;
    let second = elements.next()?;
    let handler = array_element_expression(ctx, first)?;
    let data = array_element_expression(ctx, second)?;
    Some((handler, data))
}

fn single_event_array_handler<'a>(
    ctx: &AstDomTransform<'a, '_>,
    handler: &Expression<'a>,
) -> Option<Expression<'a>> {
    let Expression::ArrayExpression(array) = handler else {
        return None;
    };
    if array.elements.len() != 1 {
        return None;
    }
    array_element_expression(ctx, array.elements.first()?)
}

fn array_element_expression<'a>(
    ctx: &AstDomTransform<'a, '_>,
    element: &oxc_ast::ast::ArrayExpressionElement<'a>,
) -> Option<Expression<'a>> {
    match element {
        oxc_ast::ast::ArrayExpressionElement::ArrowFunctionExpression(value) => Some(
            Expression::ArrowFunctionExpression(value.clone_in(ctx.allocator)),
        ),
        oxc_ast::ast::ArrayExpressionElement::Identifier(value) => {
            Some(Expression::Identifier(value.clone_in(ctx.allocator)))
        }
        oxc_ast::ast::ArrayExpressionElement::StaticMemberExpression(value) => Some(
            Expression::StaticMemberExpression(value.clone_in(ctx.allocator)),
        ),
        oxc_ast::ast::ArrayExpressionElement::CallExpression(value) => {
            Some(Expression::CallExpression(value.clone_in(ctx.allocator)))
        }
        oxc_ast::ast::ArrayExpressionElement::StringLiteral(value) => {
            Some(Expression::StringLiteral(value.clone_in(ctx.allocator)))
        }
        oxc_ast::ast::ArrayExpressionElement::NumericLiteral(value) => {
            Some(Expression::NumericLiteral(value.clone_in(ctx.allocator)))
        }
        _ => None,
    }
}
