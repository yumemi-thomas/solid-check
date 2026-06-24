use napi::bindgen_prelude::*;
use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::{ClassElement, Expression, JSXElement, JSXFragment, Program, Statement};
use oxc_ast_visit::{walk_mut, VisitMut};
use oxc_span::Span;

use crate::dom::element::AstDomTransform;
use crate::shared::fragment::lower_fragment;
use crate::ssr::transform::AstSsrTransform;
use crate::universal::transform::AstUniversalTransform;

pub(crate) trait JsxTransform<'a>: VisitMut<'a> {
    fn has_error(&self) -> bool;
    fn set_error(&mut self, error: String);
    fn process_statements(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>);
    fn lower_jsx_element(&mut self, element: &JSXElement<'a>) -> Result<Expression<'a>>;
    fn lower_jsx_fragment(&mut self, fragment: &JSXFragment<'a>) -> Result<Expression<'a>>;
    fn lower_class_field_value(&mut self, span: Span, value: Expression<'a>) -> Expression<'a>;
}

pub(crate) fn visit_program<'a, T: JsxTransform<'a>>(target: &mut T, program: &mut Program<'a>) {
    target.process_statements(&mut program.body);
}

pub(crate) fn visit_statements<'a, T: JsxTransform<'a>>(
    target: &mut T,
    statements: &mut ArenaVec<'a, Statement<'a>>,
) {
    target.process_statements(statements);
}

pub(crate) fn visit_expression<'a, T: JsxTransform<'a>>(
    target: &mut T,
    expression: &mut Expression<'a>,
) {
    if target.has_error() {
        return;
    }
    if let Expression::JSXElement(element) = expression {
        match target.lower_jsx_element(element) {
            Ok(replacement) => *expression = replacement,
            Err(error) => target.set_error(error.to_string()),
        }
        return;
    }

    if let Expression::JSXFragment(fragment) = expression {
        match target.lower_jsx_fragment(fragment) {
            Ok(replacement) => *expression = replacement,
            Err(error) => target.set_error(error.to_string()),
        }
        return;
    }

    walk_mut::walk_expression(target, expression);
}

pub(crate) fn visit_class_element<'a, T: JsxTransform<'a>>(
    target: &mut T,
    class_element: &mut ClassElement<'a>,
) {
    let ClassElement::PropertyDefinition(property) = class_element else {
        walk_mut::walk_class_element(target, class_element);
        return;
    };
    let Some(value) = property.value.take() else {
        return;
    };
    property.value = Some(target.lower_class_field_value(property.span, value));
}

impl<'a> JsxTransform<'a> for AstDomTransform<'a, '_> {
    fn has_error(&self) -> bool {
        self.error.is_some()
    }

    fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    fn process_statements(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>) {
        AstDomTransform::process_statements(self, statements);
    }

    fn lower_jsx_element(&mut self, element: &JSXElement<'a>) -> Result<Expression<'a>> {
        self.lower_element(element)
    }

    fn lower_jsx_fragment(&mut self, fragment: &JSXFragment<'a>) -> Result<Expression<'a>> {
        lower_fragment(self, fragment)
    }

    fn lower_class_field_value(&mut self, span: Span, value: Expression<'a>) -> Expression<'a> {
        AstDomTransform::lower_class_field_value(self, span, value)
    }
}

impl<'a> VisitMut<'a> for AstDomTransform<'a, '_> {
    fn visit_program(&mut self, program: &mut Program<'a>) {
        visit_program(self, program);
    }

    fn visit_statements(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>) {
        visit_statements(self, statements);
    }

    fn visit_expression(&mut self, expression: &mut Expression<'a>) {
        visit_expression(self, expression);
    }

    fn visit_class_element(&mut self, class_element: &mut ClassElement<'a>) {
        visit_class_element(self, class_element);
    }
}

impl<'a> JsxTransform<'a> for AstSsrTransform<'a, '_> {
    fn has_error(&self) -> bool {
        self.error.is_some()
    }

    fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    fn process_statements(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>) {
        AstSsrTransform::process_statements(self, statements);
    }

    fn lower_jsx_element(&mut self, element: &JSXElement<'a>) -> Result<Expression<'a>> {
        self.lower_element(element)
    }

    fn lower_jsx_fragment(&mut self, fragment: &JSXFragment<'a>) -> Result<Expression<'a>> {
        self.lower_fragment(fragment)
    }

    fn lower_class_field_value(&mut self, span: Span, value: Expression<'a>) -> Expression<'a> {
        AstSsrTransform::lower_class_field_value(self, span, value)
    }
}

impl<'a> VisitMut<'a> for AstSsrTransform<'a, '_> {
    fn visit_program(&mut self, program: &mut Program<'a>) {
        visit_program(self, program);
    }

    fn visit_statements(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>) {
        visit_statements(self, statements);
    }

    fn visit_expression(&mut self, expression: &mut Expression<'a>) {
        visit_expression(self, expression);
    }

    fn visit_class_element(&mut self, class_element: &mut ClassElement<'a>) {
        visit_class_element(self, class_element);
    }
}

impl<'a> JsxTransform<'a> for AstUniversalTransform<'a, '_> {
    fn has_error(&self) -> bool {
        self.error.is_some()
    }

    fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    fn process_statements(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>) {
        AstUniversalTransform::process_statements(self, statements);
    }

    fn lower_jsx_element(&mut self, element: &JSXElement<'a>) -> Result<Expression<'a>> {
        let (replacement, setup) = self.lower_element(element)?;
        Ok(self.setup_iife(element.span, setup, replacement))
    }

    fn lower_jsx_fragment(&mut self, fragment: &JSXFragment<'a>) -> Result<Expression<'a>> {
        self.lower_fragment(fragment)
    }

    fn lower_class_field_value(
        &mut self,
        _span: Span,
        mut value: Expression<'a>,
    ) -> Expression<'a> {
        self.visit_expression(&mut value);
        value
    }
}

impl<'a> VisitMut<'a> for AstUniversalTransform<'a, '_> {
    fn visit_program(&mut self, program: &mut Program<'a>) {
        visit_program(self, program);
    }

    fn visit_statements(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>) {
        visit_statements(self, statements);
    }

    fn visit_expression(&mut self, expression: &mut Expression<'a>) {
        visit_expression(self, expression);
    }

    fn visit_class_element(&mut self, class_element: &mut ClassElement<'a>) {
        visit_class_element(self, class_element);
    }
}
