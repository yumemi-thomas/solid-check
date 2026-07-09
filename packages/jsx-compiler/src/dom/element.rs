use napi::bindgen_prelude::*;
use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::ast::{
    AssignmentOperator, AssignmentTarget, Expression, JSXElement, JSXExpression, Statement,
};

use crate::dom::attrs::CloseTagContext;
use crate::dom::template::DomTemplateState;
use crate::shared::component::lower_component_with_setup;
use crate::shared::utils::{element_name, is_component_name, static_jsx_expression, StaticValue};

pub(crate) struct AstDomTransform<'a, 'source> {
    pub(crate) allocator: &'a Allocator,
    pub(crate) source: &'source str,
    pub(crate) module_name: &'source str,
    pub(crate) hydratable: bool,
    pub(crate) dev: bool,
    pub(crate) context_to_custom_elements: bool,
    pub(crate) delegate_events: bool,
    pub(crate) delegated_events: std::vec::Vec<String>,
    pub(crate) omit_quotes: bool,
    pub(crate) omit_attribute_spacing: bool,
    pub(crate) inline_styles: bool,
    pub(crate) effect_wrapper: bool,
    pub(crate) wrap_conditionals: bool,
    pub(crate) memo_wrapper: bool,
    pub(crate) static_marker: String,
    pub(crate) omit_nested_closing_tags: bool,
    pub(crate) omit_last_closing_tag: bool,
    pub(crate) built_ins: std::vec::Vec<String>,
    pub(crate) template_state: DomTemplateState,
    pub(crate) error: Option<String>,
    pub(crate) static_bindings: std::vec::Vec<(String, StaticValue)>,
    pub(crate) const_bindings: std::vec::Vec<String>,
    pub(crate) function_bindings: std::vec::Vec<String>,
    pub(crate) pending_this_capture: Option<String>,
    pub(crate) current_this_capture: Option<String>,
    pub(crate) statement_depth: usize,
    pub(crate) skip_xmlns_attribute: bool,
    /// After a hydration `getNextMarker` destructure, positional child walks
    /// in the same parent chain from the marker's end node — the SSR'd DOM
    /// holds arbitrary content between `<!$>` and `<!/>`, so root-relative
    /// `firstChild.nextSibling…` paths would land inside the marker region.
    /// `(end node identifier, template index of the end node)`.
    pub(crate) hydration_walk_anchor: Option<(String, usize)>,
    /// Whether the current template root saw a delegated event handler or a
    /// spread (which may carry one); consumed at the root to emit a single
    /// `runHydrationEvents()` after setup.
    pub(crate) has_hydratable_event: bool,
    pub(crate) element_index: usize,
    pub(crate) this_index: usize,
    pub(crate) ref_index: usize,
    pub(crate) condition_index: usize,
}

pub(crate) struct DomTransformConfig {
    pub(crate) hydratable: bool,
    pub(crate) dev: bool,
    pub(crate) context_to_custom_elements: bool,
    pub(crate) delegate_events: bool,
    pub(crate) delegated_events: std::vec::Vec<String>,
    pub(crate) omit_quotes: bool,
    pub(crate) omit_attribute_spacing: bool,
    pub(crate) inline_styles: bool,
    pub(crate) effect_wrapper: bool,
    pub(crate) wrap_conditionals: bool,
    pub(crate) memo_wrapper: bool,
    pub(crate) static_marker: String,
    pub(crate) omit_nested_closing_tags: bool,
    pub(crate) omit_last_closing_tag: bool,
    pub(crate) built_ins: std::vec::Vec<String>,
}

impl<'a, 'source> AstDomTransform<'a, 'source> {
    pub(crate) fn new(
        allocator: &'a Allocator,
        source: &'source str,
        module_name: &'source str,
        config: DomTransformConfig,
    ) -> Self {
        Self {
            allocator,
            source,
            module_name,
            hydratable: config.hydratable,
            dev: config.dev,
            context_to_custom_elements: config.context_to_custom_elements,
            delegate_events: config.delegate_events,
            delegated_events: config.delegated_events,
            omit_quotes: config.omit_quotes,
            omit_attribute_spacing: config.omit_attribute_spacing,
            inline_styles: config.inline_styles,
            effect_wrapper: config.effect_wrapper,
            wrap_conditionals: config.wrap_conditionals,
            memo_wrapper: config.memo_wrapper,
            static_marker: config.static_marker,
            omit_nested_closing_tags: config.omit_nested_closing_tags,
            omit_last_closing_tag: config.omit_last_closing_tag,
            built_ins: config.built_ins,
            template_state: DomTemplateState::new(),
            error: None,
            static_bindings: std::vec::Vec::new(),
            const_bindings: std::vec::Vec::new(),
            function_bindings: std::vec::Vec::new(),
            pending_this_capture: None,
            current_this_capture: None,
            statement_depth: 0,
            skip_xmlns_attribute: false,
            hydration_walk_anchor: None,
            has_hydratable_event: false,
            element_index: 0,
            this_index: 0,
            ref_index: 0,
            condition_index: 0,
        }
    }

    pub(crate) fn lower_element(&mut self, element: &JSXElement<'a>) -> Result<Expression<'a>> {
        let (result, setup) = self.lower_element_with_setup(element)?;
        if setup.is_empty() {
            return Ok(result);
        }

        let mut statements = self.ast().vec();
        statements.extend(setup);
        statements.push(self.ast().statement_return(element.span, Some(result)));
        let arrow = self.arrow_iife(element.span, statements);
        Ok(self.call_expression(element.span, arrow, std::vec::Vec::new()))
    }

    pub(crate) fn lower_element_with_setup(
        &mut self,
        element: &JSXElement<'a>,
    ) -> Result<(Expression<'a>, std::vec::Vec<Statement<'a>>)> {
        if is_component_name(&element.opening_element.name) {
            return lower_component_with_setup(self, element);
        }

        let tag_name = element_name(&element.opening_element.name)?;

        // Each native template root replays hydratable events independently
        // (Babel transforms component children with `topLevel: true`).
        let saved_hydratable_event = self.has_hydratable_event;
        self.has_hydratable_event = false;

        // A non-literal `children` attribute participates in child insertion
        // rather than attribute handling (Babel parity): when the element has
        // no real children, the value becomes its child expression; when it
        // does, the attribute is dropped.
        let element: &JSXElement<'a> = if element.children.is_empty() {
            if let Some(container) = children_attribute_container(element) {
                let mut clone = element.clone_in(self.allocator);
                clone
                    .children
                    .push(oxc_ast::ast::JSXChild::ExpressionContainer(
                        oxc_allocator::Box::new_in(
                            container.clone_in(self.allocator),
                            self.allocator,
                        ),
                    ));
                self.allocator.alloc(clone)
            } else {
                element
            }
        } else {
            element
        };

        // XML partial handling (Babel parity): template-root SVG/MathML
        // elements other than <svg>/<math> themselves get wrapped in their
        // owner tag and flagged, and the `xmlns` attribute (only needed to
        // detect the namespace) is dropped from the template.
        let wrapper_tag = self.xml_wrapper_tag(element, &tag_name);
        let skip_xmlns = wrapper_tag.is_some() || tag_name == "svg" || tag_name == "math";

        let mut template = format!("<{tag_name}");
        let mut declarations = std::vec::Vec::new();
        let mut operations = std::vec::Vec::new();
        let element_id = self.next_element_id();

        let saved_skip_xmlns = self.skip_xmlns_attribute;
        self.skip_xmlns_attribute = skip_xmlns;
        let attribute_result = self.lower_template_attributes(
            &element.opening_element.attributes,
            &tag_name,
            &element_id,
            !element.children.is_empty(),
            &mut template,
            &mut operations,
        );
        self.skip_xmlns_attribute = saved_skip_xmlns;
        attribute_result?;

        template.push('>');
        self.lower_dom_children(
            element,
            &tag_name,
            &element_id,
            &mut template,
            &mut declarations,
            &mut operations,
        )?;
        if self.should_close_tag(&tag_name, CloseTagContext::root()) {
            template.push_str(&format!("</{tag_name}>"));
        }
        if let Some(wrapper) = wrapper_tag {
            template = format!("<{wrapper}>{template}</{wrapper}>");
        }

        let needs_custom_element_context =
            self.should_capture_custom_element_context(element, &tag_name);
        let template_flag = if wrapper_tag.is_some() {
            Some(2)
        } else if self.template_subtree_is_import_node(element) {
            Some(1)
        } else {
            None
        };
        // Babel's `skipTemplate`: `$ServerOnly` elements and document shells
        // (`html`/`head`/`body`) never render client-side markup — the element
        // is only recovered from the hydration walk.
        let skip_template = self.hydratable
            && (has_attribute_named(element, "$ServerOnly")
                || matches!(tag_name.as_str(), "html" | "head" | "body"));
        let template_id = if skip_template {
            None
        } else {
            Some(self.template_id_with_options(template, template_flag))
        };
        let has_hydratable_event = self.has_hydratable_event;
        self.has_hydratable_event = saved_hydratable_event;

        if declarations.is_empty()
            && operations.is_empty()
            && !needs_custom_element_context
            && !has_hydratable_event
        {
            Ok((
                self.template_call(element.span, template_id.as_deref()),
                std::vec::Vec::new(),
            ))
        } else {
            let init = self.template_call(element.span, template_id.as_deref());
            let mut setup = std::vec::Vec::new();
            setup.push(self.variable_statement(element.span, &element_id, init));
            // Babel hoists all positional walk declarations ahead of the
            // effectful statements (attribute setters, inserts), so walks are
            // resolved before inserts mutate sibling positions.
            setup.extend(declarations);
            setup.extend(operations);
            if needs_custom_element_context {
                setup.push(self.custom_element_context_statement(element.span, &element_id));
            }
            if has_hydratable_event {
                self.template_state.uses_run_hydration_events = true;
                setup.push(self.ast().statement_expression(
                    element.span,
                    self.call_identifier(
                        element.span,
                        "_$runHydrationEvents",
                        std::vec::Vec::new(),
                    ),
                ));
            }
            Ok((self.identifier_expression(element.span, &element_id), setup))
        }
    }

    fn template_call(&mut self, span: oxc_span::Span, template_id: Option<&str>) -> Expression<'a> {
        if self.hydratable {
            self.template_state.uses_get_next_element = true;
            let args = match template_id {
                Some(template_id) => vec![self.identifier_expression(span, template_id)],
                None => std::vec::Vec::new(),
            };
            self.call_identifier(span, "_$getNextElement", args)
        } else {
            let template_id =
                template_id.expect("non-hydratable templates are always registered");
            self.call_identifier(span, template_id, std::vec::Vec::new())
        }
    }

    fn should_capture_custom_element_context(
        &self,
        element: &JSXElement<'a>,
        tag_name: &str,
    ) -> bool {
        self.context_to_custom_elements
            && (tag_name == "slot" || self.has_custom_element_marker(element, tag_name))
    }

    fn has_custom_element_marker(&self, element: &JSXElement<'a>, tag_name: &str) -> bool {
        tag_name.contains('-') || has_attribute_named(element, "is")
    }

    /// Owner tag (`svg` / `math`) for a template-root XML partial, detected
    /// by element name or an explicit `xmlns` attribute, mirroring the Babel
    /// plugin's top-level XML handling.
    fn xml_wrapper_tag(&self, element: &JSXElement<'a>, tag_name: &str) -> Option<&'static str> {
        if tag_name == "svg" || tag_name == "math" {
            return None;
        }
        let xmlns = xmlns_attribute_value(element);
        if crate::shared::constants::svg_elements(tag_name)
            || xmlns.as_deref() == Some("http://www.w3.org/2000/svg")
        {
            return Some("svg");
        }
        if crate::shared::constants::mathml_elements(tag_name)
            || xmlns.as_deref() == Some("http://www.w3.org/1998/Math/MathML")
        {
            return Some("math");
        }
        None
    }

    /// Whether any native element in the template's subtree requires
    /// `importNode` cloning (custom elements, `is` attributes, or lazy-loading
    /// img/iframe). Component subtrees produce their own templates and are
    /// not descended into.
    fn template_subtree_is_import_node(&self, element: &JSXElement<'a>) -> bool {
        if is_component_name(&element.opening_element.name) {
            return false;
        }
        let Ok(tag_name) = element_name(&element.opening_element.name) else {
            return false;
        };
        if self.has_custom_element_marker(element, &tag_name)
            || ((tag_name == "img" || tag_name == "iframe")
                && has_attribute_named(element, "loading"))
        {
            return true;
        }
        element.children.iter().any(|child| {
            matches!(
                child,
                oxc_ast::ast::JSXChild::Element(child)
                    if self.template_subtree_is_import_node(child)
            )
        })
    }

    fn custom_element_context_statement(
        &mut self,
        span: oxc_span::Span,
        element_id: &str,
    ) -> Statement<'a> {
        self.template_state.uses_get_owner = true;
        let target = AssignmentTarget::StaticMemberExpression(
            self.ast().alloc_static_member_expression(
                span,
                self.identifier_expression(span, element_id),
                self.ast()
                    .identifier_name(span, self.ast().ident("_$owner")),
                false,
            ),
        );
        let value = self.call_identifier(span, "_$getOwner", std::vec::Vec::new());
        self.ast().statement_expression(
            span,
            self.ast()
                .expression_assignment(span, AssignmentOperator::Assign, target, value),
        )
    }
}

fn has_attribute_named(element: &JSXElement<'_>, attribute_name: &str) -> bool {
    element.opening_element.attributes.iter().any(|attr| {
        matches!(
            attr,
            oxc_ast::ast::JSXAttributeItem::Attribute(attribute)
                if matches!(
                    &attribute.name,
                    oxc_ast::ast::JSXAttributeName::Identifier(name)
                        if name.name == attribute_name
                )
        )
    })
}

/// Static string value of an element's `xmlns` attribute, if present.
fn xmlns_attribute_value(element: &JSXElement<'_>) -> Option<String> {
    element.opening_element.attributes.iter().find_map(|attr| {
        let oxc_ast::ast::JSXAttributeItem::Attribute(attr) = attr else {
            return None;
        };
        let oxc_ast::ast::JSXAttributeName::Identifier(name) = &attr.name else {
            return None;
        };
        if name.name != "xmlns" {
            return None;
        }
        match &attr.value {
            Some(oxc_ast::ast::JSXAttributeValue::StringLiteral(value)) => {
                Some(value.value.to_string())
            }
            Some(oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container)) => {
                match &container.expression {
                    JSXExpression::StringLiteral(value) => Some(value.value.to_string()),
                    _ => None,
                }
            }
            _ => None,
        }
    })
}

/// Matches the Babel plugin's `children`-attribute capture: a `children`
/// attribute with a non-literal expression container value is treated as
/// element children (insert), not as an attribute or property.
pub(crate) fn children_attribute_container<'e, 'a>(
    element: &'e JSXElement<'a>,
) -> Option<&'e oxc_ast::ast::JSXExpressionContainer<'a>> {
    element
        .opening_element
        .attributes
        .iter()
        .find_map(|attr| children_attribute_container_from_item(attr))
}

pub(crate) fn children_attribute_container_from_item<'e, 'a>(
    attr: &'e oxc_ast::ast::JSXAttributeItem<'a>,
) -> Option<&'e oxc_ast::ast::JSXExpressionContainer<'a>> {
    let oxc_ast::ast::JSXAttributeItem::Attribute(attr) = attr else {
        return None;
    };
    let oxc_ast::ast::JSXAttributeName::Identifier(name) = &attr.name else {
        return None;
    };
    if name.name != "children" {
        return None;
    }
    let Some(oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
        return None;
    };
    if matches!(
        container.expression,
        JSXExpression::StringLiteral(_)
            | JSXExpression::NumericLiteral(_)
            | JSXExpression::BooleanLiteral(_)
            | JSXExpression::EmptyExpression(_)
    ) {
        return None;
    }
    Some(container)
}

pub(crate) fn jsx_expression_to_expression<'a>(
    expression: &JSXExpression<'a>,
    allocator: &'a Allocator,
) -> Expression<'a> {
    expression.clone_in(allocator).into_expression()
}

impl AstDomTransform<'_, '_> {
    pub(crate) fn static_jsx_expression_value(
        &self,
        expression: &JSXExpression<'_>,
    ) -> Option<String> {
        static_jsx_expression(expression, &self.static_bindings)
            .map(StaticValue::into_template_value)
    }

    /// Whether the source region carries the configured static marker comment
    /// (`/*@static*/` by default), opting the value out of effect wrapping.
    pub(crate) fn has_static_marker(&self, span: oxc_span::Span) -> bool {
        crate::shared::utils::source_from_span(span, self.source).contains(&self.static_marker)
    }
}
