use oxc_allocator::CloneIn;
use oxc_ast::ast::{Expression, ObjectPropertyKind, Statement};
use oxc_span::Span;

use crate::dom::element::AstDomTransform;

pub(crate) fn component_ref_property<'a>(
    ctx: &mut AstDomTransform<'a, '_>,
    span: Span,
    value: Expression<'a>,
    setup: &mut std::vec::Vec<Statement<'a>>,
) -> Option<ObjectPropertyKind<'a>> {
    if let Expression::Identifier(identifier) = &value {
        let name = identifier.name.to_string();
        if ctx.const_bindings.iter().any(|binding| binding == &name)
            && !ctx.function_bindings.iter().any(|binding| binding == &name)
        {
            return Some(ctx.object_property(span, "ref", value));
        }
    }
    if matches!(
        value,
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    ) {
        return Some(ctx.object_property(span, "ref", value));
    }

    if let Expression::CallExpression(_) = &value {
        ctx.template_state.uses_apply_ref = true;
        let ref_id = ctx.next_ref_id();
        setup.push(ctx.variable_statement(span, &ref_id, value));
        let ref_identifier = ctx.identifier_expression(span, &ref_id);
        let mut statements = ctx.ast().vec();
        let typeof_ref = ref_callable_test(ctx, span, ref_identifier.clone_in(ctx.allocator));
        let apply_ref = ctx.call_identifier(
            span,
            "_$applyRef",
            vec![ref_identifier, ctx.identifier_expression(span, "r$")],
        );
        statements.push(ctx.ast().statement_expression(
            span,
            ctx.ast().expression_logical(
                span,
                typeof_ref,
                oxc_ast::ast::LogicalOperator::And,
                apply_ref,
            ),
        ));
        return Some(ctx.object_method_property(span, "ref", "r$", statements));
    }

    let ref_id = ctx.next_ref_id();
    let ref_identifier = ctx.identifier_expression(span, &ref_id);
    let value_identifier = ctx.identifier_expression(span, "r$");
    let mut statements = ctx.ast().vec();
    statements.push(ctx.variable_statement(span, &ref_id, value.clone_in(ctx.allocator)));
    let test = ref_callable_test(ctx, span, ref_identifier.clone_in(ctx.allocator));
    let assign_fallback =
        ref_assignment_fallback(ctx, span, &value, ctx.identifier_expression(span, "r$"))?;
    ctx.template_state.uses_apply_ref = true;
    let apply_ref = ctx.call_identifier(
        span,
        "_$applyRef",
        vec![ref_identifier, value_identifier.clone_in(ctx.allocator)],
    );
    statements.push(
        ctx.ast().statement_expression(
            span,
            ctx.ast()
                .expression_conditional(span, test, apply_ref, assign_fallback),
        ),
    );
    Some(ctx.object_method_property(span, "ref", "r$", statements))
}

pub(crate) fn ref_callable_test<'a>(
    ctx: &AstDomTransform<'a, '_>,
    span: Span,
    value: Expression<'a>,
) -> Expression<'a> {
    let typeof_ref = ctx.ast().expression_binary(
        span,
        ctx.ast().expression_unary(
            span,
            oxc_ast::ast::UnaryOperator::Typeof,
            value.clone_in(ctx.allocator),
        ),
        oxc_ast::ast::BinaryOperator::StrictEquality,
        ctx.ast()
            .expression_string_literal(span, ctx.ast().atom("function"), None),
    );
    let array_is_array = ctx.call_expression(
        span,
        ctx.static_member_expression_from_expression(
            span,
            ctx.identifier_expression(span, "Array"),
            "isArray",
        ),
        vec![value],
    );
    ctx.ast().expression_logical(
        span,
        typeof_ref,
        oxc_ast::ast::LogicalOperator::Or,
        array_is_array,
    )
}

pub(crate) fn ref_assignment_fallback<'a>(
    ctx: &AstDomTransform<'a, '_>,
    span: Span,
    value: &Expression<'a>,
    assignment_value: Expression<'a>,
) -> Option<Expression<'a>> {
    match value {
        Expression::Identifier(identifier) => {
            Some(ctx.ast().expression_assignment(
                span,
                oxc_ast::ast::AssignmentOperator::Assign,
                oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(
                    ctx.ast().alloc_identifier_reference(
                        identifier.span,
                        ctx.ast().ident(&identifier.name),
                    ),
                ),
                assignment_value,
            ))
        }
        Expression::StaticMemberExpression(member) if member.optional => {
            let Expression::Identifier(object) = &member.object else {
                return None;
            };
            let test = ctx.ast().expression_unary(
                span,
                oxc_ast::ast::UnaryOperator::LogicalNot,
                ctx.ast().expression_unary(
                    span,
                    oxc_ast::ast::UnaryOperator::LogicalNot,
                    ctx.identifier_expression(span, &object.name),
                ),
            );
            let mut non_optional = member.clone_in(ctx.allocator);
            non_optional.optional = false;
            let assign = ctx.ast().expression_assignment(
                span,
                oxc_ast::ast::AssignmentOperator::Assign,
                ctx.assignment_target_from_static_member(&non_optional),
                assignment_value,
            );
            Some(ctx.ast().expression_logical(
                span,
                test,
                oxc_ast::ast::LogicalOperator::And,
                assign,
            ))
        }
        Expression::StaticMemberExpression(member) => Some(ctx.ast().expression_assignment(
            span,
            oxc_ast::ast::AssignmentOperator::Assign,
            ctx.assignment_target_from_static_member(member),
            assignment_value,
        )),
        Expression::ComputedMemberExpression(member) if !member.optional => {
            Some(ctx.ast().expression_assignment(
                span,
                oxc_ast::ast::AssignmentOperator::Assign,
                oxc_ast::ast::AssignmentTarget::ComputedMemberExpression(
                    member.clone_in(ctx.allocator),
                ),
                assignment_value,
            ))
        }
        Expression::ChainExpression(chain) => {
            let oxc_ast::ast::ChainElement::StaticMemberExpression(member) = &chain.expression
            else {
                return None;
            };
            if !member.optional {
                return None;
            }
            let Expression::Identifier(object) = &member.object else {
                return None;
            };
            let test = ctx.ast().expression_unary(
                span,
                oxc_ast::ast::UnaryOperator::LogicalNot,
                ctx.ast().expression_unary(
                    span,
                    oxc_ast::ast::UnaryOperator::LogicalNot,
                    ctx.identifier_expression(span, &object.name),
                ),
            );
            let mut non_optional = member.clone_in(ctx.allocator);
            non_optional.optional = false;
            let assign = ctx.ast().expression_assignment(
                span,
                oxc_ast::ast::AssignmentOperator::Assign,
                ctx.assignment_target_from_static_member(&non_optional),
                assignment_value,
            );
            Some(ctx.ast().expression_logical(
                span,
                test,
                oxc_ast::ast::LogicalOperator::And,
                assign,
            ))
        }
        _ => None,
    }
}
