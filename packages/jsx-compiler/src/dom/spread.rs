use napi::bindgen_prelude::*;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeValue, ObjectPropertyKind, Statement};
use oxc_span::Span;

use crate::dom::element::{jsx_expression_to_expression, AstDomTransform};
use crate::shared::component_props::{component_spread_expression, flush_component_props};
use crate::shared::utils::decode_html_entities;

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn spread_attribute_statement(
        &mut self,
        attributes: &[JSXAttributeItem<'a>],
        element_id: &str,
        skip_children: bool,
    ) -> Result<Statement<'a>> {
        self.template_state.uses_spread = true;
        // A spread may carry delegated event handlers, which can't be known at
        // compile time; hydratable roots must replay events (Babel parity).
        if self.hydratable {
            self.has_hydratable_event = true;
        }
        let mut prop_objects = std::vec::Vec::new();
        let mut running_props = std::vec::Vec::new();
        for attr in attributes {
            match attr {
                JSXAttributeItem::SpreadAttribute(spread) => {
                    flush_component_props(self, &mut running_props, &mut prop_objects, spread.span);
                    prop_objects.push(
                        component_spread_expression(self, &spread.argument, spread.span).value,
                    );
                }
                JSXAttributeItem::Attribute(attr) => {
                    running_props.push(self.spread_attribute_property(attr)?);
                }
            }
        }
        flush_component_props(self, &mut running_props, &mut prop_objects, Span::default());

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
