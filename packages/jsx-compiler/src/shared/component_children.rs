use napi::bindgen_prelude::*;
use oxc_allocator::CloneIn;
use oxc_ast::ast::{Expression, JSXChild, JSXExpression, Statement};
use oxc_span::GetSpan;

use crate::dom::element::AstDomTransform;
use crate::shared::array::expression_to_array_element;
use crate::shared::component::transform_component_expression;
use crate::shared::utils::{decode_html_entities, trim_jsx_text};

pub(crate) struct ComponentChildren<'a> {
    pub(crate) value: Expression<'a>,
    pub(crate) needs_getter: bool,
    pub(crate) setup: std::vec::Vec<Statement<'a>>,
}

struct ChildValue<'a> {
    value: Expression<'a>,
    memoize: bool,
}

pub(crate) fn component_children<'a>(
    ctx: &mut AstDomTransform<'a, '_>,
    children: &[JSXChild<'a>],
) -> Result<Option<ComponentChildren<'a>>> {
    let mut values = std::vec::Vec::new();
    let mut dynamic = false;
    let mut needs_getter = false;
    let mut child_setup = std::vec::Vec::new();
    for child in children {
        match child {
            JSXChild::Text(text) => {
                let span = text.span;
                let value = decode_html_entities(&trim_jsx_text(&text.value));
                if !value.is_empty() {
                    values.push(ChildValue {
                        value: ctx.ast().expression_string_literal(
                            span,
                            ctx.ast().atom(&value),
                            None,
                        ),
                        memoize: false,
                    });
                }
            }
            JSXChild::ExpressionContainer(container) => {
                if !matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                    dynamic = true;
                    needs_getter = needs_getter
                        || matches!(
                            container.expression,
                            JSXExpression::StaticMemberExpression(_)
                                | JSXExpression::ComputedMemberExpression(_)
                                | JSXExpression::ChainExpression(_)
                                | JSXExpression::JSXElement(_)
                                | JSXExpression::CallExpression(_)
                                | JSXExpression::ConditionalExpression(_)
                                | JSXExpression::LogicalExpression(_)
                        );
                    if let JSXExpression::JSXElement(element) = &container.expression {
                        let (value, setup) = ctx.lower_element_with_setup(element)?;
                        values.push(ChildValue {
                            value,
                            memoize: false,
                        });
                        child_setup.extend(setup);
                    } else {
                        let value = transform_component_expression(ctx, &container.expression);
                        let memoize = ctx.should_memoize_dynamic_expression(&value);
                        values.push(ChildValue { value, memoize });
                    }
                }
            }
            JSXChild::Element(element) => {
                dynamic = true;
                needs_getter = true;
                let (value, setup) = ctx.lower_element_with_setup(element)?;
                values.push(ChildValue {
                    value,
                    memoize: false,
                });
                child_setup.extend(setup);
            }
            JSXChild::Spread(spread) => {
                dynamic = true;
                needs_getter = needs_getter
                    || matches!(
                        spread.expression,
                        Expression::StaticMemberExpression(_)
                            | Expression::ComputedMemberExpression(_)
                            | Expression::ChainExpression(_)
                    );
                let value = spread.expression.clone_in(ctx.allocator);
                let memoize = ctx.should_memoize_dynamic_expression(&value);
                values.push(ChildValue { value, memoize });
            }
            _ => {
                return Err(Error::from_reason(
                    "Only text and expression component children are implemented in the AST-native milestone",
                ));
            }
        }
    }

    Ok(match values.len() {
        0 => None,
        1 => Some(ComponentChildren {
            value: values.pop().expect("component child exists").value,
            needs_getter,
            setup: child_setup,
        }),
        _ => Some(ComponentChildren {
            value: ctx.ast().expression_array(
                children
                    .first()
                    .map_or_else(|| oxc_span::Span::new(0, 0), JSXChild::span),
                ctx.ast().vec_from_iter(values.into_iter().map(|child| {
                    let value = if child.memoize {
                        ctx.memoized_dynamic_expression(
                            children
                                .first()
                                .map_or_else(|| oxc_span::Span::new(0, 0), JSXChild::span),
                            child.value,
                        )
                    } else {
                        child.value
                    };
                    expression_to_array_element(value)
                })),
            ),
            needs_getter: dynamic,
            setup: child_setup,
        }),
    })
}
