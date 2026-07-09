use napi::bindgen_prelude::*;
use oxc_allocator::{CloneIn, Vec as ArenaVec};
use oxc_ast::{
    ast::{Expression, FormalParameterKind, JSXAttributeItem, JSXAttributeValue, Statement},
    NONE,
};
use oxc_span::Span;

use crate::dom::element::{jsx_expression_to_expression, AstDomTransform};
use crate::dom::style::static_style_object_value;
use crate::shared::bindings::push_unique;
use crate::shared::constants::{
    child_properties, dom_with_state, has_namespace, inline_elements, namespaces,
    ALWAYS_CLOSE_ELEMENTS, BLOCK_ELEMENTS,
};
use crate::shared::refs::{ref_assignment_fallback, ref_callable_test};
use crate::shared::utils::{
    decode_html_entities, dedupe_attributes, format_attribute_value_with_quotes,
    is_dynamic_attribute_expression, is_void_element, static_jsx_expression_value,
};

pub(crate) enum StaticAttributeResult {
    Appended,
    Dynamic,
    Spread,
}

#[derive(Clone, Default)]
pub(crate) struct CloseTagContext {
    pub(crate) last_element: bool,
    pub(crate) to_be_closed: Option<std::vec::Vec<String>>,
}

impl CloseTagContext {
    pub(crate) fn root() -> Self {
        Self {
            last_element: true,
            to_be_closed: None,
        }
    }
}

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn lower_template_attributes(
        &mut self,
        attributes: &[JSXAttributeItem<'a>],
        tag_name: &str,
        element_id: &str,
        skip_children: bool,
        template: &mut String,
        operations: &mut std::vec::Vec<Statement<'a>>,
    ) -> Result<()> {
        if attributes
            .iter()
            .any(|attr| matches!(attr, JSXAttributeItem::SpreadAttribute(_)))
        {
            operations.push(self.spread_attribute_statement(
                attributes,
                element_id,
                skip_children,
            )?);
            return Ok(());
        }

        for attr in dedupe_attributes(attributes) {
            // The static marker comment (`/*@static*/` by default) opts an
            // attribute value out of effect wrapping entirely, mirroring the
            // Babel plugin's isDynamic short-circuit.
            let marker_static = matches!(
                attr,
                JSXAttributeItem::Attribute(attribute)
                    if matches!(
                        &attribute.value,
                        Some(JSXAttributeValue::ExpressionContainer(container))
                            if self.has_static_marker(container.span)
                    )
            );
            let saved_effect_wrapper = self.effect_wrapper;
            if marker_static {
                self.effect_wrapper = false;
            }
            let result = self.lower_single_attribute(attr, tag_name, element_id, template, operations);
            self.effect_wrapper = saved_effect_wrapper;
            result?;
        }
        Ok(())
    }

    fn lower_single_attribute(
        &mut self,
        attr: &JSXAttributeItem<'a>,
        tag_name: &str,
        element_id: &str,
        template: &mut String,
        operations: &mut std::vec::Vec<Statement<'a>>,
    ) -> Result<()> {
        // Non-literal `children` attributes are consumed by child insertion
        // (see `lower_element_with_setup`), never emitted as attributes.
        if crate::dom::element::children_attribute_container_from_item(attr).is_some() {
            return Ok(());
        }
        // `$ServerOnly` is a directive (handled at the element level in
        // hydratable mode), not a real attribute.
        if self.hydratable
            && matches!(
                attr,
                JSXAttributeItem::Attribute(attribute)
                    if matches!(
                        &attribute.name,
                        oxc_ast::ast::JSXAttributeName::Identifier(name)
                            if name.name == "$ServerOnly"
                    )
            )
        {
            return Ok(());
        }
        // The `xmlns` attribute on template-root XML elements only signals
        // the namespace; it is dropped from the serialized template.
        if self.skip_xmlns_attribute
            && matches!(
                attr,
                JSXAttributeItem::Attribute(attribute)
                    if matches!(
                        &attribute.name,
                        oxc_ast::ast::JSXAttributeName::Identifier(name) if name.name == "xmlns"
                    )
            )
        {
            return Ok(());
        }
        if let Some(statement) = self.attribute_operation_statement(attr, element_id)? {
            operations.push(statement);
            return Ok(());
        }
        if self.class_array_attribute_operations(attr, element_id, template, operations)?
            || self.style_object_attribute_operations(attr, element_id, template, operations)?
        {
            return Ok(());
        }
        match append_static_template_attribute(Some(self), attr, template)? {
            StaticAttributeResult::Appended => {}
            StaticAttributeResult::Spread => {
                return Err(Error::from_reason(
                    "Spread attributes are not implemented in the AST-native milestone yet",
                ));
            }
            StaticAttributeResult::Dynamic => {
                operations.push(self.dynamic_attribute_statement(attr, tag_name, element_id)?);
            }
        }
        Ok(())
    }

    fn attribute_operation_statement(
        &mut self,
        attr: &JSXAttributeItem<'a>,
        element_id: &str,
    ) -> Result<Option<Statement<'a>>> {
        if let Some(statement) = self.no_inline_style_attribute_statement(attr, element_id)? {
            return Ok(Some(statement));
        }
        if let Some(statement) = self.prop_attribute_statement(attr, element_id)? {
            return Ok(Some(statement));
        }
        self.child_property_attribute_statement(attr, element_id)
    }

    fn dynamic_attribute_statement(
        &mut self,
        attr: &JSXAttributeItem<'a>,
        tag_name: &str,
        element_id: &str,
    ) -> Result<Statement<'a>> {
        let JSXAttributeItem::Attribute(attr) = attr else {
            return Err(Error::from_reason(
                "Spread attributes are not implemented in the AST-native milestone yet",
            ));
        };
        let oxc_ast::ast::JSXAttributeName::Identifier(name) = &attr.name else {
            if let Some(statement) = self.namespaced_attribute_statement(attr, element_id)? {
                return Ok(statement);
            }
            return Err(Error::from_reason(
                "Namespaced attributes are not implemented in the AST-native milestone yet",
            ));
        };
        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            return Err(Error::from_reason(
                "Only expression dynamic DOM attributes are implemented in the AST-native milestone",
            ));
        };
        if name.name.starts_with("on") {
            return Ok(self.event_statement(
                attr.span,
                element_id,
                &name.name,
                &container.expression,
            ));
        }
        if name.name == "style" {
            let value = jsx_expression_to_expression(&container.expression, self.allocator);
            return Ok(self.dynamic_style_statement(attr.span, element_id, value));
        }
        if name.name == "class" || name.name == "className" {
            if let Some(statement) =
                self.class_object_statement(attr.span, element_id, &container.expression)
            {
                return Ok(statement);
            }
            let value = jsx_expression_to_expression(&container.expression, self.allocator);
            return Ok(self.dynamic_class_statement(attr.span, element_id, value));
        }
        if name.name == "ref" {
            let value = jsx_expression_to_expression(&container.expression, self.allocator);
            return Ok(self.dom_ref_statement(attr.span, element_id, value));
        }
        if child_properties(&name.name) {
            let value = jsx_expression_to_expression(&container.expression, self.allocator);
            return Ok(self.dynamic_property_statement(attr.span, element_id, &name.name, value));
        }
        if dom_with_state(tag_name, &name.name).is_some() {
            let value = jsx_expression_to_expression(&container.expression, self.allocator);
            return Ok(self.dynamic_property_statement(attr.span, element_id, &name.name, value));
        }
        if is_special_dynamic_attribute(tag_name, &name.name) {
            return Err(Error::from_reason(
                "Special dynamic DOM attributes are not implemented in the AST-native milestone yet",
            ));
        }

        self.template_state.uses_set_attribute = true;

        let value = jsx_expression_to_expression(&container.expression, self.allocator);
        if !self.effect_wrapper || !is_dynamic_attribute_expression(&value) {
            return Ok(self.ast().statement_expression(
                attr.span,
                self.call_identifier(
                    attr.span,
                    "_$setAttribute",
                    vec![
                        self.identifier_expression(attr.span, element_id),
                        self.ast().expression_string_literal(
                            attr.span,
                            self.ast().atom(&name.name),
                            None,
                        ),
                        value,
                    ],
                ),
            ));
        }

        self.template_state.uses_effect = true;
        let getter = self.arrow_with_return(attr.span, std::vec::Vec::new(), value);
        let setter = self.dynamic_set_attribute_callback(attr.span, element_id, &name.name);

        Ok(self.ast().statement_expression(
            attr.span,
            self.call_identifier(attr.span, "_$effect", vec![getter, setter]),
        ))
    }

    fn namespaced_attribute_statement(
        &mut self,
        attr: &oxc_ast::ast::JSXAttribute<'a>,
        element_id: &str,
    ) -> Result<Option<Statement<'a>>> {
        let oxc_ast::ast::JSXAttributeName::NamespacedName(name) = &attr.name else {
            return Ok(None);
        };
        let Some(namespace) = namespaces(&name.namespace.name) else {
            return Ok(None);
        };
        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            return Err(Error::from_reason(
                "Only expression namespaced DOM attributes are implemented in the AST-native milestone",
            ));
        };

        self.template_state.uses_set_attribute_ns = true;
        let local_name = format!("{}:{}", name.namespace.name, name.name.name);
        Ok(Some(self.ast().statement_expression(
            attr.span,
            self.call_identifier(
                attr.span,
                "_$setAttributeNS",
                vec![
                    self.identifier_expression(attr.span, element_id),
                    self.ast().expression_string_literal(
                        attr.span,
                        self.ast().atom(namespace),
                        None,
                    ),
                    self.ast().expression_string_literal(
                        attr.span,
                        self.ast().atom(&local_name),
                        None,
                    ),
                    jsx_expression_to_expression(&container.expression, self.allocator),
                ],
            ),
        )))
    }

    fn dom_ref_statement(
        &mut self,
        span: Span,
        element_id: &str,
        value: Expression<'a>,
    ) -> Statement<'a> {
        if matches!(
            value,
            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
        ) {
            self.template_state.uses_ref = true;
            let getter = self.arrow_with_return(span, std::vec::Vec::new(), value);
            return self.ast().statement_expression(
                span,
                self.call_identifier(
                    span,
                    "_$ref",
                    vec![getter, self.identifier_expression(span, element_id)],
                ),
            );
        }

        let ref_id = self.next_ref_id();
        let ref_identifier = self.identifier_expression(span, &ref_id);
        let mut statements = self.ast().vec();
        statements.push(self.variable_statement(span, &ref_id, value.clone_in(self.allocator)));
        let callable = ref_callable_test(self, span, ref_identifier.clone_in(self.allocator));
        let ref_call = {
            self.template_state.uses_ref = true;
            let getter = self.arrow_with_return(
                span,
                std::vec::Vec::new(),
                ref_identifier.clone_in(self.allocator),
            );
            self.call_identifier(
                span,
                "_$ref",
                vec![getter, self.identifier_expression(span, element_id)],
            )
        };
        let fallback = ref_assignment_fallback(
            self,
            span,
            &value,
            self.identifier_expression(span, element_id),
        );

        let statement = match fallback {
            Some(fallback) => self.ast().statement_expression(
                span,
                self.ast()
                    .expression_conditional(span, callable, ref_call, fallback),
            ),
            None => self.ast().statement_expression(
                span,
                self.ast().expression_logical(
                    span,
                    callable,
                    oxc_ast::ast::LogicalOperator::And,
                    ref_call,
                ),
            ),
        };
        statements.push(statement);
        self.ast().statement_block(span, statements)
    }

    fn dynamic_set_attribute_callback(
        &self,
        span: Span,
        element_id: &str,
        name: &str,
    ) -> Expression<'a> {
        let value_id = "_v$";
        let statement = self.ast().statement_expression(
            span,
            self.call_identifier(
                span,
                "_$setAttribute",
                vec![
                    self.identifier_expression(span, element_id),
                    self.ast()
                        .expression_string_literal(span, self.ast().atom(name), None),
                    self.identifier_expression(span, value_id),
                ],
            ),
        );
        self.arrow_with_statements(span, vec![value_id], self.ast().vec1(statement))
    }

    pub(crate) fn arrow_with_return(
        &self,
        span: Span,
        param_names: std::vec::Vec<&str>,
        value: Expression<'a>,
    ) -> Expression<'a> {
        let statements = self
            .ast()
            .vec1(self.ast().statement_return(span, Some(value)));
        self.arrow_with_statements(span, param_names, statements)
    }

    pub(crate) fn arrow_with_statements(
        &self,
        span: Span,
        param_names: std::vec::Vec<&str>,
        statements: ArenaVec<'a, Statement<'a>>,
    ) -> Expression<'a> {
        let params = self
            .ast()
            .vec_from_iter(param_names.into_iter().map(|name| {
                self.ast().formal_parameter(
                    span,
                    self.ast().vec(),
                    self.ast()
                        .binding_pattern_binding_identifier(span, self.ast().ident(name)),
                    NONE,
                    NONE,
                    false,
                    None,
                    false,
                    false,
                )
            }));
        let params = self.ast().formal_parameters(
            span,
            FormalParameterKind::ArrowFormalParameters,
            params,
            NONE,
        );
        let body = self.ast().function_body(span, self.ast().vec(), statements);
        self.ast()
            .expression_arrow_function(span, false, false, NONE, params, NONE, body)
    }

    pub(crate) fn should_close_tag(&self, tag_name: &str, context: CloseTagContext) -> bool {
        if is_void_element(tag_name) {
            return false;
        }
        !context.last_element
            || !self.omit_last_closing_tag
            || context.to_be_closed.as_ref().is_some_and(|to_be_closed| {
                !self.omit_nested_closing_tags
                    || to_be_closed.iter().any(|candidate| candidate == tag_name)
            })
    }

    pub(crate) fn append_static_attribute_value(
        &self,
        template: &mut String,
        name: &str,
        value: &str,
    ) {
        append_static_attribute_value(
            template,
            name,
            value,
            self.omit_quotes,
            self.omit_attribute_spacing,
        );
    }

    pub(crate) fn child_close_context(
        &self,
        tag_name: &str,
        context: CloseTagContext,
    ) -> Option<std::vec::Vec<String>> {
        if !self.should_close_tag(tag_name, context.clone()) {
            return context.to_be_closed;
        }

        let mut to_be_closed = context.to_be_closed.unwrap_or_else(|| {
            ALWAYS_CLOSE_ELEMENTS
                .iter()
                .map(|name| (*name).to_string())
                .collect()
        });
        push_unique(&mut to_be_closed, tag_name);
        if inline_elements(tag_name) {
            for element in BLOCK_ELEMENTS {
                push_unique(&mut to_be_closed, element);
            }
        }
        Some(to_be_closed)
    }
}

fn attribute_prefix(template: &str, omit_attribute_spacing: bool) -> &'static str {
    // The Babel plugin skips the separating space after a quoted value when
    // `omitAttributeSpacing` is on; a template ending in `"` is exactly that.
    if omit_attribute_spacing && template.ends_with('"') {
        ""
    } else {
        " "
    }
}

fn append_bare_attribute(template: &mut String, name: &str, omit_attribute_spacing: bool) {
    let prefix = attribute_prefix(template, omit_attribute_spacing);
    template.push_str(&format!("{prefix}{name}"));
}

fn append_static_attribute_value(
    template: &mut String,
    name: &str,
    value: &str,
    omit_quotes: bool,
    omit_attribute_spacing: bool,
) {
    let value = normalize_static_attribute_value(name, value);
    // An empty value (after class/style normalization) serializes as a bare
    // attribute, matching the Babel plugin.
    if value.is_empty() {
        append_bare_attribute(template, name, omit_attribute_spacing);
        return;
    }
    let prefix = attribute_prefix(template, omit_attribute_spacing);
    template.push_str(&format!(
        "{prefix}{name}={}",
        format_attribute_value_with_quotes(&value, omit_quotes)
    ));
}

fn normalize_static_attribute_value(name: &str, value: &str) -> String {
    if name != "style" && name != "class" {
        return value.to_string();
    }

    let mut normalized = String::new();
    let mut previous_was_whitespace = false;
    for char in value.chars().filter(|char| *char != '\r') {
        if char.is_whitespace() {
            if !previous_was_whitespace {
                normalized.push(' ');
                previous_was_whitespace = true;
            }
        } else {
            normalized.push(char);
            previous_was_whitespace = false;
        }
    }

    if name == "style" {
        normalized.replace("; ", ";").replace(": ", ":")
    } else {
        normalized
    }
}

pub(crate) fn try_append_static_template_attributes(
    ctx: Option<&AstDomTransform<'_, '_>>,
    attributes: &[JSXAttributeItem<'_>],
    template: &mut String,
) -> Result<bool> {
    for attr in dedupe_attributes(attributes) {
        match append_static_template_attribute(ctx, attr, template)? {
            StaticAttributeResult::Appended => {}
            StaticAttributeResult::Dynamic | StaticAttributeResult::Spread => return Ok(false),
        }
    }
    Ok(true)
}

fn append_static_template_attribute(
    ctx: Option<&AstDomTransform<'_, '_>>,
    attr: &JSXAttributeItem<'_>,
    template: &mut String,
) -> Result<StaticAttributeResult> {
    let JSXAttributeItem::Attribute(attr) = attr else {
        return Ok(StaticAttributeResult::Spread);
    };
    let name = match &attr.name {
        oxc_ast::ast::JSXAttributeName::Identifier(name) => name,
        oxc_ast::ast::JSXAttributeName::NamespacedName(name) if name.namespace.name == "prop" => {
            return Ok(StaticAttributeResult::Dynamic);
        }
        oxc_ast::ast::JSXAttributeName::NamespacedName(name)
            if namespaces(&name.namespace.name).is_some() =>
        {
            return Ok(StaticAttributeResult::Dynamic);
        }
        oxc_ast::ast::JSXAttributeName::NamespacedName(_) => {
            return Err(Error::from_reason(
                "Namespaced attributes are not implemented in the AST-native milestone yet",
            ));
        }
    };

    // Child properties (innerHTML/textContent/...) are runtime property
    // assignments, never template markup.
    if child_properties(&name.name) {
        return Ok(StaticAttributeResult::Dynamic);
    }

    // `_hk` is the internal hydration-key attribute; the Babel plugin warns
    // and strips it (usually pasted-in SSR output). Match by dropping it.
    if name.name == "_hk" {
        return Ok(StaticAttributeResult::Appended);
    }

    let omit_attribute_spacing = ctx.is_none_or(|ctx| ctx.omit_attribute_spacing);
    match &attr.value {
        None => append_bare_attribute(template, &name.name, omit_attribute_spacing),
        Some(JSXAttributeValue::StringLiteral(value)) => {
            let value = decode_html_entities(&value.value);
            if let Some(ctx) = ctx {
                ctx.append_static_attribute_value(template, &name.name, &value);
            } else {
                append_static_attribute_value(template, &name.name, &value, true, true);
            }
        }
        Some(JSXAttributeValue::ExpressionContainer(container)) => {
            // Boolean literals control attribute presence rather than being
            // serialized: `attr={true}` -> bare attr, `attr={false}` -> omitted.
            // Null routes to the runtime setter, all mirroring the Babel plugin.
            match &container.expression {
                oxc_ast::ast::JSXExpression::BooleanLiteral(literal) => {
                    if literal.value {
                        append_bare_attribute(template, &name.name, omit_attribute_spacing);
                    }
                    return Ok(StaticAttributeResult::Appended);
                }
                oxc_ast::ast::JSXExpression::NullLiteral(_) => {
                    return Ok(StaticAttributeResult::Dynamic);
                }
                _ => {}
            }
            if name.name == "style" {
                if let Some(value) = static_style_object_value(&container.expression) {
                    if !value.is_empty() {
                        if let Some(ctx) = ctx {
                            ctx.append_static_attribute_value(template, "style", &value);
                        } else {
                            append_static_attribute_value(template, "style", &value, true, true);
                        }
                    }
                    return Ok(StaticAttributeResult::Appended);
                }
            }
            let value = ctx
                .and_then(|ctx| ctx.static_jsx_expression_value(&container.expression))
                .or_else(|| static_jsx_expression_value(&container.expression));
            let Some(value) = value else {
                return Ok(StaticAttributeResult::Dynamic);
            };
            if let Some(ctx) = ctx {
                ctx.append_static_attribute_value(template, &name.name, &value);
            } else {
                append_static_attribute_value(template, &name.name, &value, true, true);
            }
        }
        Some(JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_)) => {
            return Err(Error::from_reason(
                "JSX attribute element values are not implemented in the AST-native milestone yet",
            ));
        }
    }

    Ok(StaticAttributeResult::Appended)
}

fn is_special_dynamic_attribute(tag_name: &str, name: &str) -> bool {
    name == "ref"
        || name == "class"
        || name == "className"
        || name == "style"
        || has_namespace(name)
        || child_properties(name)
        || dom_with_state(tag_name, name).is_some()
}
