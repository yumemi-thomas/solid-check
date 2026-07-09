use napi::bindgen_prelude::*;
use oxc_allocator::CloneIn;
use oxc_ast::ast::{
    ArrayExpressionElement, Expression, JSXAttributeItem, JSXAttributeValue, ObjectPropertyKind,
    Statement,
};
use oxc_span::Span;

use crate::dom::element::AstDomTransform;
use crate::dom::style::static_style_key;
use crate::shared::utils::is_dynamic_attribute_expression;

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn class_array_attribute_operations(
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
        if name.name != "class" && name.name != "className" {
            return Ok(false);
        }
        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            return Ok(false);
        };
        let oxc_ast::ast::JSXExpression::ArrayExpression(array) = &container.expression else {
            return Ok(false);
        };

        let mut static_classes = std::vec::Vec::new();
        let mut class_operations = std::vec::Vec::new();
        for element in &array.elements {
            match element {
                ArrayExpressionElement::StringLiteral(value) => {
                    let value = value.value.trim();
                    if !value.is_empty() {
                        static_classes.push(value.to_string());
                    }
                }
                ArrayExpressionElement::ObjectExpression(object) => {
                    for property in &object.properties {
                        let ObjectPropertyKind::ObjectProperty(property) = property else {
                            return Ok(false);
                        };
                        if property.computed {
                            return Ok(false);
                        }
                        let Some(class_name) = static_style_key(&property.key) else {
                            return Ok(false);
                        };
                        if let Some(include) = static_truthy_expression(&property.value) {
                            if include {
                                static_classes.push(class_name);
                            }
                            continue;
                        }
                        class_operations.push(self.class_toggle_statement(
                            attr.span,
                            element_id,
                            &class_name,
                            property.value.clone_in(self.allocator),
                        ));
                    }
                }
                _ => return Ok(false),
            }
        }

        if !static_classes.is_empty() {
            self.append_static_attribute_value(template, "class", &static_classes.join(" "));
        }
        operations.extend(class_operations);
        Ok(true)
    }

    pub(crate) fn dynamic_class_statement(
        &mut self,
        span: Span,
        element_id: &str,
        value: Expression<'a>,
    ) -> Statement<'a> {
        self.template_state.uses_class_name = true;
        if !self.effect_wrapper || !is_dynamic_attribute_expression(&value) {
            return self.ast().statement_expression(
                span,
                self.call_identifier(
                    span,
                    "_$className",
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
                "_$className",
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

    pub(crate) fn class_object_statement(
        &self,
        span: Span,
        element_id: &str,
        expression: &oxc_ast::ast::JSXExpression<'a>,
    ) -> Option<Statement<'a>> {
        let oxc_ast::ast::JSXExpression::ObjectExpression(object) = expression else {
            return None;
        };
        let mut statements = self.ast().vec();
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            if property.computed {
                return None;
            }
            let class_name = static_style_key(&property.key)?;
            let value = property.value.clone_in(self.allocator);
            statements.push(self.class_toggle_call_statement(
                span,
                element_id,
                &class_name,
                self.boolean_expression(span, value),
            ));
        }
        Some(self.ast().statement_block(span, statements))
    }

    fn class_toggle_statement(
        &mut self,
        span: Span,
        element_id: &str,
        class_name: &str,
        value: Expression<'a>,
    ) -> Statement<'a> {
        if self.effect_wrapper && is_dynamic_attribute_expression(&value) {
            self.template_state.uses_effect = true;
            let getter = self.arrow_with_return(
                span,
                std::vec::Vec::new(),
                self.boolean_expression(span, value),
            );
            let value_id = "_v$";
            let statement = self.class_toggle_call_statement(
                span,
                element_id,
                class_name,
                self.identifier_expression(span, value_id),
            );
            let setter =
                self.arrow_with_statements(span, vec![value_id], self.ast().vec1(statement));
            return self.ast().statement_expression(
                span,
                self.call_identifier(span, "_$effect", vec![getter, setter]),
            );
        }

        self.class_toggle_call_statement(
            span,
            element_id,
            class_name,
            self.boolean_expression(span, value),
        )
    }

    fn class_toggle_call_statement(
        &self,
        span: Span,
        element_id: &str,
        class_name: &str,
        value: Expression<'a>,
    ) -> Statement<'a> {
        let toggle = self.call_expression(
            span,
            self.static_member_expression_from_expression(
                span,
                self.static_member_expression(span, element_id, "classList"),
                "toggle",
            ),
            vec![
                self.ast()
                    .expression_string_literal(span, self.ast().atom(class_name), None),
                value,
            ],
        );
        self.ast().statement_expression(span, toggle)
    }

    fn boolean_expression(&self, span: Span, value: Expression<'a>) -> Expression<'a> {
        self.ast().expression_unary(
            span,
            oxc_ast::ast::UnaryOperator::LogicalNot,
            self.ast()
                .expression_unary(span, oxc_ast::ast::UnaryOperator::LogicalNot, value),
        )
    }
}

fn static_truthy_expression(value: &Expression<'_>) -> Option<bool> {
    match value {
        Expression::BooleanLiteral(value) => Some(value.value),
        Expression::StringLiteral(value) => Some(!value.value.is_empty()),
        Expression::NumericLiteral(value) => Some(value.value != 0.0),
        Expression::NullLiteral(_) => Some(false),
        Expression::Identifier(identifier) if identifier.name == "undefined" => Some(false),
        _ => None,
    }
}

