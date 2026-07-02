use napi::bindgen_prelude::*;
use oxc_ast::ast::BinaryOperator;
use oxc_ast::ast::{Expression, JSXElementName, JSXExpression};
use oxc_span::Span;

use crate::shared::constants::void_elements;

#[derive(Clone)]
pub(crate) enum StaticValue {
    String(String),
    Number(f64),
}

impl StaticValue {
    pub(crate) fn into_template_value(self) -> String {
        match self {
            StaticValue::String(value) => value,
            StaticValue::Number(value) => format_number(value),
        }
    }
}

pub(crate) fn element_name(name: &JSXElementName<'_>) -> Result<String> {
    match name {
        JSXElementName::Identifier(identifier) => Ok(identifier.name.to_string()),
        JSXElementName::IdentifierReference(identifier) => Ok(identifier.name.to_string()),
        JSXElementName::NamespacedName(name) => {
            Ok(format!("{}:{}", name.namespace.name, name.name.name))
        }
        _ => Err(Error::from_reason(
            "Only simple JSX element names are implemented in the AST-native milestone",
        )),
    }
}

pub(crate) fn is_component_name(name: &JSXElementName<'_>) -> bool {
    matches!(
        name,
        JSXElementName::MemberExpression(_) | JSXElementName::ThisExpression(_)
    ) || matches!(
        name,
        JSXElementName::IdentifierReference(identifier)
            if identifier
                .name
                .chars()
                .next()
                .is_some_and(|first| first.is_ascii_uppercase() || first == '_' || first == '$')
    )
}

pub(crate) fn static_jsx_expression_value(expression: &JSXExpression<'_>) -> Option<String> {
    static_jsx_expression(expression, &[]).map(StaticValue::into_template_value)
}

pub(crate) fn static_jsx_expression(
    expression: &JSXExpression<'_>,
    bindings: &[(String, StaticValue)],
) -> Option<StaticValue> {
    match expression {
        JSXExpression::StringLiteral(value) => Some(StaticValue::String(value.value.to_string())),
        JSXExpression::NumericLiteral(value) => Some(StaticValue::Number(value.value)),
        JSXExpression::BooleanLiteral(value) => Some(StaticValue::String(value.value.to_string())),
        JSXExpression::NullLiteral(_) => Some(StaticValue::String("null".to_string())),
        JSXExpression::Identifier(identifier) => bindings
            .iter()
            .find(|(name, _)| name == identifier.name.as_str())
            .map(|(_, value)| value.clone()),
        JSXExpression::BinaryExpression(binary) => {
            static_binary_expression(&binary.left, binary.operator, &binary.right, bindings)
        }
        _ => None,
    }
}

pub(crate) fn static_expression(
    expression: &Expression<'_>,
    bindings: &[(String, StaticValue)],
) -> Option<StaticValue> {
    match expression {
        Expression::StringLiteral(value) => Some(StaticValue::String(value.value.to_string())),
        Expression::NumericLiteral(value) => Some(StaticValue::Number(value.value)),
        Expression::Identifier(identifier) => bindings
            .iter()
            .find(|(name, _)| name == identifier.name.as_str())
            .map(|(_, value)| value.clone()),
        Expression::BinaryExpression(binary) => {
            static_binary_expression(&binary.left, binary.operator, &binary.right, bindings)
        }
        _ => None,
    }
}

fn static_binary_expression(
    left: &Expression<'_>,
    operator: BinaryOperator,
    right: &Expression<'_>,
    bindings: &[(String, StaticValue)],
) -> Option<StaticValue> {
    let left = static_expression(left, bindings)?;
    let right = static_expression(right, bindings)?;
    match operator {
        BinaryOperator::Addition => match (left, right) {
            (StaticValue::Number(left), StaticValue::Number(right)) => {
                Some(StaticValue::Number(left + right))
            }
            (left, right) => Some(StaticValue::String(format!(
                "{}{}",
                left.into_template_value(),
                right.into_template_value()
            ))),
        },
        _ => None,
    }
}

pub(crate) fn source_from_span(span: Span, source: &str) -> &str {
    &source[span.start as usize..span.end as usize]
}

pub(crate) fn trim_jsx_text(value: &str) -> String {
    let collapsed = value
        .split_whitespace()
        .collect::<std::vec::Vec<_>>()
        .join(" ");
    if collapsed.is_empty() && !value.contains('\n') && value.chars().any(char::is_whitespace) {
        return " ".to_string();
    }
    if collapsed.is_empty() || value.contains('\n') {
        return collapsed;
    }

    let leading = value.chars().next().is_some_and(char::is_whitespace);
    let trailing = value.chars().last().is_some_and(char::is_whitespace);
    format!(
        "{}{}{}",
        if leading { " " } else { "" },
        collapsed,
        if trailing { " " } else { "" }
    )
}

pub(crate) fn escape_html_text(value: &str) -> String {
    value.replace('<', "&lt;")
}

pub(crate) fn escape_html_text_expression(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;")
}

pub(crate) fn decode_html_entities(value: &str) -> String {
    value
        .replace("&nbsp;", "\u{a0}")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&hellip;", "…")
        .replace("&amp;", "&")
}

pub(crate) fn format_attribute_value_with_quotes(value: &str, omit_quotes: bool) -> String {
    if omit_quotes && can_omit_attribute_quotes(value) {
        value.to_string()
    } else {
        format!("{value:?}")
    }
}

fn can_omit_attribute_quotes(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|char| {
            !matches!(
                char,
                ' ' | '\t' | '\n' | '\r' | '"' | '\'' | '`' | '=' | '<' | '>'
            )
        })
}

pub(crate) fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}

pub(crate) fn is_void_element(tag_name: &str) -> bool {
    void_elements(tag_name)
}

pub(crate) fn is_identifier_key(name: &str) -> bool {
    name.chars()
        .all(|char| char == '_' || char == '$' || char.is_ascii_alphanumeric())
        && name
            .chars()
            .next()
            .is_some_and(|char| char == '_' || char == '$' || char.is_ascii_alphabetic())
}

/// Builds a 1-based generated local name such as `_el$`, `_el$2`, `_ref$`,
/// `_c$`, or `_self$`. The first occurrence omits the numeric suffix to match
/// the Babel plugin's naming.
pub(crate) fn indexed_local(prefix: &str, index: usize) -> String {
    if index == 1 {
        format!("{prefix}$")
    } else {
        format!("{prefix}${index}")
    }
}

pub(crate) fn template_id(index: usize) -> String {
    if index == 0 {
        "_tmpl$".to_string()
    } else {
        format!("_tmpl${}", index + 1)
    }
}
