use napi::bindgen_prelude::*;
use oxc_ast::ast::{Expression, JSXElementName, JSXMemberExpression, JSXMemberExpressionObject};
use oxc_span::Span;

use crate::shared::utils::is_identifier_key;
use crate::universal::transform::AstUniversalTransform;

impl<'a> AstUniversalTransform<'a, '_> {
    pub(super) fn component_callee_expression(
        &mut self,
        name: &JSXElementName<'a>,
    ) -> Result<Expression<'a>> {
        match name {
            JSXElementName::Identifier(identifier) => {
                Ok(self.component_identifier_expression(&identifier.name))
            }
            JSXElementName::IdentifierReference(identifier) => {
                Ok(self.component_identifier_expression(&identifier.name))
            }
            JSXElementName::MemberExpression(member) => self.component_member_expression(member),
            JSXElementName::ThisExpression(_) => Err(Error::from_reason(
                "Universal this-component callees are not implemented in the AST-native milestone yet",
            )),
            JSXElementName::NamespacedName(_) => Err(Error::from_reason(
                "Universal namespaced component callees are not implemented in the AST-native milestone yet",
            )),
        }
    }

    fn component_member_expression(
        &mut self,
        member: &JSXMemberExpression<'a>,
    ) -> Result<Expression<'a>> {
        let object = match &member.object {
            JSXMemberExpressionObject::IdentifierReference(identifier) => {
                self.component_identifier_expression(&identifier.name)
            }
            JSXMemberExpressionObject::MemberExpression(member) => {
                self.component_member_expression(member)?
            }
            JSXMemberExpressionObject::ThisExpression(_) => {
                return Err(Error::from_reason(
                    "Universal this-component member callees are not implemented in the AST-native milestone yet",
                ));
            }
        };
        Ok(if is_identifier_key(&member.property.name) {
            Expression::StaticMemberExpression(
                self.ast().alloc_static_member_expression(
                    member.span,
                    object,
                    self.ast()
                        .identifier_name(member.span, self.ast().ident(&member.property.name)),
                    false,
                ),
            )
        } else {
            Expression::ComputedMemberExpression(self.ast().alloc_computed_member_expression(
                member.span,
                object,
                self.ast().expression_string_literal(
                    member.span,
                    self.ast().atom(&member.property.name),
                    None,
                ),
                false,
            ))
        })
    }

    fn component_identifier_expression(&mut self, component: &str) -> Expression<'a> {
        if self.built_ins.iter().any(|built_in| built_in == component) {
            if !self
                .built_in_imports
                .iter()
                .any(|built_in| built_in == component)
            {
                self.built_in_imports.push(component.to_string());
            }
            self.ast()
                .expression_identifier(Span::new(0, 0), self.ast().ident(&format!("_{component}")))
        } else {
            self.ast()
                .expression_identifier(Span::new(0, 0), self.ast().ident(component))
        }
    }
}
