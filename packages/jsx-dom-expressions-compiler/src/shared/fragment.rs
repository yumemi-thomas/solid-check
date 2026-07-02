use napi::bindgen_prelude::*;
use oxc_ast::ast::{Expression, JSXChild, JSXExpression, JSXFragment};
use oxc_span::{GetSpan, Span};

use crate::dom::element::AstDomTransform;
use crate::shared::array::expression_to_array_element;
use crate::shared::component::transform_component_expression;
use crate::shared::utils::{decode_html_entities, trim_jsx_text};

pub(crate) fn lower_fragment<'a>(
    ctx: &mut AstDomTransform<'a, '_>,
    fragment: &JSXFragment<'a>,
) -> Result<Expression<'a>> {
    let mut values = std::vec::Vec::new();
    for child in &fragment.children {
        match child {
            JSXChild::Text(text) => {
                let value = decode_html_entities(&trim_jsx_text(&text.value));
                if !value.is_empty() {
                    values.push(ctx.ast().expression_string_literal(
                        text.span,
                        ctx.ast().atom(&value),
                        None,
                    ));
                }
            }
            JSXChild::ExpressionContainer(container) => {
                if !matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                    let value = transform_component_expression(ctx, &container.expression);
                    if ctx.should_memoize_dynamic_expression(&value) {
                        values.push(ctx.memoized_dynamic_expression(container.span, value));
                    } else {
                        values.push(value);
                    }
                }
            }
            JSXChild::Element(element) => {
                values.push(ctx.lower_element(element)?);
            }
            JSXChild::Fragment(fragment) => {
                values.push(lower_fragment(ctx, fragment)?);
            }
            JSXChild::Spread(_) => {
                return Err(Error::from_reason(
                    "Fragment spread children are not implemented in the AST-native milestone yet",
                ));
            }
        }
    }

    Ok(match values.len() {
        0 => ctx.ast().expression_array(fragment.span, ctx.ast().vec()),
        1 => values.pop().expect("fragment value exists"),
        _ => ctx.ast().expression_array(
            fragment
                .children
                .first()
                .map_or_else(|| Span::new(0, 0), JSXChild::span),
            ctx.ast()
                .vec_from_iter(values.into_iter().map(expression_to_array_element)),
        ),
    })
}
