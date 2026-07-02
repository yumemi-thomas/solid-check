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
        let mut template = format!("<{tag_name}");
        let mut operations = std::vec::Vec::new();
        let element_id = self.next_element_id();

        self.lower_template_attributes(
            &element.opening_element.attributes,
            &tag_name,
            &element_id,
            !element.children.is_empty(),
            &mut template,
            &mut operations,
        )?;

        template.push('>');
        self.lower_dom_children(
            element,
            &tag_name,
            &element_id,
            &mut template,
            &mut operations,
        )?;
        if self.should_close_tag(&tag_name, CloseTagContext::root()) {
            template.push_str(&format!("</{tag_name}>"));
        }

        let needs_custom_element_context =
            self.should_capture_custom_element_context(element, &tag_name);
        let template_id = self
            .template_id_with_options(template, self.has_custom_element_marker(element, &tag_name));
        if operations.is_empty() && !needs_custom_element_context {
            Ok((
                self.template_call(element.span, &template_id),
                std::vec::Vec::new(),
            ))
        } else {
            let init = self.template_call(element.span, &template_id);
            let mut setup = std::vec::Vec::new();
            setup.push(self.variable_statement(element.span, &element_id, init));
            setup.extend(operations);
            if needs_custom_element_context {
                setup.push(self.custom_element_context_statement(element.span, &element_id));
            }
            Ok((self.identifier_expression(element.span, &element_id), setup))
        }
    }

    fn template_call(&mut self, span: oxc_span::Span, template_id: &str) -> Expression<'a> {
        if self.hydratable {
            self.template_state.uses_get_next_element = true;
            self.call_identifier(
                span,
                "_$getNextElement",
                vec![self.identifier_expression(span, template_id)],
            )
        } else {
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
        tag_name.contains('-')
            || element.opening_element.attributes.iter().any(|attr| {
                matches!(
                    attr,
                    oxc_ast::ast::JSXAttributeItem::Attribute(attribute)
                        if matches!(
                            &attribute.name,
                            oxc_ast::ast::JSXAttributeName::Identifier(name) if name.name == "is"
                        )
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
}
