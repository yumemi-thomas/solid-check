use napi::bindgen_prelude::*;
use oxc_allocator::CloneIn;
use oxc_ast::ast::{
    Expression, JSXAttributeItem, JSXAttributeValue, ObjectPropertyKind, Statement,
};
use oxc_span::Span;

use crate::dom::element::{jsx_expression_to_expression, AstDomTransform};
use crate::shared::utils::decode_html_entities;

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn spread_attribute_statement(
        &mut self,
        attributes: &[JSXAttributeItem<'a>],
        element_id: &str,
        skip_children: bool,
    ) -> Result<Statement<'a>> {
        self.template_state.uses_spread = true;
        let mut prop_objects = std::vec::Vec::new();
        let mut running_props = std::vec::Vec::new();
        for attr in attributes {
            match attr {
                JSXAttributeItem::SpreadAttribute(spread) => {
                    flush_object_properties(
                        self,
                        spread.span,
                        &mut running_props,
                        &mut prop_objects,
                    );
                    prop_objects.push(spread_argument_expression(
                        self,
                        &spread.argument,
                        spread.span,
                    ));
                }
                JSXAttributeItem::Attribute(attr) => {
                    running_props.push(self.spread_attribute_property(attr)?);
                }
            }
        }
        flush_object_properties(self, Span::default(), &mut running_props, &mut prop_objects);

        let props = match prop_objects.len() {
            0 => self
                .ast()
                .expression_object(Span::default(), self.ast().vec()),
            1 => prop_objects
                .pop()
                .expect("single spread props object exists"),
            _ => {
                self.template_state.uses_merge_props = true;
                self.call_identifier(Span::default(), "_$mergeProps", prop_objects)
            }
        };

        Ok(self.ast().statement_expression(
            Span::default(),
            self.call_identifier(
                Span::default(),
                "_$spread",
                vec![
                    self.identifier_expression(Span::default(), element_id),
                    props,
                    self.ast()
                        .expression_boolean_literal(Span::default(), skip_children),
                ],
            ),
        ))
    }

    fn spread_attribute_property(
        &mut self,
        attr: &oxc_ast::ast::JSXAttribute<'a>,
    ) -> Result<ObjectPropertyKind<'a>> {
        let name = match &attr.name {
            oxc_ast::ast::JSXAttributeName::Identifier(name) => name.name.to_string(),
            oxc_ast::ast::JSXAttributeName::NamespacedName(name)
                if name.namespace.name == "prop" =>
            {
                name.name.name.to_string()
            }
            oxc_ast::ast::JSXAttributeName::NamespacedName(_) => {
                return Err(Error::from_reason(
                    "Namespaced attributes are not implemented in the AST-native milestone yet",
                ));
            }
        };

        let (value, needs_getter) = match &attr.value {
            None => (
                self.ast().expression_boolean_literal(attr.span, true),
                false,
            ),
            Some(JSXAttributeValue::StringLiteral(value)) => {
                let value = decode_html_entities(&value.value);
                (
                    self.ast()
                        .expression_string_literal(attr.span, self.ast().atom(&value), None),
                    false,
                )
            }
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                let value = jsx_expression_to_expression(&container.expression, self.allocator);
                (
                    value,
                    spread_attribute_requires_getter(self, &name, container),
                )
            }
            Some(JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_)) => {
                return Err(Error::from_reason(
                    "JSX spread attribute object values are not implemented in the AST-native milestone yet",
                ));
            }
        };

        Ok(if needs_getter {
            self.object_getter_property(attr.span, &name, value)
        } else {
            self.object_property(attr.span, &name, value)
        })
    }
}

fn flush_object_properties<'a>(
    ctx: &AstDomTransform<'a, '_>,
    span: Span,
    running_props: &mut std::vec::Vec<ObjectPropertyKind<'a>>,
    prop_objects: &mut std::vec::Vec<Expression<'a>>,
) {
    if running_props.is_empty() {
        return;
    }
    let props = std::mem::take(running_props);
    prop_objects.push(
        ctx.ast()
            .expression_object(span, ctx.ast().vec_from_iter(props)),
    );
}

fn spread_argument_expression<'a>(
    ctx: &AstDomTransform<'a, '_>,
    expression: &Expression<'a>,
    span: Span,
) -> Expression<'a> {
    let expression = expression.clone_in(ctx.allocator);
    if matches!(expression, Expression::CallExpression(_)) {
        ctx.arrow_return_expression(span, expression)
    } else {
        expression
    }
}

fn spread_attribute_requires_getter(
    ctx: &AstDomTransform<'_, '_>,
    name: &str,
    container: &oxc_ast::ast::JSXExpressionContainer<'_>,
) -> bool {
    if crate::shared::utils::source_from_span(container.span, ctx.source)
        .contains(&ctx.static_marker)
    {
        return false;
    }
    matches!(
        container.expression,
        oxc_ast::ast::JSXExpression::StaticMemberExpression(_)
            | oxc_ast::ast::JSXExpression::ComputedMemberExpression(_)
            | oxc_ast::ast::JSXExpression::CallExpression(_)
            | oxc_ast::ast::JSXExpression::ObjectExpression(_)
            | oxc_ast::ast::JSXExpression::ArrayExpression(_)
    ) || name == "class"
        || name == "className"
        || name == "style"
}
