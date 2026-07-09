use napi::bindgen_prelude::*;
use oxc_allocator::CloneIn;
use oxc_ast::ast::{
    Expression, JSXAttributeItem, JSXAttributeValue, ObjectPropertyKind, PropertyKey, Statement,
};
use oxc_span::Span;

use crate::dom::element::{jsx_expression_to_expression, AstDomTransform};
use crate::shared::utils::{format_number, is_dynamic_attribute_expression};

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn no_inline_style_attribute_statement(
        &mut self,
        attr: &JSXAttributeItem<'a>,
        element_id: &str,
    ) -> Result<Option<Statement<'a>>> {
        if self.inline_styles {
            return Ok(None);
        }
        let JSXAttributeItem::Attribute(attr) = attr else {
            return Ok(None);
        };
        let oxc_ast::ast::JSXAttributeName::Identifier(name) = &attr.name else {
            return Ok(None);
        };
        if name.name != "style" {
            return Ok(None);
        }

        let value = match &attr.value {
            Some(JSXAttributeValue::StringLiteral(value)) => {
                self.ast()
                    .expression_string_literal(attr.span, self.ast().atom(&value.value), None)
            }
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                jsx_expression_to_expression(&container.expression, self.allocator)
            }
            None => self
                .ast()
                .expression_string_literal(attr.span, self.ast().atom(""), None),
            Some(JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_)) => {
                return Err(Error::from_reason(
                    "JSX style attribute element values are not implemented in the AST-native milestone yet",
                ));
            }
        };

        // The Babel plugin always effect-wraps style in no-inline-styles mode,
        // even for fully static values.
        Ok(Some(self.dynamic_style_statement_with_wrap(
            attr.span, element_id, value, true,
        )))
    }

    pub(crate) fn style_object_attribute_operations(
        &mut self,
        attr: &JSXAttributeItem<'a>,
        element_id: &str,
        template: &mut String,
        operations: &mut std::vec::Vec<Statement<'a>>,
    ) -> Result<bool> {
        let JSXAttributeItem::Attribute(attr) = attr else {
            return Ok(false);
        };
        let oxc_ast::ast::JSXAttributeName::Identifier(name) = &attr.name else {
            return Ok(false);
        };
        if name.name != "style" {
            return Ok(false);
        }
        if !self.inline_styles {
            return Ok(false);
        }
        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            return Ok(false);
        };
        let oxc_ast::ast::JSXExpression::ObjectExpression(object) = &container.expression else {
            return Ok(false);
        };

        let mut style = String::new();
        let mut style_operations = std::vec::Vec::new();
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return Ok(false);
            };
            if property.computed {
                return Ok(false);
            }
            let Some(key) = static_style_key(&property.key) else {
                return Ok(false);
            };
            if let Some(value) = static_style_value(&property.value) {
                let Some(value) = value else {
                    continue;
                };
                if !style.is_empty() {
                    style.push(';');
                }
                style.push_str(&key);
                style.push(':');
                style.push_str(&value);
                continue;
            }

            style_operations.push(self.style_property_statement(
                attr.span,
                element_id,
                &key,
                property.value.clone_in(self.allocator),
            ));
        }

        if !style.is_empty() {
            self.append_static_attribute_value(template, "style", &style);
        }
        operations.extend(style_operations);
        Ok(true)
    }

    pub(crate) fn dynamic_style_statement(
        &mut self,
        span: Span,
        element_id: &str,
        value: Expression<'a>,
    ) -> Statement<'a> {
        let wrap = is_dynamic_attribute_expression(&value);
        self.dynamic_style_statement_with_wrap(span, element_id, value, wrap)
    }

    fn dynamic_style_statement_with_wrap(
        &mut self,
        span: Span,
        element_id: &str,
        value: Expression<'a>,
        wrap: bool,
    ) -> Statement<'a> {
        self.template_state.uses_style = true;
        if !self.effect_wrapper || !wrap {
            return self.ast().statement_expression(
                span,
                self.call_identifier(
                    span,
                    "_$style",
                    vec![self.identifier_expression(span, element_id), value],
                ),
            );
        }

        self.template_state.uses_effect = true;
        let getter = self.arrow_with_return(span, std::vec::Vec::new(), value);
        let value_id = "_v$";
        let previous_id = "_$p";
        let statement = self.ast().statement_expression(
            span,
            self.call_identifier(
                span,
                "_$style",
                vec![
                    self.identifier_expression(span, element_id),
                    self.identifier_expression(span, value_id),
                    self.identifier_expression(span, previous_id),
                ],
            ),
        );
        let setter = self.arrow_with_statements(
            span,
            vec![value_id, previous_id],
            self.ast().vec1(statement),
        );
        self.ast().statement_expression(
            span,
            self.call_identifier(span, "_$effect", vec![getter, setter]),
        )
    }

    fn style_property_statement(
        &mut self,
        span: Span,
        element_id: &str,
        name: &str,
        value: Expression<'a>,
    ) -> Statement<'a> {
        self.template_state.uses_set_style_property = true;
        if self.effect_wrapper && is_dynamic_attribute_expression(&value) {
            self.template_state.uses_effect = true;
            let getter = self.arrow_with_return(span, std::vec::Vec::new(), value);
            let value_id = "_v$";
            let statement = self.style_property_call_statement(
                span,
                element_id,
                name,
                self.identifier_expression(span, value_id),
            );
            let setter =
                self.arrow_with_statements(span, vec![value_id], self.ast().vec1(statement));
            return self.ast().statement_expression(
                span,
                self.call_identifier(span, "_$effect", vec![getter, setter]),
            );
        }

        self.style_property_call_statement(span, element_id, name, value)
    }

    fn style_property_call_statement(
        &self,
        span: Span,
        element_id: &str,
        name: &str,
        value: Expression<'a>,
    ) -> Statement<'a> {
        self.ast().statement_expression(
            span,
            self.call_identifier(
                span,
                "_$setStyleProperty",
                vec![
                    self.identifier_expression(span, element_id),
                    self.ast()
                        .expression_string_literal(span, self.ast().atom(name), None),
                    value,
                ],
            ),
        )
    }
}

pub(crate) fn static_style_object_value(
    expression: &oxc_ast::ast::JSXExpression<'_>,
) -> Option<String> {
    let oxc_ast::ast::JSXExpression::ObjectExpression(object) = expression else {
        return None;
    };
    let mut style = String::new();
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed {
            return None;
        }
        let key = static_style_key(&property.key)?;
        let value = static_style_value(&property.value)?;
        let Some(value) = value else {
            continue;
        };
        if !style.is_empty() {
            style.push(';');
        }
        style.push_str(&key);
        style.push(':');
        style.push_str(&value);
    }
    Some(style)
}

pub(crate) fn static_style_key(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.to_string()),
        PropertyKey::StringLiteral(value) => Some(value.value.to_string()),
        PropertyKey::NumericLiteral(value) => Some(value.value.to_string()),
        _ => None,
    }
}

pub(crate) fn static_style_value(value: &Expression<'_>) -> Option<Option<String>> {
    match value {
        Expression::StringLiteral(value) => Some(Some(value.value.to_string())),
        Expression::NumericLiteral(value) => Some(Some(format_number(value.value))),
        Expression::NullLiteral(_) => Some(None),
        Expression::Identifier(identifier) if identifier.name == "undefined" => Some(None),
        _ => None,
    }
}

