use napi::bindgen_prelude::*;
use oxc_allocator::CloneIn;
use oxc_allocator::Vec as ArenaVec;
use oxc_ast::{
    ast::{
        ArrayExpressionElement, AssignmentTarget, Expression, FormalParameterKind, FunctionType,
        ObjectPropertyKind, Program, PropertyKey, PropertyKind, Statement, TemplateElementValue,
        VariableDeclarationKind,
    },
    AstBuilder, NONE,
};
use oxc_span::Span;
use oxc_syntax::number::NumberBase;

use crate::dom::element::AstDomTransform;
use crate::shared::ast::{
    arrow_iife, arrow_return_expression, expression_to_argument, import_named,
    object_getter_property, object_getter_property_with_setup, object_property, variable_statement,
};
use crate::shared::utils::{is_identifier_key, template_id};

pub(crate) struct DomTemplateState {
    pub(crate) templates: std::vec::Vec<DomTemplate>,
    pub(crate) uses_template: bool,
    pub(crate) uses_get_next_element: bool,
    pub(crate) uses_get_first_child: bool,
    pub(crate) uses_get_next_sibling: bool,
    pub(crate) uses_get_owner: bool,
    pub(crate) uses_insert: bool,
    pub(crate) uses_scope: bool,
    pub(crate) uses_memo: bool,
    pub(crate) uses_create_component: bool,
    pub(crate) uses_spread: bool,
    pub(crate) uses_merge_props: bool,
    pub(crate) uses_apply_ref: bool,
    pub(crate) uses_ref: bool,
    pub(crate) uses_style: bool,
    pub(crate) uses_set_style_property: bool,
    pub(crate) uses_class_name: bool,
    pub(crate) uses_effect: bool,
    pub(crate) uses_set_attribute: bool,
    pub(crate) uses_set_attribute_ns: bool,
    pub(crate) uses_add_event_listener: bool,
    pub(crate) uses_delegate_events: bool,
    pub(crate) uses_run_hydration_events: bool,
    pub(crate) delegated_events: std::vec::Vec<String>,
    pub(crate) built_in_imports: std::vec::Vec<String>,
}

pub(crate) struct DomTemplate {
    pub(crate) html: String,
    pub(crate) custom_element: bool,
}

impl DomTemplateState {
    pub(crate) fn new() -> Self {
        Self {
            templates: std::vec::Vec::new(),
            uses_template: false,
            uses_get_next_element: false,
            uses_get_first_child: false,
            uses_get_next_sibling: false,
            uses_get_owner: false,
            uses_insert: false,
            uses_scope: false,
            uses_memo: false,
            uses_create_component: false,
            uses_spread: false,
            uses_merge_props: false,
            uses_apply_ref: false,
            uses_ref: false,
            uses_style: false,
            uses_set_style_property: false,
            uses_class_name: false,
            uses_effect: false,
            uses_set_attribute: false,
            uses_set_attribute_ns: false,
            uses_add_event_listener: false,
            uses_delegate_events: false,
            uses_run_hydration_events: false,
            delegated_events: std::vec::Vec::new(),
            built_in_imports: std::vec::Vec::new(),
        }
    }
}

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn prepend_helpers(&mut self, program: &mut Program<'a>) -> Result<()> {
        let mut statements = std::vec::Vec::new();
        if self.template_state.uses_template {
            statements.push(self.import_named("template", "_$template"));
        }
        if self.template_state.uses_get_next_element {
            statements.push(self.import_named("getNextElement", "_$getNextElement"));
        }
        if self.template_state.uses_get_first_child {
            statements.push(self.import_named("getFirstChild", "_$getFirstChild"));
        }
        if self.template_state.uses_get_next_sibling {
            statements.push(self.import_named("getNextSibling", "_$getNextSibling"));
        }
        if self.template_state.uses_get_owner {
            statements.push(self.import_named("getOwner", "_$getOwner"));
        }
        if self.template_state.uses_insert {
            statements.push(self.import_named("insert", "_$insert"));
        }
        if self.template_state.uses_scope {
            statements.push(self.import_named("scope", "_$scope"));
        }
        if self.template_state.uses_memo {
            statements.push(self.import_named("memo", "_$memo"));
        }
        if self.template_state.uses_create_component {
            statements.push(self.import_named("createComponent", "_$createComponent"));
        }
        if self.template_state.uses_spread {
            statements.push(self.import_named("spread", "_$spread"));
        }
        if self.template_state.uses_merge_props {
            statements.push(self.import_named("mergeProps", "_$mergeProps"));
        }
        if self.template_state.uses_apply_ref {
            statements.push(self.import_named("applyRef", "_$applyRef"));
        }
        if self.template_state.uses_ref {
            statements.push(self.import_named("ref", "_$ref"));
        }
        if self.template_state.uses_style {
            statements.push(self.import_named("style", "_$style"));
        }
        if self.template_state.uses_set_style_property {
            statements.push(self.import_named("setStyleProperty", "_$setStyleProperty"));
        }
        if self.template_state.uses_class_name {
            statements.push(self.import_named("className", "_$className"));
        }
        if self.template_state.uses_effect {
            statements.push(self.import_named("effect", "_$effect"));
        }
        if self.template_state.uses_set_attribute {
            statements.push(self.import_named("setAttribute", "_$setAttribute"));
        }
        if self.template_state.uses_set_attribute_ns {
            statements.push(self.import_named("setAttributeNS", "_$setAttributeNS"));
        }
        if self.template_state.uses_add_event_listener {
            statements.push(self.import_named("addEvent", "_$addEvent"));
        }
        if self.template_state.uses_delegate_events {
            statements.push(self.import_named("delegateEvents", "_$delegateEvents"));
        }
        if self.template_state.uses_run_hydration_events {
            statements.push(self.import_named("runHydrationEvents", "_$runHydrationEvents"));
        }
        for built_in in &self.template_state.built_in_imports {
            statements.push(self.import_named(built_in, &format!("_${built_in}")));
        }
        for (index, template) in self.template_state.templates.iter().enumerate() {
            statements.push(self.template_declaration(index, template));
        }

        statements.extend(program.body.drain(..));
        if self.template_state.uses_delegate_events {
            statements.push(self.delegate_events_statement());
        }
        let mut body = ArenaVec::new_in(self.allocator);
        body.extend(statements);
        program.body = body;
        Ok(())
    }

    pub(crate) fn template_id_with_options(
        &mut self,
        template: String,
        custom_element: bool,
    ) -> String {
        self.template_state.uses_template = true;
        if let Some(index) = self.template_state.templates.iter().position(|candidate| {
            candidate.html == template && candidate.custom_element == custom_element
        }) {
            template_id(index)
        } else {
            self.template_state.templates.push(DomTemplate {
                html: template,
                custom_element,
            });
            template_id(self.template_state.templates.len() - 1)
        }
    }

    pub(crate) fn ast(&self) -> AstBuilder<'a> {
        AstBuilder::new(self.allocator)
    }

    pub(crate) fn insert_statement(
        &self,
        span: Span,
        parent: &str,
        value: Expression<'a>,
        marker: Option<Expression<'a>>,
    ) -> Statement<'a> {
        let mut args = vec![self.identifier_expression(span, parent), value];
        if let Some(marker) = marker {
            args.push(marker);
        }
        self.ast()
            .statement_expression(span, self.call_identifier(span, "_$insert", args))
    }

    pub(crate) fn object_property(
        &self,
        span: Span,
        name: &str,
        value: Expression<'a>,
    ) -> ObjectPropertyKind<'a> {
        object_property(self.allocator, span, name, value)
    }

    pub(crate) fn object_getter_property(
        &self,
        span: Span,
        name: &str,
        value: Expression<'a>,
    ) -> ObjectPropertyKind<'a> {
        object_getter_property(self.allocator, span, name, value)
    }

    pub(crate) fn object_getter_property_with_setup(
        &self,
        span: Span,
        name: &str,
        setup: std::vec::Vec<Statement<'a>>,
        value: Expression<'a>,
    ) -> ObjectPropertyKind<'a> {
        object_getter_property_with_setup(self.allocator, span, name, setup, value)
    }

    pub(crate) fn object_method_property(
        &self,
        span: Span,
        name: &str,
        param_name: &str,
        statements: oxc_allocator::Vec<'a, Statement<'a>>,
    ) -> ObjectPropertyKind<'a> {
        let key = if is_identifier_key(name) {
            self.ast()
                .property_key_static_identifier(span, self.ast().ident(name))
        } else {
            PropertyKey::StringLiteral(self.ast().alloc_string_literal(
                span,
                self.ast().atom(name),
                None,
            ))
        };
        let param = self.ast().formal_parameter(
            span,
            self.ast().vec(),
            self.ast()
                .binding_pattern_binding_identifier(span, self.ast().ident(param_name)),
            NONE,
            NONE,
            false,
            None,
            false,
            false,
        );
        let params = self.ast().formal_parameters(
            span,
            FormalParameterKind::FormalParameter,
            self.ast().vec1(param),
            NONE,
        );
        let body = self.ast().function_body(span, self.ast().vec(), statements);
        let value = self.ast().expression_function(
            span,
            FunctionType::FunctionExpression,
            None,
            false,
            false,
            false,
            NONE,
            NONE,
            params,
            NONE,
            Some(body),
        );
        self.ast().object_property_kind_object_property(
            span,
            PropertyKind::Init,
            key,
            value,
            true,
            false,
            false,
        )
    }

    pub(crate) fn call_identifier(
        &self,
        span: Span,
        callee: &str,
        args: std::vec::Vec<Expression<'a>>,
    ) -> Expression<'a> {
        self.call_expression(span, self.identifier_expression(span, callee), args)
    }

    pub(crate) fn call_expression(
        &self,
        span: Span,
        callee: Expression<'a>,
        args: std::vec::Vec<Expression<'a>>,
    ) -> Expression<'a> {
        let args = self
            .ast()
            .vec_from_iter(args.into_iter().map(expression_to_argument));
        self.ast().expression_call(span, callee, NONE, args, false)
    }

    pub(crate) fn identifier_expression(&self, span: Span, name: &str) -> Expression<'a> {
        self.ast()
            .expression_identifier(span, self.ast().ident(name))
    }

    pub(crate) fn static_member_expression(
        &self,
        span: Span,
        object: &str,
        property: &str,
    ) -> Expression<'a> {
        Expression::StaticMemberExpression(self.ast().alloc_static_member_expression(
            span,
            self.identifier_expression(span, object),
            self.ast().identifier_name(span, self.ast().ident(property)),
            false,
        ))
    }

    pub(crate) fn static_member_expression_from_expression(
        &self,
        span: Span,
        object: Expression<'a>,
        property: &str,
    ) -> Expression<'a> {
        Expression::StaticMemberExpression(self.ast().alloc_static_member_expression(
            span,
            object,
            self.ast().identifier_name(span, self.ast().ident(property)),
            false,
        ))
    }

    pub(crate) fn assignment_target_from_static_member(
        &self,
        member: &oxc_ast::ast::StaticMemberExpression<'a>,
    ) -> AssignmentTarget<'a> {
        AssignmentTarget::StaticMemberExpression(oxc_allocator::Box::new_in(
            member.clone_in(self.allocator),
            self.allocator,
        ))
    }

    pub(crate) fn child_node_expression(
        &self,
        span: Span,
        parent: &str,
        index: usize,
    ) -> Expression<'a> {
        let mut expression = self.static_member_expression(span, parent, "firstChild");
        for _ in 0..index {
            expression =
                self.static_member_expression_from_expression(span, expression, "nextSibling");
        }
        expression
    }

    pub(crate) fn variable_statement(
        &self,
        span: Span,
        name: &str,
        init: Expression<'a>,
    ) -> Statement<'a> {
        self.variable_statement_with_kind(span, VariableDeclarationKind::Var, name, init)
    }

    pub(crate) fn const_statement(
        &self,
        span: Span,
        name: &str,
        init: Expression<'a>,
    ) -> Statement<'a> {
        self.variable_statement_with_kind(span, VariableDeclarationKind::Const, name, init)
    }

    fn variable_statement_with_kind(
        &self,
        span: Span,
        kind: VariableDeclarationKind,
        name: &str,
        init: Expression<'a>,
    ) -> Statement<'a> {
        variable_statement(self.allocator, span, kind, name, init)
    }

    pub(crate) fn arrow_iife(
        &self,
        span: Span,
        statements: ArenaVec<'a, Statement<'a>>,
    ) -> Expression<'a> {
        arrow_iife(self.allocator, span, statements)
    }

    pub(crate) fn arrow_return_expression(
        &self,
        span: Span,
        value: Expression<'a>,
    ) -> Expression<'a> {
        arrow_return_expression(self.allocator, span, value)
    }

    fn import_named(&self, imported: &str, local: &str) -> Statement<'a> {
        import_named(self.allocator, self.module_name, imported, local)
    }

    fn template_declaration(&self, index: usize, template: &DomTemplate) -> Statement<'a> {
        let span = Span::new(0, 0);
        let template_literal = self.template_literal_expression(span, &template.html);
        let mut args = vec![template_literal];
        if template.custom_element {
            args.push(
                self.ast()
                    .expression_numeric_literal(span, 1.0, None, NumberBase::Decimal),
            );
        }
        let mut init = self.call_identifier(span, "_$template", args);
        if let Expression::CallExpression(call) = &mut init {
            call.pure = true;
        }
        self.variable_statement(span, &template_id(index), init)
    }

    fn template_literal_expression(&self, span: Span, value: &str) -> Expression<'a> {
        let element = self.ast().template_element(
            span,
            TemplateElementValue {
                raw: self.ast().atom(value),
                cooked: Some(self.ast().atom(value)),
            },
            true,
            true,
        );
        self.ast()
            .expression_template_literal(span, self.ast().vec1(element), self.ast().vec())
    }

    fn delegate_events_statement(&self) -> Statement<'a> {
        let span = Span::new(0, 0);
        let events = self
            .ast()
            .vec_from_iter(self.template_state.delegated_events.iter().map(|event| {
                ArrayExpressionElement::StringLiteral(self.ast().alloc_string_literal(
                    span,
                    self.ast().atom(event),
                    None,
                ))
            }));
        let events = self.ast().expression_array(span, events);
        self.ast().statement_expression(
            span,
            self.call_identifier(span, "_$delegateEvents", vec![events]),
        )
    }
}
