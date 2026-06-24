use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{
        Argument, Expression, FormalParameterKind, FunctionType, ImportOrExportKind,
        ObjectPropertyKind, PropertyKey, PropertyKind, Statement, VariableDeclarationKind,
    },
    AstBuilder, NONE,
};
use oxc_span::Span;

use crate::shared::utils::is_identifier_key;

fn ast<'a>(allocator: &'a Allocator) -> AstBuilder<'a> {
    AstBuilder::new(allocator)
}

pub(crate) fn expression_to_argument<'a>(expression: Expression<'a>) -> Argument<'a> {
    match expression {
        Expression::BooleanLiteral(value) => Argument::BooleanLiteral(value),
        Expression::NullLiteral(value) => Argument::NullLiteral(value),
        Expression::NumericLiteral(value) => Argument::NumericLiteral(value),
        Expression::BigIntLiteral(value) => Argument::BigIntLiteral(value),
        Expression::RegExpLiteral(value) => Argument::RegExpLiteral(value),
        Expression::StringLiteral(value) => Argument::StringLiteral(value),
        Expression::ArrayExpression(value) => Argument::ArrayExpression(value),
        Expression::TemplateLiteral(value) => Argument::TemplateLiteral(value),
        Expression::Identifier(value) => Argument::Identifier(value),
        Expression::MetaProperty(value) => Argument::MetaProperty(value),
        Expression::Super(value) => Argument::Super(value),
        Expression::ThisExpression(value) => Argument::ThisExpression(value),
        Expression::CallExpression(value) => Argument::CallExpression(value),
        Expression::ChainExpression(value) => Argument::ChainExpression(value),
        Expression::ObjectExpression(value) => Argument::ObjectExpression(value),
        Expression::ArrowFunctionExpression(value) => Argument::ArrowFunctionExpression(value),
        Expression::AssignmentExpression(value) => Argument::AssignmentExpression(value),
        Expression::AwaitExpression(value) => Argument::AwaitExpression(value),
        Expression::BinaryExpression(value) => Argument::BinaryExpression(value),
        Expression::StaticMemberExpression(value) => Argument::StaticMemberExpression(value),
        Expression::ComputedMemberExpression(value) => Argument::ComputedMemberExpression(value),
        Expression::PrivateFieldExpression(value) => Argument::PrivateFieldExpression(value),
        Expression::ClassExpression(value) => Argument::ClassExpression(value),
        Expression::FunctionExpression(value) => Argument::FunctionExpression(value),
        Expression::ImportExpression(value) => Argument::ImportExpression(value),
        Expression::LogicalExpression(value) => Argument::LogicalExpression(value),
        Expression::NewExpression(value) => Argument::NewExpression(value),
        Expression::ConditionalExpression(value) => Argument::ConditionalExpression(value),
        Expression::PrivateInExpression(value) => Argument::PrivateInExpression(value),
        Expression::UnaryExpression(value) => Argument::UnaryExpression(value),
        Expression::UpdateExpression(value) => Argument::UpdateExpression(value),
        Expression::YieldExpression(value) => Argument::YieldExpression(value),
        Expression::ParenthesizedExpression(value) => Argument::ParenthesizedExpression(value),
        Expression::SequenceExpression(value) => Argument::SequenceExpression(value),
        Expression::TaggedTemplateExpression(value) => Argument::TaggedTemplateExpression(value),
        Expression::JSXElement(value) => Argument::JSXElement(value),
        Expression::JSXFragment(value) => Argument::JSXFragment(value),
        Expression::TSAsExpression(value) => Argument::TSAsExpression(value),
        Expression::TSSatisfiesExpression(value) => Argument::TSSatisfiesExpression(value),
        Expression::TSTypeAssertion(value) => Argument::TSTypeAssertion(value),
        Expression::TSNonNullExpression(value) => Argument::TSNonNullExpression(value),
        Expression::TSInstantiationExpression(value) => Argument::TSInstantiationExpression(value),
        Expression::V8IntrinsicExpression(value) => Argument::V8IntrinsicExpression(value),
    }
}

pub(crate) fn import_named<'a>(
    allocator: &'a Allocator,
    module_name: &str,
    imported: &str,
    local: &str,
) -> Statement<'a> {
    let ast = ast(allocator);
    let span = Span::new(0, 0);
    let specifier = ast.import_declaration_specifier_import_specifier(
        span,
        ast.module_export_name_identifier_name(span, ast.ident(imported)),
        ast.binding_identifier(span, ast.ident(local)),
        ImportOrExportKind::Value,
    );
    Statement::ImportDeclaration(ast.alloc_import_declaration(
        span,
        Some(ast.vec1(specifier)),
        ast.string_literal(span, ast.atom(module_name), None),
        None,
        NONE,
        ImportOrExportKind::Value,
    ))
}

pub(crate) fn object_property<'a>(
    allocator: &'a Allocator,
    span: Span,
    name: &str,
    value: Expression<'a>,
) -> ObjectPropertyKind<'a> {
    let ast = ast(allocator);
    let key = if is_identifier_key(name) {
        ast.property_key_static_identifier(span, ast.ident(name))
    } else {
        PropertyKey::StringLiteral(ast.alloc_string_literal(span, ast.atom(name), None))
    };
    ast.object_property_kind_object_property(
        span,
        PropertyKind::Init,
        key,
        value,
        false,
        false,
        false,
    )
}

pub(crate) fn object_getter_property<'a>(
    allocator: &'a Allocator,
    span: Span,
    name: &str,
    value: Expression<'a>,
) -> ObjectPropertyKind<'a> {
    object_getter_property_with_setup(allocator, span, name, Vec::new(), value)
}

pub(crate) fn object_getter_property_with_setup<'a>(
    allocator: &'a Allocator,
    span: Span,
    name: &str,
    setup: std::vec::Vec<Statement<'a>>,
    value: Expression<'a>,
) -> ObjectPropertyKind<'a> {
    let ast = ast(allocator);
    let key = if is_identifier_key(name) {
        ast.property_key_static_identifier(span, ast.ident(name))
    } else {
        PropertyKey::StringLiteral(ast.alloc_string_literal(span, ast.atom(name), None))
    };
    let params = ast.formal_parameters(span, FormalParameterKind::FormalParameter, ast.vec(), NONE);
    let mut statements = ast.vec();
    statements.extend(setup);
    statements.push(ast.statement_return(span, Some(value)));
    let body = ast.function_body(span, ast.vec(), statements);
    let value = ast.expression_function(
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
    ast.object_property_kind_object_property(
        span,
        PropertyKind::Get,
        key,
        value,
        false,
        false,
        false,
    )
}

pub(crate) fn arrow_return_expression<'a>(
    allocator: &'a Allocator,
    span: Span,
    value: Expression<'a>,
) -> Expression<'a> {
    let ast = ast(allocator);
    let params = ast.formal_parameters(
        span,
        FormalParameterKind::ArrowFormalParameters,
        ast.vec(),
        NONE,
    );
    let body = ast.function_body(
        span,
        ast.vec(),
        ast.vec1(ast.statement_return(span, Some(value))),
    );
    ast.expression_arrow_function(span, false, false, NONE, params, NONE, body)
}

pub(crate) fn variable_statement<'a>(
    allocator: &'a Allocator,
    span: Span,
    kind: VariableDeclarationKind,
    name: &str,
    init: Expression<'a>,
) -> Statement<'a> {
    let ast = ast(allocator);
    let declarator = ast.variable_declarator(
        span,
        kind,
        ast.binding_pattern_binding_identifier(span, ast.ident(name)),
        NONE,
        Some(init),
        false,
    );
    Statement::VariableDeclaration(ast.alloc_variable_declaration(
        span,
        kind,
        ast.vec1(declarator),
        false,
    ))
}
