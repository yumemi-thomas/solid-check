use napi::bindgen_prelude::*;
use oxc_ast::ast::{
    Expression, JSXAttributeItem, JSXAttributeValue, JSXElement, JSXElementName, JSXExpression,
    JSXMemberExpression, JSXMemberExpressionObject, Statement,
};
use oxc_ast_visit::{walk_mut, VisitMut};
use oxc_span::{GetSpan, Span};

use crate::dom::element::jsx_expression_to_expression;
use crate::dom::element::AstDomTransform;
use crate::shared::component_children::component_children;
use crate::shared::component_props::{
    component_property, component_props_expression, component_spread_expression,
    flush_component_props,
};
use crate::shared::refs::component_ref_property;
use crate::shared::utils::decode_html_entities;

pub(crate) fn lower_component_with_setup<'a>(
    ctx: &mut AstDomTransform<'a, '_>,
    element: &JSXElement<'a>,
) -> Result<(Expression<'a>, std::vec::Vec<Statement<'a>>)> {
    ctx.template_state.uses_create_component = true;
    let component = component_callee_expression(ctx, &element.opening_element.name)?;
    let mut prop_objects = std::vec::Vec::new();
    let mut running_props = std::vec::Vec::new();
    let mut force_merge_props = false;
    let mut setup = std::vec::Vec::new();

    for attr in &element.opening_element.attributes {
        let attr = match attr {
            JSXAttributeItem::Attribute(attr) => attr,
            JSXAttributeItem::SpreadAttribute(spread) => {
                flush_component_props(ctx, &mut running_props, &mut prop_objects, element.span);
                let spread = component_spread_expression(ctx, &spread.argument, spread.span);
                force_merge_props = force_merge_props || spread.force_merge;
                prop_objects.push(spread.value);
                continue;
            }
        };
        let name = match &attr.name {
            oxc_ast::ast::JSXAttributeName::Identifier(name) => name.name.to_string(),
            _ => {
                return Err(Error::from_reason(
                    "Component namespace attributes are not implemented in the AST-native milestone yet",
                ));
            }
        };
        let (value, needs_getter) = match &attr.value {
            None => (ctx.ast().expression_boolean_literal(attr.span, true), false),
            Some(JSXAttributeValue::StringLiteral(value)) => {
                let span = value.span;
                let value = decode_html_entities(&value.value);
                (
                    ctx.ast()
                        .expression_string_literal(span, ctx.ast().atom(&value), None),
                    false,
                )
            }
            Some(JSXAttributeValue::ExpressionContainer(container)) => (
                transform_component_expression(ctx, &container.expression),
                component_prop_requires_getter(ctx, &name, container),
            ),
            _ => {
                return Err(Error::from_reason(
                    "Component JSX attribute values are not implemented in the AST-native milestone yet",
                ));
            }
        };
        if name == "ref" {
            if let Some(ref_property) = component_ref_property(ctx, attr.span, value, &mut setup) {
                running_props.push(ref_property);
            }
        } else {
            running_props.push(component_property(
                ctx,
                attr.span,
                &name,
                value,
                needs_getter,
            ));
        }
    }

    let children = component_children(ctx, &element.children)?;
    if let Some(children) = children {
        if children.needs_getter {
            running_props.push(ctx.object_getter_property_with_setup(
                element.span,
                "children",
                children.setup,
                children.value,
            ));
        } else {
            running_props.push(ctx.object_property(element.span, "children", children.value));
        }
    }

    flush_component_props(ctx, &mut running_props, &mut prop_objects, element.span);
    let props = component_props_expression(ctx, element.span, prop_objects, force_merge_props);
    Ok((
        ctx.call_identifier(element.span, "_$createComponent", vec![component, props]),
        setup,
    ))
}

fn component_prop_requires_getter(
    ctx: &AstDomTransform<'_, '_>,
    name: &str,
    container: &oxc_ast::ast::JSXExpressionContainer<'_>,
) -> bool {
    if name == "ref" {
        return false;
    }
    if crate::shared::utils::source_from_span(container.span, ctx.source)
        .contains(&ctx.static_marker)
    {
        return false;
    }
    matches!(
        container.expression,
        JSXExpression::StaticMemberExpression(_)
            | JSXExpression::ComputedMemberExpression(_)
            | JSXExpression::ChainExpression(_)
            | JSXExpression::JSXElement(_)
            | JSXExpression::CallExpression(_)
            | JSXExpression::ConditionalExpression(_)
            | JSXExpression::LogicalExpression(_)
    )
}

pub(crate) fn transform_component_expression<'a>(
    ctx: &mut AstDomTransform<'a, '_>,
    expression: &JSXExpression<'a>,
) -> Expression<'a> {
    let mut expression = jsx_expression_to_expression(expression, ctx.allocator);
    replace_this_expression(ctx, &mut expression);
    ctx.visit_expression(&mut expression);
    expression = ctx.condition_component_expression(expression.span(), expression);
    expression
}

fn replace_this_expression<'a>(ctx: &mut AstDomTransform<'a, '_>, expression: &mut Expression<'a>) {
    struct ThisReplacer<'ctx, 'a, 'source> {
        ctx: &'ctx mut AstDomTransform<'a, 'source>,
    }

    impl<'a> VisitMut<'a> for ThisReplacer<'_, 'a, '_> {
        fn visit_expression(&mut self, expression: &mut Expression<'a>) {
            if let Expression::ThisExpression(this) = expression {
                *expression = self.ctx.capture_this_expression(this.span);
                return;
            }
            walk_mut::walk_expression(self, expression);
        }
    }

    ThisReplacer { ctx }.visit_expression(expression);
}

fn component_callee_expression<'a>(
    ctx: &mut AstDomTransform<'a, '_>,
    name: &JSXElementName<'a>,
) -> Result<Expression<'a>> {
    match name {
        JSXElementName::Identifier(identifier) => {
            Ok(ctx.component_identifier_expression(&identifier.name))
        }
        JSXElementName::IdentifierReference(identifier) => {
            Ok(ctx.component_identifier_expression(&identifier.name))
        }
        JSXElementName::MemberExpression(member) => component_member_expression(ctx, member),
        JSXElementName::ThisExpression(this) => Ok(ctx.capture_this_expression(this.span)),
        JSXElementName::NamespacedName(_) => Err(Error::from_reason(
            "Component namespace attributes are not implemented in the AST-native milestone yet",
        )),
    }
}

fn component_member_expression<'a>(
    ctx: &mut AstDomTransform<'a, '_>,
    member: &JSXMemberExpression<'a>,
) -> Result<Expression<'a>> {
    let object = match &member.object {
        JSXMemberExpressionObject::IdentifierReference(identifier) => {
            ctx.component_identifier_expression(&identifier.name)
        }
        JSXMemberExpressionObject::MemberExpression(member) => {
            component_member_expression(ctx, member)?
        }
        JSXMemberExpressionObject::ThisExpression(this) => ctx.capture_this_expression(this.span),
    };
    Ok(member_property_expression(
        ctx,
        member.span,
        object,
        &member.property.name,
    ))
}

fn member_property_expression<'a>(
    ctx: &AstDomTransform<'a, '_>,
    span: Span,
    object: Expression<'a>,
    property: &str,
) -> Expression<'a> {
    if crate::shared::utils::is_identifier_key(property) {
        ctx.static_member_expression_from_expression(span, object, property)
    } else {
        Expression::ComputedMemberExpression(
            ctx.ast().alloc_computed_member_expression(
                span,
                object,
                ctx.ast()
                    .expression_string_literal(span, ctx.ast().atom(property), None),
                false,
            ),
        )
    }
}

impl<'a> AstDomTransform<'a, '_> {
    fn component_identifier_expression(&mut self, component: &str) -> Expression<'a> {
        if self.built_ins.iter().any(|built_in| built_in == component) {
            if !self
                .template_state
                .built_in_imports
                .iter()
                .any(|built_in| built_in == component)
            {
                self.template_state
                    .built_in_imports
                    .push(component.to_string());
            }
            self.identifier_expression(Span::new(0, 0), &format!("_${component}"))
        } else {
            self.identifier_expression(Span::new(0, 0), component)
        }
    }
}
