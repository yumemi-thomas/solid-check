use napi::bindgen_prelude::*;
use oxc_allocator::CloneIn;
use oxc_ast::ast::Expression;
use oxc_ast::ast::{JSXChild, JSXElement, JSXExpression, Statement};
use oxc_ast_visit::VisitMut;

use crate::dom::attrs::CloseTagContext;
use crate::dom::element::{jsx_expression_to_expression, AstDomTransform};
use crate::dom::static_template::lower_static_native_template;
use crate::dom::template::InsertMarker;
use crate::shared::utils::{
    child_slot_allocates_ids, element_name, escape_html_text, escape_html_text_expression,
    is_component_name, is_dynamic_child_slot, static_jsx_expression, trim_jsx_text,
};

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn lower_dom_children(
        &mut self,
        element: &JSXElement<'a>,
        tag_name: &str,
        element_id: &str,
        template: &mut String,
        declarations: &mut std::vec::Vec<Statement<'a>>,
        operations: &mut std::vec::Vec<Statement<'a>>,
    ) -> Result<()> {
        // Walk anchors are per-parent: this element's positional walks start
        // from its own reference, not an outer marker.
        let saved_anchor = self.hydration_walk_anchor.take();
        let result = self.lower_dom_children_inner(
            element,
            tag_name,
            element_id,
            template,
            declarations,
            operations,
        );
        self.hydration_walk_anchor = saved_anchor;
        result
    }

    fn lower_dom_children_inner(
        &mut self,
        element: &JSXElement<'a>,
        tag_name: &str,
        element_id: &str,
        template: &mut String,
        declarations: &mut std::vec::Vec<Statement<'a>>,
        operations: &mut std::vec::Vec<Statement<'a>>,
    ) -> Result<()> {
        let child_to_be_closed = self.child_close_context(tag_name, CloseTagContext::root());
        let last_element = self.find_last_element(&element.children);
        // Mirror of the Babel generate: when a parent hosts multiple dynamic
        // slots, each slot needs its own truthy insertion marker — the marker
        // doubles as the runtime's $$SLOT ownership tag, and shared or null
        // markers let one slot's cleanup destroy a node that migrated to its
        // neighbor (solidjs/solid#2830). Slots ride an immediately following
        // static sibling when one exists, otherwise get a dedicated `<!>`.
        let per_slot = !self.hydratable && self.dynamic_slot_count(&element.children) > 1;
        // Hydratable `<html>` children resolve by tag with `getNextMatch`
        // (chained from the previous match) — browsers normalize the document
        // shell, so positional walks are unreliable there.
        let html_tag_walk = self.hydratable && tag_name == "html";
        let mut previous_html_child: Option<String> = None;
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
                        let lowered = self.lower_element(child)?;
                        let marker = self.dynamic_slot_marker(
                            &element.children,
                            index,
                            index + 1,
                            per_slot,
                            element.span,
                            element_id,
                            &mut child_node_index,
                            template,
                            declarations,
                        );
                        operations.push(self.insert_statement(
                            element.span,
                            element_id,
                            lowered,
                            marker,
                        ));
                    } else if let Some(static_template) = lower_static_native_template(
                        self,
                        child,
                        CloseTagContext {
                            last_element: Some(index) == last_element,
                            to_be_closed: child_to_be_closed.clone(),
                        },
                    )? {
                        template.push_str(&static_template);
                        child_node_index += 1;
                    } else {
                        let lookup_override = if html_tag_walk {
                            Some(self.next_match_expression(
                                child,
                                element_id,
                                previous_html_child.as_deref(),
                            )?)
                        } else {
                            None
                        };
                        let child_id = self.lower_dynamic_native_child(
                            child,
                            CloseTagContext {
                                last_element: Some(index) == last_element,
                                to_be_closed: child_to_be_closed.clone(),
                            },
                            element_id,
                            child_node_index,
                            lookup_override,
                            template,
                            declarations,
                            operations,
                        )?;
                        previous_html_child = Some(child_id);
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
                    // Single-slot parents keep the previous marker strategy
                    // untouched: one shared placeholder when boxed by text,
                    // the leading static content otherwise. Hydratable slots
                    // always use `<!$><!/>` marker pairs instead.
                    let shared_marker_name = if !self.hydratable
                        && !per_slot
                        && has_previous_static_text(&element.children[..index])
                        && has_next_static_text(&element.children[run_end..])
                    {
                        template.push_str("<!>");
                        let marker_name = self.next_element_id();
                        let lookup =
                            self.child_walk_expression(element.span, element_id, child_node_index);
                        declarations.push(self.variable_statement(
                            element.span,
                            &marker_name,
                            lookup,
                        ));
                        child_node_index += 1;
                        Some(marker_name)
                    } else {
                        None
                    };

                    for (run_offset, dynamic_child) in
                        element.children[index..run_end].iter().enumerate()
                    {
                        let run_index = index + run_offset;
                        let JSXChild::ExpressionContainer(container) = dynamic_child else {
                            return Err(Error::from_reason(
                                "Dynamic child run included a non-expression child",
                            ));
                        };
                        self.template_state.uses_insert = true;
                        let mut value =
                            jsx_expression_to_expression(&container.expression, self.allocator);
                        self.visit_expression(&mut value);
                        // A `/*@static*/` marker opts the hole out of deferral:
                        // the value inserts once, unwrapped and unscoped.
                        let marked_static = self.has_static_marker(container.span);
                        let value = if marked_static {
                            value
                        } else {
                            self.dom_child_expression(container.span, value)
                        };
                        // Mirror of the ssr generate's `scope()` wrap: deferred
                        // holes that can allocate hydration ids get their own
                        // owner scope. Both flags come from shared predicates
                        // so the generates can't desync.
                        let value = if !marked_static
                            && self.hydratable
                            && child_slot_allocates_ids(dynamic_child)
                            && is_dynamic_child_slot(dynamic_child)
                        {
                            self.scope_child_expression(container.span, value)
                        } else {
                            value
                        };
                        let marker = if let Some(name) = shared_marker_name.as_ref() {
                            Some(InsertMarker {
                                marker: self.identifier_expression(element.span, name),
                                initial: None,
                            })
                        } else {
                            self.dynamic_slot_marker(
                                &element.children,
                                run_index,
                                run_end,
                                per_slot,
                                element.span,
                                element_id,
                                &mut child_node_index,
                                template,
                                declarations,
                            )
                        };
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
                    // Spread children always allocate ids; scope keyed off the
                    // same shared dynamic predicate as the ssr generate.
                    let value = if self.hydratable && is_dynamic_child_slot(child) {
                        self.scope_child_expression(spread.span, value)
                    } else {
                        value
                    };
                    let marker = self.dynamic_slot_marker(
                        &element.children,
                        index,
                        index + 1,
                        per_slot,
                        element.span,
                        element_id,
                        &mut child_node_index,
                        template,
                        declarations,
                    );
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

    /// Mirror of the Babel generate's `findLastElement`: the last retained
    /// child that ends the parent's template content. In hydratable mode any
    /// retained child counts (dynamic slots append `<!$><!/>` markers, so an
    /// element followed by one is not last); otherwise only template-inlined
    /// children (text, static expressions, native elements) count.
    pub(crate) fn find_last_element(&self, children: &[JSXChild<'a>]) -> Option<usize> {
        for (index, child) in children.iter().enumerate().rev() {
            let retained = match child {
                JSXChild::Text(text) => !trim_jsx_text(&text.value).is_empty(),
                JSXChild::ExpressionContainer(container) => {
                    !matches!(container.expression, JSXExpression::EmptyExpression(_))
                }
                _ => true,
            };
            if !retained {
                continue;
            }
            let qualifies = self.hydratable
                || match child {
                    JSXChild::Text(_) => true,
                    JSXChild::ExpressionContainer(container) => self
                        .static_jsx_expression_value(&container.expression)
                        .is_some(),
                    JSXChild::Element(child) => !is_component_name(&child.opening_element.name),
                    _ => false,
                };
            if qualifies {
                return Some(index);
            }
        }
        None
    }

    /// Number of children that compile to `insert()` calls: dynamic expression
    /// containers, components, and spread children. Static expressions and
    /// text inline into the template; native elements walk.
    fn dynamic_slot_count(&self, children: &[JSXChild<'a>]) -> usize {
        children
            .iter()
            .filter(|child| match child {
                JSXChild::ExpressionContainer(container) => {
                    !matches!(container.expression, JSXExpression::EmptyExpression(_))
                        && self
                            .static_jsx_expression_value(&container.expression)
                            .is_none()
                }
                JSXChild::Element(child) => is_component_name(&child.opening_element.name),
                JSXChild::Spread(_) => true,
                _ => false,
            })
            .count()
    }

    /// Marker for one slot of a multi-slot parent (solidjs/solid#2830): the
    /// immediately following template node when one exists — unless the slot
    /// is boxed by text, where a placeholder comment is structurally required
    /// to keep the surrounding template text nodes from merging — otherwise a
    /// dedicated `<!>` placeholder appended at the slot's template position.
    #[allow(clippy::too_many_arguments)]
    fn dynamic_slot_marker(
        &mut self,
        children: &[JSXChild<'a>],
        index: usize,
        following_start: usize,
        per_slot: bool,
        span: oxc_span::Span,
        element_id: &str,
        child_node_index: &mut usize,
        template: &mut String,
        declarations: &mut std::vec::Vec<Statement<'a>>,
    ) -> Option<InsertMarker<'a>> {
        if self.hydratable {
            return self.hydration_slot_marker(
                children,
                following_start,
                span,
                element_id,
                child_node_index,
                template,
                declarations,
            );
        }
        if per_slot {
            return Some(InsertMarker {
                marker: self.per_slot_marker(
                    children,
                    index,
                    span,
                    element_id,
                    child_node_index,
                    template,
                    declarations,
                ),
                initial: None,
            });
        }
        if has_following_static_content(&children[following_start..]) {
            return Some(InsertMarker {
                marker: self.child_walk_expression(span, element_id, *child_node_index),
                initial: None,
            });
        }
        if *child_node_index > 0 {
            return Some(InsertMarker {
                marker: self.ast().expression_null_literal(span),
                initial: None,
            });
        }
        None
    }

    #[allow(clippy::too_many_arguments)]
    fn hydration_slot_marker(
        &mut self,
        children: &[JSXChild<'a>],
        following_start: usize,
        span: oxc_span::Span,
        element_id: &str,
        child_node_index: &mut usize,
        template: &mut String,
        declarations: &mut std::vec::Vec<Statement<'a>>,
    ) -> Option<InsertMarker<'a>> {
        if self.dynamic_slot_count(children) == 1
            && *child_node_index == 0
            && !has_following_static_content(&children[following_start..])
        {
            return None;
        }

        template.push_str("<!$><!/>");
        let start_marker = self.child_walk_expression(span, element_id, *child_node_index);
        let marker_name = self.next_element_id();
        let current_name = self.next_element_id();
        self.template_state.uses_get_next_marker = true;
        let marker_lookup =
            self.static_member_expression_from_expression(span, start_marker, "nextSibling");
        let init = self.call_identifier(span, "_$getNextMarker", vec![marker_lookup]);
        declarations.push(self.array_destructure_statement(
            span,
            &[&marker_name, &current_name],
            init,
        ));
        // At hydration time the SSR'd DOM holds arbitrary content between the
        // `<!$>`/`<!/>` pair; later positional walks in this parent must chain
        // from the end marker node `getNextMarker` located, not from the root.
        self.hydration_walk_anchor = Some((marker_name.clone(), *child_node_index + 1));
        *child_node_index += 2;
        Some(InsertMarker {
            marker: self.identifier_expression(span, &marker_name),
            initial: Some(self.identifier_expression(span, &current_name)),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn per_slot_marker(
        &mut self,
        children: &[JSXChild<'a>],
        index: usize,
        span: oxc_span::Span,
        element_id: &str,
        child_node_index: &mut usize,
        template: &mut String,
        declarations: &mut std::vec::Vec<Statement<'a>>,
    ) -> Expression<'a> {
        if !self.slot_boxed_by_text(children, index)
            && self.next_child_is_template_node(children, index)
        {
            return self.child_walk_expression(span, element_id, *child_node_index);
        }
        template.push_str("<!>");
        let marker_name = self.next_element_id();
        let lookup = self.child_walk_expression(span, element_id, *child_node_index);
        declarations.push(self.variable_statement(span, &marker_name, lookup));
        *child_node_index += 1;
        self.identifier_expression(span, &marker_name)
    }

    /// Nearest template-contributing sibling on both sides is text (mirrors
    /// the Babel generate's `wrappedByText`): dynamic slots and components are
    /// transparent to the walk; a native element or slot-free boundary stops it.
    fn slot_boxed_by_text(&self, children: &[JSXChild<'a>], index: usize) -> bool {
        self.nearest_template_sibling_is_text(children[..index].iter().rev())
            && self.nearest_template_sibling_is_text(children[index + 1..].iter())
    }

    fn nearest_template_sibling_is_text<'b>(
        &self,
        children: impl Iterator<Item = &'b JSXChild<'a>>,
    ) -> bool
    where
        'a: 'b,
    {
        for child in children {
            match child {
                JSXChild::Text(text) => {
                    if !trim_jsx_text(&text.value).is_empty() {
                        return true;
                    }
                }
                JSXChild::ExpressionContainer(container) => {
                    if matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                        continue;
                    }
                    if self
                        .static_jsx_expression_value(&container.expression)
                        .is_some()
                    {
                        return true;
                    }
                }
                JSXChild::Element(child) if !is_component_name(&child.opening_element.name) => {
                    return false;
                }
                JSXChild::Element(_) => {}
                _ => {}
            }
        }
        false
    }

    /// Whether the immediately following retained child contributes a template
    /// node (non-empty text, static expression, or native element) that can
    /// serve as this slot's marker.
    fn next_child_is_template_node(&self, children: &[JSXChild<'a>], index: usize) -> bool {
        for child in &children[index + 1..] {
            return match child {
                JSXChild::Text(text) => {
                    if trim_jsx_text(&text.value).is_empty() {
                        continue;
                    }
                    true
                }
                JSXChild::ExpressionContainer(container) => {
                    if matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                        continue;
                    }
                    self.static_jsx_expression_value(&container.expression)
                        .is_some()
                }
                JSXChild::Element(child) => !is_component_name(&child.opening_element.name),
                _ => false,
            };
        }
        false
    }

    /// `getNextMatch(<previous>.nextSibling | <parent>.firstChild, "<tag>")`
    /// lookup for a direct child of a hydratable `<html>` element.
    fn next_match_expression(
        &mut self,
        child: &JSXElement<'a>,
        parent_id: &str,
        previous_child: Option<&str>,
    ) -> Result<Expression<'a>> {
        let tag_name = element_name(&child.opening_element.name)?;
        self.template_state.uses_get_next_match = true;
        let base = match previous_child {
            Some(previous) => {
                self.static_member_expression(child.span, previous, "nextSibling")
            }
            None => self.static_member_expression(child.span, parent_id, "firstChild"),
        };
        let tag = self
            .ast()
            .expression_string_literal(child.span, self.ast().atom(&tag_name), None);
        Ok(self.call_identifier(child.span, "_$getNextMatch", vec![base, tag]))
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_dynamic_native_child(
        &mut self,
        child: &JSXElement<'a>,
        close_context: CloseTagContext,
        parent_id: &str,
        child_node_index: usize,
        lookup_override: Option<Expression<'a>>,
        parent_template: &mut String,
        declarations: &mut std::vec::Vec<Statement<'a>>,
        operations: &mut std::vec::Vec<Statement<'a>>,
    ) -> Result<String> {
        let tag_name = element_name(&child.opening_element.name)?;
        let mut child_template = format!("<{tag_name}");
        let child_id = self.next_element_id();
        let mut child_declarations = std::vec::Vec::new();
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
            &mut child_declarations,
            &mut child_operations,
        )?;
        if self.should_close_tag(&tag_name, close_context) {
            child_template.push_str(&format!("</{tag_name}>"));
        }

        parent_template.push_str(&child_template);
        let child_lookup = match lookup_override {
            Some(lookup) => lookup,
            None => {
                self.child_element_expression(child.span, parent_id, child_node_index, &tag_name)
            }
        };
        declarations.push(self.variable_statement(child.span, &child_id, child_lookup));
        declarations.extend(child_declarations);
        operations.extend(child_operations);
        Ok(child_id)
    }

    /// Wraps an insert accessor in `_$scope(...)`. The child lowering
    /// simplifies `{sig()}` to the bare getter `sig`; rewrap it as
    /// `() => sig()` so tagging the scope doesn't mutate the user's function.
    fn scope_child_expression(
        &mut self,
        span: oxc_span::Span,
        value: Expression<'a>,
    ) -> Expression<'a> {
        self.template_state.uses_scope = true;
        let already_function = match &value {
            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => true,
            Expression::CallExpression(call) => matches!(
                call.callee,
                Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
            ),
            _ => false,
        };
        let value = if already_function {
            value
        } else {
            let call =
                self.ast()
                    .expression_call(span, value, oxc_ast::NONE, self.ast().vec(), false);
            self.arrow_return_expression(span, call)
        };
        self.call_identifier(span, "_$scope", vec![value])
    }

    fn child_element_expression(
        &mut self,
        span: oxc_span::Span,
        parent: &str,
        index: usize,
        tag_name: &str,
    ) -> Expression<'a> {
        if !self.hydratable || !self.dev {
            return self.child_walk_expression(span, parent, index);
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
        let previous = self.child_walk_expression(span, parent, index - 1);
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
