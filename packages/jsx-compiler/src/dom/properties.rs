use napi::bindgen_prelude::*;
use oxc_ast::ast::{
    AssignmentOperator, AssignmentTarget, Expression, JSXAttributeItem, JSXAttributeValue,
    Statement,
};
use oxc_span::Span;

use crate::dom::element::{jsx_expression_to_expression, AstDomTransform};
use crate::shared::constants::child_properties;
use crate::shared::utils::{decode_html_entities, static_jsx_expression_value};

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn prop_attribute_statement(
        &self,
        attr: &JSXAttributeItem<'a>,
        element_id: &str,
    ) -> Result<Option<Statement<'a>>> {
        let JSXAttributeItem::Attribute(attr) = attr else {
            return Ok(None);
        };
        let oxc_ast::ast::JSXAttributeName::NamespacedName(name) = &attr.name else {
            return Ok(None);
        };
        if name.namespace.name != "prop" {
            return Ok(None);
        }

        let property_name = name.name.name.as_str();
        let value = match &attr.value {
            None => self.ast().expression_boolean_literal(attr.span, true),
            Some(JSXAttributeValue::StringLiteral(value)) => self.ast().expression_string_literal(
                value.span,
                self.ast().atom(&decode_html_entities(&value.value)),
                None,
            ),
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                jsx_expression_to_expression(&container.expression, self.allocator)
            }
            Some(JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_)) => {
                return Err(Error::from_reason(
                    "JSX prop:* attribute element values are not implemented in the AST-native milestone yet",
                ));
            }
        };

        Ok(Some(self.static_child_property_statement(
            attr.span,
            element_id,
            property_name,
            value,
        )))
    }

    pub(crate) fn child_property_attribute_statement(
        &mut self,
        attr: &JSXAttributeItem<'a>,
        element_id: &str,
    ) -> Result<Option<Statement<'a>>> {
        let JSXAttributeItem::Attribute(attr) = attr else {
            return Ok(None);
        };
        let oxc_ast::ast::JSXAttributeName::Identifier(name) = &attr.name else {
            return Ok(None);
        };
        if !child_properties(&name.name) {
            return Ok(None);
        }

        let Some(value) = &attr.value else {
            return Ok(Some(self.static_child_property_statement(
                attr.span,
                element_id,
                &name.name,
                self.ast().expression_boolean_literal(attr.span, true),
            )));
        };

        let value = match value {
            JSXAttributeValue::StringLiteral(value) => self.ast().expression_string_literal(
                value.span,
                self.ast().atom(value.value.as_str()),
                None,
            ),
            JSXAttributeValue::ExpressionContainer(container) => {
                let value = self
                    .static_jsx_expression_value(&container.expression)
                    .or_else(|| static_jsx_expression_value(&container.expression));
                if let Some(value) = value {
                    self.ast().expression_string_literal(
                        container.span,
                        self.ast().atom(&value),
                        None,
                    )
                } else {
                    let value = jsx_expression_to_expression(&container.expression, self.allocator);
                    return Ok(Some(self.dynamic_child_property_statement(
                        attr.span, element_id, &name.name, value,
                    )));
                }
            }
            JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_) => {
                return Err(Error::from_reason(
                    "JSX child-property attribute element values are not implemented in the AST-native milestone yet",
                ));
            }
        };

        Ok(Some(self.static_child_property_statement(
            attr.span, element_id, &name.name, value,
        )))
    }

    fn static_child_property_statement(
        &self,
        span: Span,
        element_id: &str,
        name: &str,
        value: Expression<'a>,
    ) -> Statement<'a> {
        self.ast().statement_expression(
            span,
            self.ast().expression_assignment(
                span,
                AssignmentOperator::Assign,
                self.child_property_assignment_target(span, element_id, name),
                value,
            ),
        )
    }

    fn dynamic_child_property_statement(
        &mut self,
        span: Span,
        element_id: &str,
        name: &str,
        value: Expression<'a>,
    ) -> Statement<'a> {
        if !self.effect_wrapper {
            return self.static_child_property_statement(span, element_id, name, value);
        }

        self.template_state.uses_effect = true;
        let getter = self.arrow_with_return(span, std::vec::Vec::new(), value);
        let value_id = "_v$";
        let statement = self.ast().statement_expression(
            span,
            self.ast().expression_assignment(
                span,
                AssignmentOperator::Assign,
                self.child_property_assignment_target(span, element_id, name),
                self.identifier_expression(span, value_id),
            ),
        );
        let setter = self.arrow_with_statements(span, vec![value_id], self.ast().vec1(statement));
        self.ast().statement_expression(
            span,
            self.call_identifier(span, "_$effect", vec![getter, setter]),
        )
    }

    pub(crate) fn dynamic_property_statement(
        &mut self,
        span: Span,
        element_id: &str,
        name: &str,
        value: Expression<'a>,
    ) -> Statement<'a> {
        if !self.effect_wrapper {
            return self.static_child_property_statement(span, element_id, name, value);
        }

        self.template_state.uses_effect = true;
        let getter = self.arrow_with_return(span, std::vec::Vec::new(), value);
        let value_id = "_v$";
        let statement = self.ast().statement_expression(
            span,
            self.ast().expression_assignment(
                span,
                AssignmentOperator::Assign,
                self.child_property_assignment_target(span, element_id, name),
                self.identifier_expression(span, value_id),
            ),
        );
        let setter = self.arrow_with_statements(span, vec![value_id], self.ast().vec1(statement));
        self.ast().statement_expression(
            span,
            self.call_identifier(span, "_$effect", vec![getter, setter]),
        )
    }

    fn child_property_assignment_target(
        &self,
        span: Span,
        element_id: &str,
        name: &str,
    ) -> AssignmentTarget<'a> {
        AssignmentTarget::StaticMemberExpression(self.ast().alloc_static_member_expression(
            span,
            self.identifier_expression(span, element_id),
            self.ast().identifier_name(span, self.ast().ident(name)),
            false,
        ))
    }
}
