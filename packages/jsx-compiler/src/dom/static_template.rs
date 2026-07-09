use napi::bindgen_prelude::*;
use oxc_ast::ast::{JSXChild, JSXElement, JSXExpression};

use crate::dom::attrs::{try_append_static_template_attributes, CloseTagContext};
use crate::dom::element::AstDomTransform;
use crate::shared::utils::{
    element_name, escape_html_text, escape_html_text_expression, is_component_name,
    static_jsx_expression_value, trim_jsx_text,
};

pub(crate) fn lower_static_native_template(
    ctx: &AstDomTransform<'_, '_>,
    element: &JSXElement<'_>,
    close_context: CloseTagContext,
) -> Result<Option<String>> {
    if is_component_name(&element.opening_element.name) {
        return Ok(None);
    }

    let tag_name = element_name(&element.opening_element.name)?;
    let mut template = format!("<{tag_name}");

    if !try_append_static_template_attributes(
        Some(ctx),
        &element.opening_element.attributes,
        &mut template,
    )? {
        return Ok(None);
    }

    template.push('>');

    if tag_name == "noscript" {
        if ctx.should_close_tag(&tag_name, close_context) {
            template.push_str(&format!("</{tag_name}>"));
        }
        return Ok(Some(template));
    }

    let child_to_be_closed = ctx.child_close_context(&tag_name, close_context.clone());
    let last_element = ctx.find_last_element(&element.children);
    for (index, child) in element.children.iter().enumerate() {
        match child {
            JSXChild::Text(text) => {
                let text = trim_jsx_text(&text.value);
                if !text.is_empty() {
                    template.push_str(&escape_html_text(&text));
                }
            }
            JSXChild::ExpressionContainer(container) => {
                if matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                    continue;
                }
                let value = ctx
                    .static_jsx_expression_value(&container.expression)
                    .or_else(|| static_jsx_expression_value(&container.expression));
                let Some(value) = value else {
                    return Ok(None);
                };
                template.push_str(&escape_html_text_expression(&value));
            }
            JSXChild::Element(child) => {
                let Some(child_template) = lower_static_native_template(
                    ctx,
                    child,
                    CloseTagContext {
                        last_element: Some(index) == last_element,
                        to_be_closed: child_to_be_closed.clone(),
                    },
                )?
                else {
                    return Ok(None);
                };
                template.push_str(&child_template);
            }
            _ => return Ok(None),
        }
    }

    if ctx.should_close_tag(&tag_name, close_context) {
        template.push_str(&format!("</{tag_name}>"));
    }

    Ok(Some(template))
}