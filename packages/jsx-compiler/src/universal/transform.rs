use napi::bindgen_prelude::*;
use oxc_allocator::{Allocator, CloneIn, Vec as ArenaVec};
use oxc_ast::{
    ast::{
        Argument, ArrayExpressionElement, Expression, JSXAttributeItem, JSXAttributeValue,
        JSXChild, JSXElement, JSXExpression, JSXFragment, ObjectPropertyKind, Program, Statement,
    },
    AstBuilder,
};
use oxc_ast_visit::VisitMut;
use oxc_span::{GetSpan, Span};

use crate::dom::element::{AstDomTransform, DomTransformConfig};
use crate::shared::array::expression_to_array_element;
use crate::shared::ast::arrow_return_expression;
use crate::shared::ast::expression_to_argument;
use crate::shared::component_callee::component_callee_expression;
use crate::shared::component_props::{
    component_property, component_props_expression, component_spread_expression,
    flush_component_props, ComponentPropContext,
};
use crate::shared::utils::{
    decode_html_entities, element_name, is_component_name, source_from_span,
    static_jsx_expression_value, trim_jsx_text,
};

pub(crate) struct AstUniversalTransform<'a, 'source> {
    pub(super) allocator: &'a Allocator,
    source: &'source str,
    pub(super) module_name: &'source str,
    pub(super) built_ins: std::vec::Vec<String>,
    pub(super) built_in_imports: std::vec::Vec<String>,
    pub(super) element_index: usize,
    uses_create_element: bool,
    uses_create_component: bool,
    uses_merge_props: bool,
    uses_spread: bool,
    uses_create_text_node: bool,
    uses_insert: bool,
    uses_insert_node: bool,
    uses_set_prop: bool,
    dynamic_dom: Option<AstDomTransform<'a, 'source>>,
    dynamic_dom_elements: std::vec::Vec<String>,
    static_marker: String,
    pub(crate) error: Option<String>,
}

pub(crate) struct DynamicDomConfig<'source> {
    pub(crate) module_name: &'source str,
    pub(crate) elements: std::vec::Vec<String>,
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
}

impl<'a, 'source> AstUniversalTransform<'a, 'source> {
    pub(crate) fn new(
        allocator: &'a Allocator,
        source: &'source str,
        module_name: &'source str,
        built_ins: std::vec::Vec<String>,
        static_marker: String,
    ) -> Self {
        Self {
            allocator,
            source,
            module_name,
            built_ins,
            built_in_imports: std::vec::Vec::new(),
            element_index: 0,
            uses_create_element: false,
            uses_create_component: false,
            uses_merge_props: false,
            uses_spread: false,
            uses_create_text_node: false,
            uses_insert: false,
            uses_insert_node: false,
            uses_set_prop: false,
            dynamic_dom: None,
            dynamic_dom_elements: std::vec::Vec::new(),
            static_marker,
            error: None,
        }
    }

    pub(crate) fn new_dynamic(
        allocator: &'a Allocator,
        source: &'source str,
        module_name: &'source str,
        built_ins: std::vec::Vec<String>,
        dom: DynamicDomConfig<'source>,
    ) -> Self {
        Self {
            allocator,
            source,
            module_name,
            built_ins: built_ins.clone(),
            built_in_imports: std::vec::Vec::new(),
            element_index: 0,
            uses_create_element: false,
            uses_create_component: false,
            uses_merge_props: false,
            uses_spread: false,
            uses_create_text_node: false,
            uses_insert: false,
            uses_insert_node: false,
            uses_set_prop: false,
            dynamic_dom: Some(AstDomTransform::new(
                allocator,
                source,
                dom.module_name,
                DomTransformConfig {
                    hydratable: dom.hydratable,
                    dev: dom.dev,
                    context_to_custom_elements: dom.context_to_custom_elements,
                    delegate_events: dom.delegate_events,
                    delegated_events: dom.delegated_events,
                    omit_quotes: dom.omit_quotes,
                    omit_attribute_spacing: dom.omit_attribute_spacing,
                    inline_styles: dom.inline_styles,
                    effect_wrapper: dom.effect_wrapper,
                    wrap_conditionals: dom.wrap_conditionals,
                    memo_wrapper: dom.memo_wrapper,
                    static_marker: dom.static_marker.clone(),
                    omit_nested_closing_tags: dom.omit_nested_closing_tags,
                    omit_last_closing_tag: dom.omit_last_closing_tag,
                    built_ins,
                },
            )),
            dynamic_dom_elements: dom.elements,
            static_marker: dom.static_marker,
            error: None,
        }
    }

    pub(crate) fn prepend_helpers(&mut self, program: &mut Program<'a>) {
        let mut statements = std::vec::Vec::new();
        if self.uses_create_text_node {
            statements
                .push(self.import_named("createTextNode", &self.helper_local("_$createTextNode")));
        }
        if let Some(dom) = &mut self.dynamic_dom {
            if let Err(error) = dom.prepend_helpers(program) {
                self.error = Some(error.to_string());
                return;
            }
        }
        if self.uses_create_component {
            statements.push(
                self.import_named("createComponent", &self.helper_local("_$createComponent")),
            );
        }
        if self.uses_merge_props {
            statements.push(self.import_named("mergeProps", &self.helper_local("_$mergeProps")));
        }
        if self.uses_spread {
            statements.push(self.import_named("spread", &self.helper_local("_$spread")));
        }
        if self.uses_insert {
            statements.push(self.import_named("insert", &self.helper_local("_$insert")));
        }
        if self.uses_insert_node {
            statements.push(self.import_named("insertNode", &self.helper_local("_$insertNode")));
        }
        if self.uses_set_prop {
            statements.push(self.import_named("setProp", &self.helper_local("_$setProp")));
        }
        if self.uses_create_element {
            statements
                .push(self.import_named("createElement", &self.helper_local("_$createElement")));
        }
        for built_in in &self.built_in_imports {
            statements.push(self.import_named(built_in, &format!("_${built_in}")));
        }
        statements.extend(program.body.drain(..));
        let mut body = ArenaVec::new_in(self.allocator);
        body.extend(statements);
        program.body = body;
    }

    fn helper_local(&self, local: &str) -> String {
        if self.dynamic_dom.is_some() {
            format!("{local}2")
        } else {
            local.to_string()
        }
    }

    pub(crate) fn lower_element(
        &mut self,
        element: &JSXElement<'a>,
    ) -> Result<(Expression<'a>, std::vec::Vec<Statement<'a>>)> {
        if is_component_name(&element.opening_element.name) {
            return Ok((self.lower_component(element)?, std::vec::Vec::new()));
        }
        let tag_name = element_name(&element.opening_element.name)?;
        if self
            .dynamic_dom_elements
            .iter()
            .any(|name| name == &tag_name)
        {
            let Some(dom) = &mut self.dynamic_dom else {
                unreachable!("dynamic DOM elements require a DOM transform");
            };
            let value = dom.lower_element(element)?;
            return Ok((value, std::vec::Vec::new()));
        }
        let element_id = self.next_element_id();
        self.uses_create_element = true;
        let has_spread = element
            .opening_element
            .attributes
            .iter()
            .any(|attr| matches!(attr, JSXAttributeItem::SpreadAttribute(_)));
        let mut init_props = std::vec::Vec::new();
        if !has_spread {
            for attr in &element.opening_element.attributes {
                if let Some(prop) = self.init_attribute_property(attr)? {
                    init_props.push(prop);
                }
            }
        }
        let mut create_element_args = vec![self.string_arg(element.span, &tag_name)];
        if !init_props.is_empty() {
            create_element_args.push(expression_to_argument(
                self.ast()
                    .expression_object(element.span, self.ast().vec_from_iter(init_props)),
            ));
        }
        let mut setup = std::vec::Vec::new();
        setup.push(self.variable_statement(
            element.span,
            &element_id,
            self.call_identifier(
                element.span,
                &self.helper_local("_$createElement"),
                create_element_args,
            ),
        ));

        if has_spread {
            self.lower_spread_attributes(
                &element.opening_element.attributes,
                &element_id,
                !element.children.is_empty(),
                &mut setup,
            )?;
        } else {
            for attr in &element.opening_element.attributes {
                if self.init_attribute_property(attr)?.is_none() {
                    self.lower_attribute(attr, &element_id, &mut setup)?;
                }
            }
        }
        for child in &element.children {
            self.lower_child(child, &element_id, &mut setup)?;
        }
        Ok((self.identifier_expression(element.span, &element_id), setup))
    }

    fn lower_spread_attributes(
        &mut self,
        attributes: &[JSXAttributeItem<'a>],
        element_id: &str,
        skip_children: bool,
        setup: &mut std::vec::Vec<Statement<'a>>,
    ) -> Result<()> {
        self.uses_spread = true;
        let mut prop_objects = std::vec::Vec::new();
        let mut running_props = std::vec::Vec::new();
        for attr in attributes {
            match attr {
                JSXAttributeItem::SpreadAttribute(spread) => {
                    flush_component_props(self, &mut running_props, &mut prop_objects, spread.span);
                    prop_objects.push(spread.argument.clone_in(self.allocator));
                }
                JSXAttributeItem::Attribute(attr) => {
                    running_props.push(self.spread_attribute_property(attr)?);
                }
            }
        }
        flush_component_props(self, &mut running_props, &mut prop_objects, Span::new(0, 0));
        let props = component_props_expression(self, Span::new(0, 0), prop_objects, false);
        setup.push(self.expression_statement(
            Span::new(0, 0),
            self.call_identifier(
                Span::new(0, 0),
                &self.helper_local("_$spread"),
                vec![
                    self.identifier_arg(Span::new(0, 0), element_id),
                    expression_to_argument(props),
                    Argument::BooleanLiteral(
                        self.ast()
                            .alloc_boolean_literal(Span::new(0, 0), skip_children),
                    ),
                ],
            ),
        ));
        Ok(())
    }

    fn spread_attribute_property(
        &mut self,
        attr: &oxc_ast::ast::JSXAttribute<'a>,
    ) -> Result<ObjectPropertyKind<'a>> {
        let name = match &attr.name {
            oxc_ast::ast::JSXAttributeName::Identifier(name) => name.name.to_string(),
            oxc_ast::ast::JSXAttributeName::NamespacedName(name) => {
                format!("{}:{}", name.namespace.name, name.name.name)
            }
        };
        let value = match &attr.value {
            None => self.ast().expression_boolean_literal(attr.span, true),
            Some(JSXAttributeValue::StringLiteral(value)) => self.ast().expression_string_literal(
                value.span,
                self.ast().atom(&decode_html_entities(&value.value)),
                None,
            ),
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                if let Some(value) = static_jsx_expression_value(&container.expression) {
                    self.ast().expression_string_literal(
                        container.span,
                        self.ast().atom(&value),
                        None,
                    )
                } else {
                    crate::dom::element::jsx_expression_to_expression(
                        &container.expression,
                        self.allocator,
                    )
                }
            }
            Some(JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_)) => {
                return Err(Error::from_reason(
                    "Universal JSX attribute values are not implemented in the AST-native milestone yet",
                ));
            }
        };
        Ok(self.object_property(attr.span, &name, value))
    }

    fn lower_component(&mut self, element: &JSXElement<'a>) -> Result<Expression<'a>> {
        self.uses_create_component = true;
        let component = component_callee_expression(self, &element.opening_element.name)?;
        let mut prop_objects = std::vec::Vec::new();
        let mut running_props = std::vec::Vec::new();
        let mut force_merge_props = false;
        for attr in &element.opening_element.attributes {
            let attr = match attr {
                JSXAttributeItem::Attribute(attr) => attr,
                JSXAttributeItem::SpreadAttribute(spread) => {
                    flush_component_props(
                        self,
                        &mut running_props,
                        &mut prop_objects,
                        element.span,
                    );
                    let spread = component_spread_expression(self, &spread.argument, spread.span);
                    force_merge_props = force_merge_props || spread.force_merge;
                    prop_objects.push(spread.value);
                    continue;
                }
            };
            let name = match &attr.name {
                oxc_ast::ast::JSXAttributeName::Identifier(name) => name.name.to_string(),
                _ => {
                    return Err(Error::from_reason(
                        "Universal component namespace attributes are not implemented in the AST-native milestone yet",
                    ));
                }
            };
            let (value, needs_getter) = match &attr.value {
                None => (
                    self.ast().expression_boolean_literal(attr.span, true),
                    false,
                ),
                Some(JSXAttributeValue::StringLiteral(value)) => {
                    let value = decode_html_entities(&value.value);
                    (
                        self.ast().expression_string_literal(
                            attr.span,
                            self.ast().atom(&value),
                            None,
                        ),
                        false,
                    )
                }
                Some(JSXAttributeValue::ExpressionContainer(container)) => (
                    self.transform_component_expression(&container.expression),
                    self.component_prop_requires_getter(&name, container),
                ),
                Some(JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_)) => {
                    return Err(Error::from_reason(
                        "Universal component JSX attribute values are not implemented in the AST-native milestone yet",
                    ));
                }
            };
            running_props.push(component_property(
                self,
                attr.span,
                &name,
                value,
                needs_getter,
            ));
        }
        if let Some((children, needs_getter)) =
            self.component_children_expression(&element.children)?
        {
            running_props.push(component_property(
                self,
                element.span,
                "children",
                children,
                needs_getter,
            ));
        }
        flush_component_props(self, &mut running_props, &mut prop_objects, element.span);
        let props = component_props_expression(self, element.span, prop_objects, force_merge_props);
        Ok(self.call_identifier(
            element.span,
            &self.helper_local("_$createComponent"),
            vec![
                expression_to_argument(component),
                expression_to_argument(props),
            ],
        ))
    }

    fn lower_attribute(
        &mut self,
        attr: &JSXAttributeItem<'a>,
        element_id: &str,
        setup: &mut std::vec::Vec<Statement<'a>>,
    ) -> Result<()> {
        let JSXAttributeItem::Attribute(attr) = attr else {
            return Err(Error::from_reason(
                "Universal spread attributes are not implemented in the AST-native milestone yet",
            ));
        };
        let name = match &attr.name {
            oxc_ast::ast::JSXAttributeName::Identifier(name) => name.name.to_string(),
            oxc_ast::ast::JSXAttributeName::NamespacedName(name) => {
                format!("{}:{}", name.namespace.name, name.name.name)
            }
        };
        let value = match &attr.value {
            None => self.ast().expression_boolean_literal(attr.span, true),
            Some(JSXAttributeValue::StringLiteral(value)) => self.ast().expression_string_literal(
                value.span,
                self.ast().atom(&decode_html_entities(&value.value)),
                None,
            ),
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                if let Some(value) = static_jsx_expression_value(&container.expression) {
                    self.ast().expression_string_literal(
                        container.span,
                        self.ast().atom(&value),
                        None,
                    )
                } else {
                    crate::dom::element::jsx_expression_to_expression(
                        &container.expression,
                        self.allocator,
                    )
                }
            }
            Some(JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_)) => {
                return Err(Error::from_reason(
                    "Universal JSX attribute values are not implemented in the AST-native milestone yet",
                ));
            }
        };
        self.uses_set_prop = true;
        setup.push(self.expression_statement(
            attr.span,
            self.call_identifier(
                attr.span,
                &self.helper_local("_$setProp"),
                vec![
                    self.identifier_arg(attr.span, element_id),
                    self.string_arg(attr.span, &name),
                    expression_to_argument(value),
                ],
            ),
        ));
        Ok(())
    }

    fn init_attribute_property(
        &mut self,
        attr: &JSXAttributeItem<'a>,
    ) -> Result<Option<ObjectPropertyKind<'a>>> {
        let JSXAttributeItem::Attribute(attr) = attr else {
            return Err(Error::from_reason(
                "Universal spread attributes are not implemented in the AST-native milestone yet",
            ));
        };
        let name = match &attr.name {
            oxc_ast::ast::JSXAttributeName::Identifier(name) => name.name.to_string(),
            oxc_ast::ast::JSXAttributeName::NamespacedName(name) => {
                format!("{}:{}", name.namespace.name, name.name.name)
            }
        };
        if name == "ref" || name == "children" {
            return Ok(None);
        }
        let value = match &attr.value {
            None => self.ast().expression_boolean_literal(attr.span, true),
            Some(JSXAttributeValue::StringLiteral(value)) => self.ast().expression_string_literal(
                value.span,
                self.ast().atom(&decode_html_entities(&value.value)),
                None,
            ),
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                let expression = crate::dom::element::jsx_expression_to_expression(
                    &container.expression,
                    self.allocator,
                );
                if !source_from_span(container.span, self.source).contains(&self.static_marker)
                    && expression_is_dynamic(&expression)
                {
                    return Ok(None);
                }
                expression
            }
            Some(JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_)) => {
                return Err(Error::from_reason(
                    "Universal JSX attribute values are not implemented in the AST-native milestone yet",
                ));
            }
        };
        Ok(Some(self.object_property(attr.span, &name, value)))
    }

    fn component_children_expression(
        &mut self,
        children: &[JSXChild<'a>],
    ) -> Result<Option<(Expression<'a>, bool)>> {
        let mut values = std::vec::Vec::new();
        let mut needs_getter = false;
        for child in children {
            match child {
                JSXChild::Text(text) => {
                    let value = trim_jsx_text(&text.value);
                    if !value.is_empty() {
                        values.push(self.ast().expression_string_literal(
                            text.span,
                            self.ast().atom(&value),
                            None,
                        ));
                    }
                }
                JSXChild::ExpressionContainer(container) => {
                    if matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                        continue;
                    }
                    needs_getter = true;
                    values.push(self.transform_component_expression(&container.expression));
                }
                JSXChild::Element(element) => {
                    needs_getter = true;
                    let (value, setup) = self.lower_element(element)?;
                    values.push(self.setup_iife(element.span, setup, value));
                }
                JSXChild::Spread(spread) => {
                    needs_getter = true;
                    values.push(spread.expression.clone_in(self.allocator));
                }
                JSXChild::Fragment(fragment) => {
                    needs_getter = true;
                    values.push(self.lower_fragment(fragment)?);
                }
            }
        }
        Ok(match values.len() {
            0 => None,
            1 => Some((
                values
                    .pop()
                    .expect("single universal component child exists"),
                needs_getter,
            )),
            _ => Some((
                self.ast().expression_array(
                    children
                        .first()
                        .map_or_else(|| Span::new(0, 0), oxc_ast::ast::JSXChild::span),
                    self.ast()
                        .vec_from_iter(values.into_iter().map(expression_to_array_element)),
                ),
                true,
            )),
        })
    }

    fn transform_component_expression(&mut self, expression: &JSXExpression<'a>) -> Expression<'a> {
        let mut expression =
            crate::dom::element::jsx_expression_to_expression(expression, self.allocator);
        self.visit_expression(&mut expression);
        expression
    }

    pub(crate) fn lower_fragment(&mut self, fragment: &JSXFragment<'a>) -> Result<Expression<'a>> {
        let mut values = std::vec::Vec::new();
        for child in &fragment.children {
            match child {
                JSXChild::Text(text) => {
                    let value = trim_jsx_text(&text.value);
                    if !value.is_empty() {
                        values.push(self.ast().expression_string_literal(
                            text.span,
                            self.ast().atom(&value),
                            None,
                        ));
                    }
                }
                JSXChild::ExpressionContainer(container) => {
                    if matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                        continue;
                    }
                    values.push(self.transform_component_expression(&container.expression));
                }
                JSXChild::Element(element) => {
                    let (value, setup) = self.lower_element(element)?;
                    values.push(self.setup_iife(element.span, setup, value));
                }
                JSXChild::Fragment(fragment) => values.push(self.lower_fragment(fragment)?),
                JSXChild::Spread(spread) => values.push(spread.expression.clone_in(self.allocator)),
            }
        }
        Ok(match values.len() {
            0 => self.ast().expression_null_literal(fragment.span),
            1 => values
                .pop()
                .expect("single universal fragment child exists"),
            _ => self.ast().expression_array(
                fragment.span,
                self.ast()
                    .vec_from_iter(values.into_iter().map(expression_to_array_element)),
            ),
        })
    }

    fn component_prop_requires_getter(
        &self,
        name: &str,
        container: &oxc_ast::ast::JSXExpressionContainer<'_>,
    ) -> bool {
        if name == "ref" {
            return false;
        }
        if crate::shared::utils::source_from_span(container.span, self.source)
            .contains(&self.static_marker)
        {
            return false;
        }
        matches!(
            container.expression,
            JSXExpression::StaticMemberExpression(_)
                | JSXExpression::ComputedMemberExpression(_)
                | JSXExpression::JSXElement(_)
        )
    }

    fn lower_child(
        &mut self,
        child: &JSXChild<'a>,
        parent_id: &str,
        setup: &mut std::vec::Vec<Statement<'a>>,
    ) -> Result<()> {
        match child {
            JSXChild::Text(text) => {
                let value = trim_jsx_text(&text.value);
                if !value.is_empty() {
                    let child = self.create_text_node(text.span, &value);
                    self.push_insert_node(text.span, parent_id, child, setup);
                }
            }
            JSXChild::ExpressionContainer(container) => {
                if matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                    return Ok(());
                }
                if let Some(value) = static_jsx_expression_value(&container.expression) {
                    let child = self.create_text_node(container.span, &value);
                    self.push_insert_node(container.span, parent_id, child, setup);
                } else {
                    self.uses_insert = true;
                    let value = crate::dom::element::jsx_expression_to_expression(
                        &container.expression,
                        self.allocator,
                    );
                    setup.push(self.expression_statement(
                        container.span,
                        self.call_identifier(
                            container.span,
                            &self.helper_local("_$insert"),
                            vec![
                                self.identifier_arg(container.span, parent_id),
                                expression_to_argument(value),
                            ],
                        ),
                    ));
                };
            }
            JSXChild::Element(element) => {
                let (child, child_setup) = self.lower_element(element)?;
                setup.extend(child_setup);
                self.push_insert_node(element.span, parent_id, child, setup);
            }
            JSXChild::Spread(spread) => {
                self.uses_insert = true;
                setup.push(self.expression_statement(
                    spread.span,
                    self.call_identifier(
                        spread.span,
                        &self.helper_local("_$insert"),
                        vec![
                            self.identifier_arg(spread.span, parent_id),
                            expression_to_argument(spread.expression.clone_in(self.allocator)),
                        ],
                    ),
                ));
            }
            JSXChild::Fragment(fragment) => {
                self.uses_insert = true;
                let value = self.lower_fragment(fragment)?;
                setup.push(self.expression_statement(
                    fragment.span,
                    self.call_identifier(
                        fragment.span,
                        &self.helper_local("_$insert"),
                        vec![
                            self.identifier_arg(fragment.span, parent_id),
                            expression_to_argument(value),
                        ],
                    ),
                ));
            }
        }
        Ok(())
    }

    fn create_text_node(&mut self, span: Span, value: &str) -> Expression<'a> {
        self.uses_create_text_node = true;
        self.call_identifier(
            span,
            &self.helper_local("_$createTextNode"),
            vec![self.string_arg(span, value)],
        )
    }

    fn push_insert_node(
        &mut self,
        span: Span,
        parent_id: &str,
        child: Expression<'a>,
        setup: &mut std::vec::Vec<Statement<'a>>,
    ) {
        self.uses_insert_node = true;
        setup.push(self.expression_statement(
            span,
            self.call_identifier(
                span,
                &self.helper_local("_$insertNode"),
                vec![
                    self.identifier_arg(span, parent_id),
                    expression_to_argument(child),
                ],
            ),
        ));
    }

    pub(crate) fn process_statements(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>) {
        let mut body = ArenaVec::new_in(self.allocator);
        for mut statement in statements.drain(..) {
            if self.error.is_some() {
                body.push(statement);
                continue;
            }
            if let Some(setup) = self.lower_variable_jsx_initializer(&mut statement) {
                body.extend(setup);
                body.push(statement);
                continue;
            }
            self.visit_statement(&mut statement);
            body.push(statement);
        }
        *statements = body;
    }

    fn lower_variable_jsx_initializer(
        &mut self,
        statement: &mut Statement<'a>,
    ) -> Option<std::vec::Vec<Statement<'a>>> {
        let Statement::VariableDeclaration(declaration) = statement else {
            return None;
        };
        if declaration.declarations.len() != 1 {
            return None;
        }
        let init = declaration.declarations[0].init.take()?;
        match init {
            Expression::JSXElement(element) => match self.lower_element(&element) {
                Ok((replacement, setup)) => {
                    declaration.declarations[0].init = Some(replacement);
                    Some(setup)
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    declaration.declarations[0].init = Some(Expression::JSXElement(element));
                    Some(std::vec::Vec::new())
                }
            },
            Expression::JSXFragment(fragment) => match self.lower_fragment(&fragment) {
                Ok(replacement) => {
                    declaration.declarations[0].init = Some(replacement);
                    Some(std::vec::Vec::new())
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    declaration.declarations[0].init = Some(Expression::JSXFragment(fragment));
                    Some(std::vec::Vec::new())
                }
            },
            Expression::ParenthesizedExpression(parenthesized) => {
                match parenthesized.unbox().expression {
                    Expression::JSXElement(element) => match self.lower_element(&element) {
                        Ok((replacement, setup)) => {
                            declaration.declarations[0].init = Some(replacement);
                            Some(setup)
                        }
                        Err(error) => {
                            self.error = Some(error.to_string());
                            declaration.declarations[0].init =
                                Some(Expression::JSXElement(element));
                            Some(std::vec::Vec::new())
                        }
                    },
                    Expression::JSXFragment(fragment) => match self.lower_fragment(&fragment) {
                        Ok(replacement) => {
                            declaration.declarations[0].init = Some(replacement);
                            Some(std::vec::Vec::new())
                        }
                        Err(error) => {
                            self.error = Some(error.to_string());
                            declaration.declarations[0].init =
                                Some(Expression::JSXFragment(fragment));
                            Some(std::vec::Vec::new())
                        }
                    },
                    expression => {
                        declaration.declarations[0].init = Some(expression);
                        None
                    }
                }
            }
            init => {
                declaration.declarations[0].init = Some(init);
                None
            }
        }
    }
}

impl<'a> ComponentPropContext<'a> for AstUniversalTransform<'a, '_> {
    fn allocator(&self) -> &'a Allocator {
        self.allocator
    }

    fn ast(&self) -> AstBuilder<'a> {
        self.ast()
    }

    fn mark_merge_props(&mut self) {
        self.uses_merge_props = true;
    }

    fn call_identifier(
        &self,
        span: Span,
        callee: &str,
        args: std::vec::Vec<Expression<'a>>,
    ) -> Expression<'a> {
        let callee = if callee.starts_with("_$") {
            self.helper_local(match callee {
                "_$mergeProps" => "_$mergeProps",
                "_$createComponent" => "_$createComponent",
                "_$spread" => "_$spread",
                "_$insert" => "_$insert",
                "_$insertNode" => "_$insertNode",
                "_$setProp" => "_$setProp",
                "_$createElement" => "_$createElement",
                "_$createTextNode" => "_$createTextNode",
                _ => callee,
            })
        } else {
            callee.to_string()
        };
        self.call_expression(
            span,
            self.ast()
                .expression_identifier(span, self.ast().ident(&callee)),
            args,
        )
    }

    fn arrow_return_expression(&self, span: Span, value: Expression<'a>) -> Expression<'a> {
        arrow_return_expression(self.allocator, span, value)
    }

    fn object_property(
        &self,
        span: Span,
        name: &str,
        value: Expression<'a>,
    ) -> ObjectPropertyKind<'a> {
        self.object_property(span, name, value)
    }

    fn object_getter_property(
        &self,
        span: Span,
        name: &str,
        value: Expression<'a>,
    ) -> ObjectPropertyKind<'a> {
        self.object_getter_property(span, name, value)
    }
}

fn expression_is_dynamic(value: &Expression<'_>) -> bool {
    match value {
        Expression::CallExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::PrivateFieldExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::ChainExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::PrivateInExpression(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::YieldExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_) => true,
        Expression::ObjectExpression(object) => object.properties.iter().any(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return true;
            };
            property.computed || expression_is_dynamic(&property.value)
        }),
        Expression::ArrayExpression(array) => array.elements.iter().any(array_element_is_dynamic),
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_is_dynamic(&parenthesized.expression)
        }
        Expression::TSAsExpression(expression) => expression_is_dynamic(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => {
            expression_is_dynamic(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => expression_is_dynamic(&expression.expression),
        Expression::TSNonNullExpression(expression) => {
            expression_is_dynamic(&expression.expression)
        }
        Expression::TSInstantiationExpression(expression) => {
            expression_is_dynamic(&expression.expression)
        }
        _ => false,
    }
}

fn array_element_is_dynamic(element: &ArrayExpressionElement<'_>) -> bool {
    match element {
        ArrayExpressionElement::CallExpression(_)
        | ArrayExpressionElement::StaticMemberExpression(_)
        | ArrayExpressionElement::PrivateFieldExpression(_)
        | ArrayExpressionElement::ComputedMemberExpression(_)
        | ArrayExpressionElement::ChainExpression(_)
        | ArrayExpressionElement::ConditionalExpression(_)
        | ArrayExpressionElement::LogicalExpression(_)
        | ArrayExpressionElement::PrivateInExpression(_)
        | ArrayExpressionElement::TaggedTemplateExpression(_)
        | ArrayExpressionElement::UpdateExpression(_)
        | ArrayExpressionElement::YieldExpression(_)
        | ArrayExpressionElement::AwaitExpression(_)
        | ArrayExpressionElement::JSXElement(_)
        | ArrayExpressionElement::JSXFragment(_)
        | ArrayExpressionElement::SpreadElement(_) => true,
        ArrayExpressionElement::ObjectExpression(object) => {
            object.properties.iter().any(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return true;
                };
                property.computed || expression_is_dynamic(&property.value)
            })
        }
        ArrayExpressionElement::ArrayExpression(array) => {
            array.elements.iter().any(array_element_is_dynamic)
        }
        _ => false,
    }
}
