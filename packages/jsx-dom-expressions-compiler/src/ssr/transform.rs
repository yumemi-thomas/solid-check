use napi::bindgen_prelude::*;
use oxc_allocator::{Allocator, CloneIn, Vec as ArenaVec};
use oxc_ast::{
    ast::{
        Argument, Expression, FormalParameterKind, JSXAttributeItem, JSXAttributeValue, JSXChild,
        JSXElement, JSXElementName, JSXExpression, JSXFragment, JSXMemberExpression,
        JSXMemberExpressionObject, ObjectPropertyKind, Program, Statement,
    },
    AstBuilder, NONE,
};
use oxc_ast_visit::{walk_mut, VisitMut};
use oxc_span::{GetSpan, Span};

use crate::dom::element::jsx_expression_to_expression;
use crate::shared::array::expression_to_array_element;
use crate::shared::ast::{
    arrow_return_expression, expression_to_argument, import_named, object_getter_property,
    object_property, variable_statement,
};
use crate::shared::component_props::{
    component_property, component_props_expression, component_spread_expression,
    flush_component_props, ComponentPropContext,
};
use crate::shared::constants::namespaces;
use crate::shared::utils::{
    decode_html_entities, element_name, escape_html_text, escape_html_text_expression,
    is_component_name, is_identifier_key, is_void_element, static_jsx_expression_value,
    trim_jsx_text,
};

use super::template::SsrTemplate;

pub(crate) struct AstSsrTransform<'a, 'source> {
    allocator: &'a Allocator,
    source: &'source str,
    module_name: &'source str,
    built_ins: std::vec::Vec<String>,
    built_in_imports: std::vec::Vec<String>,
    hydratable: bool,
    static_marker: String,
    uses_ssr: bool,
    uses_ssr_hydration_key: bool,
    uses_escape: bool,
    uses_ssr_element: bool,
    uses_merge_props: bool,
    pending_this_capture: Option<String>,
    current_this_capture: Option<String>,
    this_index: usize,
    statement_depth: usize,
    pub(crate) error: Option<String>,
}

impl<'a, 'source> AstSsrTransform<'a, 'source> {
    pub(crate) fn new(
        allocator: &'a Allocator,
        source: &'source str,
        module_name: &'source str,
        hydratable: bool,
        static_marker: String,
        built_ins: std::vec::Vec<String>,
    ) -> Self {
        Self {
            allocator,
            source,
            module_name,
            built_ins,
            built_in_imports: std::vec::Vec::new(),
            hydratable,
            static_marker,
            uses_ssr: false,
            uses_ssr_hydration_key: false,
            uses_escape: false,
            uses_ssr_element: false,
            uses_merge_props: false,
            pending_this_capture: None,
            current_this_capture: None,
            this_index: 0,
            statement_depth: 0,
            error: None,
        }
    }

    pub(crate) fn prepend_helpers(&self, program: &mut Program<'a>) {
        if !self.uses_ssr
            && !self.uses_ssr_hydration_key
            && !self.uses_escape
            && !self.uses_ssr_element
            && !self.uses_merge_props
        {
            return;
        }
        let mut statements = std::vec::Vec::new();
        if self.uses_escape {
            statements.push(self.import_named("escape", "_$escape"));
        }
        if self.uses_ssr {
            statements.push(self.import_named("ssr", "_$ssr"));
        }
        if self.uses_ssr_hydration_key {
            statements.push(self.import_named("ssrHydrationKey", "_$ssrHydrationKey"));
        }
        if self.uses_ssr_element {
            statements.push(self.import_named("ssrElement", "_$ssrElement"));
        }
        if self.uses_merge_props {
            statements.push(self.import_named("mergeProps", "_$mergeProps"));
        }
        for built_in in &self.built_in_imports {
            statements.push(self.import_named(built_in, &format!("_${built_in}")));
        }
        statements.extend(program.body.drain(..));
        let mut body = ArenaVec::new_in(self.allocator);
        body.extend(statements);
        program.body = body;
    }

    pub(crate) fn lower_element(&mut self, element: &JSXElement<'a>) -> Result<Expression<'a>> {
        if is_component_name(&element.opening_element.name) {
            return self.lower_component(element);
        }
        if element
            .opening_element
            .attributes
            .iter()
            .any(|attr| matches!(attr, JSXAttributeItem::SpreadAttribute(_)))
        {
            return self.lower_spread_element(element);
        }
        let template = self.ssr_template(element, self.hydratable)?;
        self.uses_ssr = true;
        Ok(self.ssr_call(element.span, template, self.hydratable))
    }

    fn lower_component(&mut self, element: &JSXElement<'a>) -> Result<Expression<'a>> {
        let component = self.component_callee_expression(&element.opening_element.name)?;
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
                        "SSR component namespace attributes are not implemented in the AST-native milestone yet",
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
                        "SSR component JSX attribute values are not implemented in the AST-native milestone yet",
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
        Ok(self.call_expression(element.span, component, vec![props]))
    }

    fn component_children_expression(
        &mut self,
        children: &[JSXChild<'a>],
    ) -> Result<Option<(Expression<'a>, bool)>> {
        let value = self.ssr_children_expression(children)?;
        if matches!(&value, Expression::Identifier(identifier) if identifier.name == "undefined") {
            return Ok(None);
        }
        let needs_getter = children.iter().any(|child| {
            matches!(
                child,
                JSXChild::Element(_)
                    | JSXChild::Fragment(_)
                    | JSXChild::ExpressionContainer(_)
                    | JSXChild::Spread(_)
            )
        });
        Ok(Some((value, needs_getter)))
    }

    fn transform_component_expression(&mut self, expression: &JSXExpression<'a>) -> Expression<'a> {
        let mut expression = jsx_expression_to_expression(expression, self.allocator);
        self.replace_this_expression(&mut expression);
        if !matches!(
            expression,
            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
        ) || crate::shared::utils::source_from_span(expression.span(), self.source).contains('<')
        {
            self.visit_expression(&mut expression);
        }
        expression
    }

    fn replace_this_expression(&mut self, expression: &mut Expression<'a>) {
        struct ThisReplacer<'ctx, 'a, 'source> {
            ctx: &'ctx mut AstSsrTransform<'a, 'source>,
        }

        impl<'a> VisitMut<'a> for ThisReplacer<'_, 'a, '_> {
            fn visit_expression(&mut self, expression: &mut Expression<'a>) {
                if let Expression::ThisExpression(this) = expression {
                    *expression = self.ctx.capture_this_expression(this.span);
                    return;
                }
                walk_mut::walk_expression(self, expression);
            }
        }

        ThisReplacer { ctx: self }.visit_expression(expression);
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

    fn lower_spread_element(&mut self, element: &JSXElement<'a>) -> Result<Expression<'a>> {
        self.uses_ssr_element = true;
        let tag_name = element_name(&element.opening_element.name)?;
        let props = self.spread_props(&element.opening_element.attributes)?;
        let children = self.ssr_children_expression(&element.children)?;
        let args = self.ast().vec_from_array([
            Argument::StringLiteral(self.ast().alloc_string_literal(
                element.span,
                self.ast().atom(&tag_name),
                None,
            )),
            expression_to_argument(props),
            expression_to_argument(children),
            Argument::BooleanLiteral(self.ast().alloc_boolean_literal(element.span, false)),
        ]);
        Ok(self.ast().expression_call(
            element.span,
            self.ast()
                .expression_identifier(element.span, self.ast().ident("_$ssrElement")),
            NONE,
            args,
            false,
        ))
    }

    fn spread_props(&mut self, attributes: &[JSXAttributeItem<'a>]) -> Result<Expression<'a>> {
        let mut prop_objects = std::vec::Vec::new();
        let mut running_props = std::vec::Vec::new();
        for attr in attributes {
            match attr {
                JSXAttributeItem::SpreadAttribute(spread) => {
                    flush_props(self, spread.span, &mut running_props, &mut prop_objects);
                    prop_objects.push(spread.argument.clone_in(self.allocator));
                }
                JSXAttributeItem::Attribute(attr) => {
                    if let Some(property) = self.spread_prop_property(attr)? {
                        running_props.push(property);
                    }
                }
            }
        }
        flush_props(self, Span::new(0, 0), &mut running_props, &mut prop_objects);
        Ok(match prop_objects.len() {
            0 => self
                .ast()
                .expression_identifier(Span::new(0, 0), self.ast().ident("undefined")),
            1 => prop_objects
                .pop()
                .expect("single SSR spread prop object exists"),
            _ => {
                self.uses_merge_props = true;
                self.call_expression(
                    Span::new(0, 0),
                    self.ast()
                        .expression_identifier(Span::new(0, 0), self.ast().ident("_$mergeProps")),
                    prop_objects,
                )
            }
        })
    }

    fn spread_prop_property(
        &mut self,
        attr: &oxc_ast::ast::JSXAttribute<'a>,
    ) -> Result<Option<ObjectPropertyKind<'a>>> {
        let name = match &attr.name {
            oxc_ast::ast::JSXAttributeName::Identifier(name) => name.name.to_string(),
            oxc_ast::ast::JSXAttributeName::NamespacedName(name)
                if name.namespace.name == "prop" =>
            {
                return Ok(None);
            }
            oxc_ast::ast::JSXAttributeName::NamespacedName(_) => {
                return Err(Error::from_reason(
                    "SSR namespaced attributes are not implemented in the AST-native milestone yet",
                ));
            }
        };
        let value = match &attr.value {
            None => self.ast().expression_boolean_literal(attr.span, true),
            Some(JSXAttributeValue::StringLiteral(value)) => {
                self.ast()
                    .expression_string_literal(attr.span, self.ast().atom(&value.value), None)
            }
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                jsx_expression_to_expression(&container.expression, self.allocator)
            }
            Some(JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_)) => {
                return Err(Error::from_reason(
                    "SSR JSX attribute values are not implemented in the AST-native milestone yet",
                ));
            }
        };
        Ok(Some(self.object_property(attr.span, &name, value)))
    }

    fn ssr_children_expression(&mut self, children: &[JSXChild<'a>]) -> Result<Expression<'a>> {
        let mut values = std::vec::Vec::new();
        for child in children {
            match child {
                JSXChild::Text(text) => {
                    let span = text.span;
                    let text = trim_jsx_text(&text.value);
                    if !text.is_empty() {
                        values.push(self.ast().expression_string_literal(
                            span,
                            self.ast().atom(&text),
                            None,
                        ));
                    }
                }
                JSXChild::Element(element) => values.push(self.lower_element(element)?),
                JSXChild::ExpressionContainer(container) => {
                    if matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                        continue;
                    }
                    values.push(self.transform_component_expression(&container.expression));
                }
                JSXChild::Fragment(fragment) => values.push(self.lower_fragment(fragment)?),
                JSXChild::Spread(spread) => values.push(spread.expression.clone_in(self.allocator)),
            }
        }
        Ok(match values.len() {
            0 => self
                .ast()
                .expression_identifier(Span::new(0, 0), self.ast().ident("undefined")),
            1 => values.pop().expect("single SSR child value exists"),
            _ => self.ast().expression_array(
                Span::new(0, 0),
                self.ast().vec_from_iter(
                    values
                        .into_iter()
                        .map(crate::shared::array::expression_to_array_element),
                ),
            ),
        })
    }

    pub(crate) fn lower_fragment(&mut self, fragment: &JSXFragment<'a>) -> Result<Expression<'a>> {
        let mut values = std::vec::Vec::new();
        for child in &fragment.children {
            match child {
                JSXChild::Text(text) => {
                    let span = text.span;
                    let text = trim_jsx_text(&text.value);
                    if !text.is_empty() {
                        values.push(self.ast().expression_string_literal(
                            span,
                            self.ast().atom(&text),
                            None,
                        ));
                    }
                }
                JSXChild::Element(element) => values.push(self.lower_element(element)?),
                JSXChild::ExpressionContainer(container) => {
                    if matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                        continue;
                    }
                    if let Some(value) = static_jsx_expression_value(&container.expression) {
                        values.push(self.ast().expression_string_literal(
                            container.span,
                            self.ast().atom(&value),
                            None,
                        ));
                    } else {
                        values.push(self.transform_component_expression(&container.expression));
                    }
                }
                JSXChild::Fragment(fragment) => values.push(self.lower_fragment(fragment)?),
                JSXChild::Spread(spread) => values.push(spread.expression.clone_in(self.allocator)),
            }
        }

        Ok(match values.len() {
            0 => self.ast().expression_null_literal(fragment.span),
            1 => values
                .pop()
                .expect("SSR fragment value exists after length check"),
            _ => self.ast().expression_array(
                fragment.span,
                self.ast()
                    .vec_from_iter(values.into_iter().map(expression_to_array_element)),
            ),
        })
    }

    fn ssr_template(
        &mut self,
        element: &JSXElement<'a>,
        hydratable_root: bool,
    ) -> Result<SsrTemplate<'a>> {
        let tag_name = element_name(&element.opening_element.name)?;
        let mut template = SsrTemplate::new(format!("<{tag_name}"));
        if hydratable_root {
            template.push_expr(self.ssr_hydration_key_call(element.span));
        }
        for attr in &element.opening_element.attributes {
            self.append_static_attribute(attr, &mut template)?;
        }
        template.current_mut().push('>');
        if !is_void_element(&tag_name) {
            let mut ordered_insert = false;
            for child in &element.children {
                self.append_static_child(child, &mut template, &mut ordered_insert)?;
            }
            template.current_mut().push_str(&format!("</{tag_name}>"));
        }
        Ok(template)
    }

    fn append_static_attribute(
        &mut self,
        attr: &JSXAttributeItem<'a>,
        template: &mut SsrTemplate<'a>,
    ) -> Result<()> {
        let JSXAttributeItem::Attribute(attr) = attr else {
            return Err(Error::from_reason(
                "SSR spread attributes are not implemented in the AST-native milestone yet",
            ));
        };
        let name = match &attr.name {
            oxc_ast::ast::JSXAttributeName::Identifier(name) => name.name.to_string(),
            oxc_ast::ast::JSXAttributeName::NamespacedName(name)
                if name.namespace.name == "prop" =>
            {
                return Ok(());
            }
            oxc_ast::ast::JSXAttributeName::NamespacedName(name)
                if namespaces(&name.namespace.name).is_some() =>
            {
                format!("{}:{}", name.namespace.name, name.name.name)
            }
            oxc_ast::ast::JSXAttributeName::NamespacedName(_) => {
                return Err(Error::from_reason(
                    "SSR namespaced attributes are not implemented in the AST-native milestone yet",
                ));
            }
        };
        match &attr.value {
            None => template.current_mut().push_str(&format!(" {}", name)),
            Some(JSXAttributeValue::StringLiteral(value)) => {
                template.current_mut().push_str(&format!(
                    " {}=\"{}\"",
                    name,
                    escape_html_attribute(&value.value)
                ));
            }
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                if let Some(value) = static_jsx_expression_value(&container.expression) {
                    template.current_mut().push_str(&format!(
                        " {}=\"{}\"",
                        name,
                        escape_html_attribute(&value)
                    ));
                } else {
                    template.current_mut().push_str(&format!(" {}=\"", name));
                    let value = jsx_expression_to_expression(&container.expression, self.allocator);
                    template.push_expr(self.escape_attribute_expression(container.span, value));
                    template.current_mut().push('"');
                };
            }
            Some(JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_)) => {
                return Err(Error::from_reason(
                    "SSR JSX attribute values are not implemented in the AST-native milestone yet",
                ));
            }
        }
        Ok(())
    }

    fn append_static_child(
        &mut self,
        child: &JSXChild<'a>,
        template: &mut SsrTemplate<'a>,
        ordered_insert: &mut bool,
    ) -> Result<()> {
        let allocates_ids = self.hydratable && child_slot_allocates_ids(child);
        match child {
            JSXChild::Text(text) => {
                let text = trim_jsx_text(&text.value);
                if !text.is_empty() {
                    template.current_mut().push_str(&escape_html_text(&text));
                }
            }
            JSXChild::Element(element) if is_component_name(&element.opening_element.name) => {
                let value = self.lower_component(element)?;
                self.push_child_expr(template, value, allocates_ids, ordered_insert, element.span);
            }
            JSXChild::Element(element) => {
                if let Ok(child) = self.ssr_template(element, false) {
                    template.append_template(child);
                } else {
                    let value = self.lower_spread_element(element)?;
                    self.push_child_expr(
                        template,
                        value,
                        allocates_ids,
                        ordered_insert,
                        element.span,
                    );
                }
            }
            JSXChild::ExpressionContainer(container) => {
                if matches!(container.expression, JSXExpression::EmptyExpression(_)) {
                    return Ok(());
                }
                if let Some(value) = static_jsx_expression_value(&container.expression) {
                    template
                        .current_mut()
                        .push_str(&escape_html_text_expression(&value));
                } else {
                    let mut value =
                        jsx_expression_to_expression(&container.expression, self.allocator);
                    self.visit_expression(&mut value);
                    let value = self.escape_expression(container.span, value);
                    let value = if self.hydratable && allocates_ids {
                        self.arrow_return_expression(container.span, value)
                    } else {
                        value
                    };
                    self.push_child_expr(
                        template,
                        value,
                        allocates_ids,
                        ordered_insert,
                        container.span,
                    );
                }
            }
            JSXChild::Spread(spread) => {
                let value =
                    self.escape_expression(spread.span, spread.expression.clone_in(self.allocator));
                let value = if self.hydratable && allocates_ids {
                    self.arrow_return_expression(spread.span, value)
                } else {
                    value
                };
                self.push_child_expr(template, value, allocates_ids, ordered_insert, spread.span);
            }
            JSXChild::Fragment(_) => {
                return Err(Error::from_reason(
                    "SSR fragments are not implemented in the AST-native milestone yet",
                ));
            }
        }
        Ok(())
    }

    fn push_child_expr(
        &mut self,
        template: &mut SsrTemplate<'a>,
        value: Expression<'a>,
        allocates_ids: bool,
        ordered_insert: &mut bool,
        span: Span,
    ) {
        let value = if self.hydratable
            && *ordered_insert
            && allocates_ids
            && !is_deferred_child_slot_expression(&value)
        {
            self.arrow_return_expression(span, value)
        } else {
            value
        };
        if self.hydratable && allocates_ids && is_deferred_child_slot_expression(&value) {
            *ordered_insert = true;
        }
        template.push_expr(value);
    }

    fn ast(&self) -> AstBuilder<'a> {
        AstBuilder::new(self.allocator)
    }

    fn ssr_call(
        &self,
        span: Span,
        mut template: SsrTemplate<'a>,
        hydratable: bool,
    ) -> Expression<'a> {
        let template_arg = if template.values.is_empty() {
            Argument::StringLiteral(
                self.ast().alloc_string_literal(
                    span,
                    self.ast()
                        .atom(template.parts.first().map_or("", String::as_str)),
                    None,
                ),
            )
        } else {
            Argument::ArrayExpression(
                self.ast().alloc_array_expression(
                    span,
                    self.ast()
                        .vec_from_iter(template.parts.into_iter().map(|part| {
                            oxc_ast::ast::ArrayExpressionElement::StringLiteral(
                                self.ast()
                                    .alloc_string_literal(span, self.ast().atom(&part), None),
                            )
                        })),
                ),
            )
        };
        let args = if hydratable {
            let hydration_key = if !template.values.is_empty() {
                template.values.remove(0)
            } else {
                self.ast()
                    .expression_identifier(span, self.ast().ident("undefined"))
            };
            self.ast().vec_from_iter(
                std::iter::once(template_arg)
                    .chain(std::iter::once(expression_to_argument(hydration_key)))
                    .chain(template.values.into_iter().map(expression_to_argument)),
            )
        } else {
            self.ast().vec_from_iter(
                std::iter::once(template_arg)
                    .chain(template.values.into_iter().map(expression_to_argument)),
            )
        };
        self.ast().expression_call(
            span,
            self.ast()
                .expression_identifier(span, self.ast().ident("_$ssr")),
            NONE,
            args,
            false,
        )
    }

    fn ssr_hydration_key_call(&mut self, span: Span) -> Expression<'a> {
        self.uses_ssr_hydration_key = true;
        self.ast().expression_call(
            span,
            self.ast()
                .expression_identifier(span, self.ast().ident("_$ssrHydrationKey")),
            NONE,
            self.ast().vec(),
            false,
        )
    }

    fn escape_expression(&mut self, span: Span, value: Expression<'a>) -> Expression<'a> {
        self.uses_escape = true;
        self.ast().expression_call(
            span,
            self.ast()
                .expression_identifier(span, self.ast().ident("_$escape")),
            NONE,
            self.ast().vec1(expression_to_argument(value)),
            false,
        )
    }

    fn escape_attribute_expression(&mut self, span: Span, value: Expression<'a>) -> Expression<'a> {
        self.uses_escape = true;
        self.ast().expression_call(
            span,
            self.ast()
                .expression_identifier(span, self.ast().ident("_$escape")),
            NONE,
            self.ast().vec_from_array([
                expression_to_argument(value),
                Argument::BooleanLiteral(self.ast().alloc_boolean_literal(span, true)),
            ]),
            false,
        )
    }

    fn call_expression(
        &self,
        span: Span,
        callee: Expression<'a>,
        args: std::vec::Vec<Expression<'a>>,
    ) -> Expression<'a> {
        self.ast().expression_call(
            span,
            callee,
            NONE,
            self.ast()
                .vec_from_iter(args.into_iter().map(expression_to_argument)),
            false,
        )
    }

    fn object_property(
        &self,
        span: Span,
        name: &str,
        value: Expression<'a>,
    ) -> ObjectPropertyKind<'a> {
        object_property(self.allocator, span, name, value)
    }

    fn object_getter_property(
        &self,
        span: Span,
        name: &str,
        value: Expression<'a>,
    ) -> ObjectPropertyKind<'a> {
        object_getter_property(self.allocator, span, name, value)
    }

    fn component_callee_expression(&mut self, name: &JSXElementName<'a>) -> Result<Expression<'a>> {
        match name {
            JSXElementName::Identifier(identifier) => {
                Ok(self.component_identifier_expression(&identifier.name))
            }
            JSXElementName::IdentifierReference(identifier) => {
                Ok(self.component_identifier_expression(&identifier.name))
            }
            JSXElementName::MemberExpression(member) => self.component_member_expression(member),
            JSXElementName::ThisExpression(this) => Ok(self.capture_this_expression(this.span)),
            JSXElementName::NamespacedName(_) => Err(Error::from_reason(
                "SSR namespaced component callees are not implemented in the AST-native milestone yet",
            )),
        }
    }

    fn component_member_expression(
        &mut self,
        member: &JSXMemberExpression<'a>,
    ) -> Result<Expression<'a>> {
        let object = match &member.object {
            JSXMemberExpressionObject::IdentifierReference(identifier) => {
                self.component_identifier_expression(&identifier.name)
            }
            JSXMemberExpressionObject::MemberExpression(member) => {
                self.component_member_expression(member)?
            }
            JSXMemberExpressionObject::ThisExpression(this) => {
                self.capture_this_expression(this.span)
            }
        };
        Ok(if is_identifier_key(&member.property.name) {
            Expression::StaticMemberExpression(
                self.ast().alloc_static_member_expression(
                    member.span,
                    object,
                    self.ast()
                        .identifier_name(member.span, self.ast().ident(&member.property.name)),
                    false,
                ),
            )
        } else {
            Expression::ComputedMemberExpression(self.ast().alloc_computed_member_expression(
                member.span,
                object,
                self.ast().expression_string_literal(
                    member.span,
                    self.ast().atom(&member.property.name),
                    None,
                ),
                false,
            ))
        })
    }

    fn component_identifier_expression(&mut self, component: &str) -> Expression<'a> {
        if self.built_ins.iter().any(|built_in| built_in == component) {
            if !self
                .built_in_imports
                .iter()
                .any(|built_in| built_in == component)
            {
                self.built_in_imports.push(component.to_string());
            }
            self.ast()
                .expression_identifier(Span::new(0, 0), self.ast().ident(&format!("_{component}")))
        } else {
            self.ast()
                .expression_identifier(Span::new(0, 0), self.ast().ident(component))
        }
    }

    fn capture_this_expression(&mut self, span: Span) -> Expression<'a> {
        let name = if let Some(name) = &self.pending_this_capture {
            let name = name.clone();
            self.current_this_capture = Some(name.clone());
            name
        } else {
            self.this_index += 1;
            let name = if self.this_index == 1 {
                "_self$".to_string()
            } else {
                format!("_self${}", self.this_index)
            };
            self.pending_this_capture = Some(name.clone());
            self.current_this_capture = Some(name.clone());
            name
        };
        self.ast()
            .expression_identifier(span, self.ast().ident(&name))
    }

    fn take_this_capture_statement(&mut self, span: Span) -> Option<Statement<'a>> {
        let name = self.pending_this_capture.take()?;
        Some(self.const_statement(span, &name, self.ast().expression_this(span)))
    }

    fn clear_this_capture_context(&mut self) {
        self.current_this_capture = None;
    }

    fn const_statement(&self, span: Span, name: &str, init: Expression<'a>) -> Statement<'a> {
        variable_statement(
            self.allocator,
            span,
            oxc_ast::ast::VariableDeclarationKind::Const,
            name,
            init,
        )
    }

    fn import_named(&self, imported: &str, local: &str) -> Statement<'a> {
        import_named(self.allocator, self.module_name, imported, local)
    }
}

impl<'a> ComponentPropContext<'a> for AstSsrTransform<'a, '_> {
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
        self.call_expression(
            span,
            self.ast()
                .expression_identifier(span, self.ast().ident(callee)),
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

fn flush_props<'a>(
    ctx: &AstSsrTransform<'a, '_>,
    span: Span,
    running_props: &mut std::vec::Vec<ObjectPropertyKind<'a>>,
    prop_objects: &mut std::vec::Vec<Expression<'a>>,
) {
    if running_props.is_empty() {
        return;
    }
    let props = std::mem::take(running_props);
    prop_objects.push(
        ctx.ast()
            .expression_object(span, ctx.ast().vec_from_iter(props)),
    );
}

impl<'a> AstSsrTransform<'a, '_> {
    pub(crate) fn process_statements(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>) {
        self.statement_depth += 1;
        let mut body = ArenaVec::new_in(self.allocator);
        for mut statement in statements.drain(..) {
            if self.error.is_some() {
                body.push(statement);
                continue;
            }
            self.visit_statement(&mut statement);
            if let Some(capture) = self.take_this_capture_statement(statement.span()) {
                body.push(capture);
                self.clear_this_capture_context();
            }
            body.push(statement);
        }
        *statements = body;
        self.statement_depth -= 1;
    }

    pub(crate) fn lower_class_field_value(
        &mut self,
        span: Span,
        value: Expression<'a>,
    ) -> Expression<'a> {
        if let Expression::ArrowFunctionExpression(mut arrow) = value {
            if arrow.expression && arrow.body.statements.len() == 1 {
                if let Some(Statement::ExpressionStatement(statement)) = arrow.body.statements.pop()
                {
                    let mut expression = statement.unbox().expression;
                    self.visit_expression(&mut expression);
                    arrow.expression = false;
                    let mut statements = self.ast().vec();
                    if let Some(capture) = self.take_this_capture_statement(span) {
                        statements.push(capture);
                        self.clear_this_capture_context();
                    }
                    statements.push(self.ast().statement_return(span, Some(expression)));
                    arrow.body.statements = statements;
                    return Expression::ArrowFunctionExpression(arrow);
                }
            }
            return Expression::ArrowFunctionExpression(arrow);
        }

        let mut value = value;
        self.visit_expression(&mut value);
        if let Some(capture) = self.take_this_capture_statement(span) {
            let mut statements = self.ast().vec();
            statements.push(capture);
            statements.push(self.ast().statement_return(span, Some(value)));
            let arrow = self.arrow_iife(span, statements);
            self.clear_this_capture_context();
            self.call_expression(span, arrow, std::vec::Vec::new())
        } else {
            value
        }
    }

    fn arrow_iife(&self, span: Span, statements: ArenaVec<'a, Statement<'a>>) -> Expression<'a> {
        let params = self.ast().formal_parameters(
            span,
            FormalParameterKind::ArrowFormalParameters,
            self.ast().vec(),
            NONE,
        );
        let body = self.ast().function_body(span, self.ast().vec(), statements);
        self.ast()
            .expression_arrow_function(span, false, false, NONE, params, NONE, body)
    }
}

fn child_slot_allocates_ids(child: &JSXChild<'_>) -> bool {
    match child {
        JSXChild::Element(_) | JSXChild::Fragment(_) | JSXChild::Spread(_) => true,
        JSXChild::ExpressionContainer(container) => {
            jsx_expression_can_return_hydratable_child(&container.expression)
        }
        _ => false,
    }
}

fn jsx_expression_can_return_hydratable_child(expression: &JSXExpression<'_>) -> bool {
    match expression {
        JSXExpression::JSXElement(_)
        | JSXExpression::JSXFragment(_)
        | JSXExpression::CallExpression(_) => true,
        JSXExpression::StaticMemberExpression(member) => member.property.name == "children",
        JSXExpression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                member.property.name == "children"
            }
            _ => false,
        },
        JSXExpression::ConditionalExpression(conditional) => {
            expression_can_return_hydratable_child(&conditional.consequent)
                || expression_can_return_hydratable_child(&conditional.alternate)
        }
        JSXExpression::LogicalExpression(logical) => {
            expression_can_return_hydratable_child(&logical.right)
        }
        _ => false,
    }
}

fn expression_can_return_hydratable_child(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::JSXElement(_) | Expression::JSXFragment(_) | Expression::CallExpression(_) => {
            true
        }
        Expression::StaticMemberExpression(member) => member.property.name == "children",
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                member.property.name == "children"
            }
            _ => false,
        },
        Expression::ConditionalExpression(conditional) => {
            expression_can_return_hydratable_child(&conditional.consequent)
                || expression_can_return_hydratable_child(&conditional.alternate)
        }
        Expression::LogicalExpression(logical) => {
            expression_can_return_hydratable_child(&logical.right)
        }
        _ => false,
    }
}

fn is_deferred_child_slot_expression(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => true,
        Expression::CallExpression(call) => matches!(
            call.callee,
            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
        ),
        _ => false,
    }
}

fn escape_html_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}
