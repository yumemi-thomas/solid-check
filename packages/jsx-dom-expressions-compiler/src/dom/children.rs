use napi::bindgen_prelude::*;
use oxc_allocator::CloneIn;
use oxc_ast::ast::Expression;
use oxc_ast::ast::{JSXChild, JSXElement, JSXExpression, Statement};
use oxc_ast_visit::VisitMut;

use crate::dom::attrs::CloseTagContext;
use crate::dom::element::{jsx_expression_to_expression, AstDomTransform};
use crate::dom::static_template::{last_static_element_child, lower_static_native_template};
use crate::shared::utils::{
    element_name, escape_html_text, escape_html_text_expression, is_component_name,
    static_jsx_expression, trim_jsx_text,
};

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn lower_dom_children(
        &mut self,
        element: &JSXElement<'a>,
        tag_name: &str,
        element_id: &str,
        template: &mut String,
        operations: &mut std::vec::Vec<Statement<'a>>,
    ) -> Result<()> {
        let child_to_be_closed = self.child_close_context(tag_name, CloseTagContext::root());
        let last_static_child = last_static_element_child(&element.children);
        let mut index = 0;
        let mut child_node_index = 0;

        while index < element.children.len() {
            let child = &element.children[index];
            match child {
                JSXChild::Text(text) => {
                    let text = trim_jsx_text(&text.value);
                    if !text.is_empty() {
                        template.push_str(&escape_html_text(&text));
                        child_node_index += 1;
                    }
                }
                JSXChild::Element(child) => {
                    if is_component_name(&child.opening_element.name) {
                        self.template_state.uses_insert = true;
                        let child = self.lower_element(child)?;
                        operations.push(self.insert_statement(
                            element.span,
                            element_id,
                            child,
                            None,
                        ));
                    } else if let Some(static_template) = lower_static_native_template(
                        self,
                        child,
                        CloseTagContext {
                            last_element: Some(index) == last_static_child
                                && !has_following_static_content(&element.children[index + 1..]),
                            to_be_closed: child_to_be_closed.clone(),
                        },
                    )? {
                        template.push_str(&static_template);
                        child_node_index += 1;
                    } else {
                        self.lower_dynamic_native_child(
                            child,
                            CloseTagContext {
                                last_element: Some(index) == last_static_child
                                    && !has_following_static_content(
                                        &element.children[index + 1..],
                                    ),
                                to_be_closed: child_to_be_closed.clone(),
                            },
                            element_id,
                            child_node_index,
                            template,
                            operations,
                        )?;
                        child_node_index += 1;
                    }
                }
                JSXChild::ExpressionContainer(container) => {
                    if matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                        index += 1;
                        continue;
                    }
                    if let Some(value) = self.static_jsx_expression_value(&container.expression) {
                        template.push_str(&escape_html_text_expression(&value));
                        child_node_index += 1;
                        index += 1;
                        continue;
                    }

                    let run_end = dynamic_run_end(&element.children, index);
                    let previous_is_text = has_previous_static_text(&element.children[..index]);
                    let next_is_text = has_next_static_text(&element.children[run_end..]);
                    let marker_name = if previous_is_text && next_is_text {
                        template.push_str("<!>");
                        let marker_name = self.next_element_id();
                        operations.push(self.variable_statement(
                            element.span,
                            &marker_name,
                            self.child_node_expression(element.span, element_id, child_node_index),
                        ));
                        child_node_index += 1;
                        Some(marker_name)
                    } else {
                        None
                    };

                    for dynamic_child in &element.children[index..run_end] {
                        let JSXChild::ExpressionContainer(container) = dynamic_child else {
                            return Err(Error::from_reason(
                                "Dynamic child run included a non-expression child",
                            ));
                        };
                        self.template_state.uses_insert = true;
                        let mut value =
                            jsx_expression_to_expression(&container.expression, self.allocator);
                        self.visit_expression(&mut value);
                        let value = self.dom_child_expression(container.span, value);
                        let marker = marker_name
                            .as_ref()
                            .map(|name| self.identifier_expression(element.span, name))
                            .or_else(|| {
                                has_following_static_content(&element.children[run_end..]).then(
                                    || self.child_node_expression(element.span, element_id, 0),
                                )
                            });
                        operations.push(self.insert_statement(
                            element.span,
                            element_id,
                            value,
                            marker,
                        ));
                    }
                    index = run_end;
                    continue;
                }
                JSXChild::Spread(spread) => {
                    self.template_state.uses_insert = true;
                    let value = spread_child_expression(self, spread.span, &spread.expression);
                    let marker = has_following_static_content(&element.children[index + 1..])
                        .then(|| self.child_node_expression(element.span, element_id, 0));
                    operations.push(self.insert_statement(element.span, element_id, value, marker));
                }
                _ => {
                    return Err(Error::from_reason(
                        "Fragments and spread children are not implemented in the AST-native milestone yet",
                    ));
                }
            }
            index += 1;
        }

        Ok(())
    }

    fn lower_dynamic_native_child(
        &mut self,
        child: &JSXElement<'a>,
        close_context: CloseTagContext,
        parent_id: &str,
        child_node_index: usize,
        parent_template: &mut String,
        operations: &mut std::vec::Vec<Statement<'a>>,
    ) -> Result<()> {
        let tag_name = element_name(&child.opening_element.name)?;
        let mut child_template = format!("<{tag_name}");
        let child_id = self.next_element_id();
        let mut child_operations = std::vec::Vec::new();

        self.lower_template_attributes(
            &child.opening_element.attributes,
            &tag_name,
            &child_id,
            !child.children.is_empty(),
            &mut child_template,
            &mut child_operations,
        )?;

        child_template.push('>');
        self.lower_dom_children(
            child,
            &tag_name,
            &child_id,
            &mut child_template,
            &mut child_operations,
        )?;
        if self.should_close_tag(&tag_name, close_context) {
            child_template.push_str(&format!("</{tag_name}>"));
        }

        parent_template.push_str(&child_template);
        let child_lookup =
            self.child_element_expression(child.span, parent_id, child_node_index, &tag_name);
        operations.push(self.variable_statement(child.span, &child_id, child_lookup));
        operations.extend(child_operations);
        Ok(())
    }

    fn child_element_expression(
        &mut self,
        span: oxc_span::Span,
        parent: &str,
        index: usize,
        tag_name: &str,
    ) -> Expression<'a> {
        if !self.hydratable || !self.dev {
            return self.child_node_expression(span, parent, index);
        }

        let tag = self
            .ast()
            .expression_string_literal(span, self.ast().atom(tag_name), None);

        if index == 0 {
            self.template_state.uses_get_first_child = true;
            return self.call_identifier(
                span,
                "_$getFirstChild",
                vec![self.identifier_expression(span, parent), tag],
            );
        }

        self.template_state.uses_get_next_sibling = true;
        let previous = self.child_node_expression(span, parent, index - 1);
        self.call_identifier(span, "_$getNextSibling", vec![previous, tag])
    }
}

fn spread_child_expression<'a>(
    ctx: &AstDomTransform<'a, '_>,
    span: oxc_span::Span,
    expression: &Expression<'a>,
) -> Expression<'a> {
    let expression = expression.clone_in(ctx.allocator);
    if matches!(
        expression,
        Expression::StaticMemberExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::ChainExpression(_)
    ) {
        ctx.arrow_return_expression(span, expression)
    } else {
        expression
    }
}

fn has_following_static_content(children: &[JSXChild<'_>]) -> bool {
    children.iter().any(|child| match child {
        JSXChild::Text(text) => !trim_jsx_text(&text.value).is_empty(),
        JSXChild::ExpressionContainer(container) => {
            !matches!(container.expression, JSXExpression::EmptyExpression(_))
                && static_jsx_expression(&container.expression, &[]).is_some()
        }
        JSXChild::Element(child) => !is_component_name(&child.opening_element.name),
        _ => false,
    })
}

fn has_previous_static_text(children: &[JSXChild<'_>]) -> bool {
    children.iter().rev().any(|child| match child {
        JSXChild::Text(text) => !trim_jsx_text(&text.value).is_empty(),
        JSXChild::ExpressionContainer(container) => {
            static_jsx_expression(&container.expression, &[]).is_some()
        }
        _ => false,
    })
}

fn has_next_static_text(children: &[JSXChild<'_>]) -> bool {
    children.iter().any(|child| match child {
        JSXChild::Text(text) => !trim_jsx_text(&text.value).is_empty(),
        JSXChild::ExpressionContainer(container) => {
            static_jsx_expression(&container.expression, &[]).is_some()
        }
        _ => false,
    })
}

fn dynamic_run_end(children: &[JSXChild<'_>], start: usize) -> usize {
    let mut index = start;
    while index < children.len() {
        let JSXChild::ExpressionContainer(container) = &children[index] else {
            break;
        };
        if matches!(container.expression, JSXExpression::EmptyExpression(_))
            || static_jsx_expression(&container.expression, &[]).is_some()
        {
            break;
        }
        index += 1;
    }
    index
}
