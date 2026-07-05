use oxc_allocator::CloneIn;
use oxc_ast::ast::Expression;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, LogicalOperator, UnaryOperator};

use crate::dom::element::AstDomTransform;

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn condition_child_expression(
        &mut self,
        span: Span,
        value: Expression<'a>,
    ) -> Expression<'a> {
        if !self.wrap_conditionals {
            return value;
        }
        let Expression::ConditionalExpression(conditional) = value else {
            return self.logical_child_expression(span, value);
        };

        if !is_dynamic_condition_branch(&conditional.consequent)
            && !is_dynamic_condition_branch(&conditional.alternate)
        {
            return Expression::ConditionalExpression(conditional);
        }
        if !is_dynamic_condition_test(&conditional.test) {
            return Expression::ConditionalExpression(conditional);
        }

        let condition_id = self.next_condition_id();
        let condition_test =
            self.boolean_condition_expression(span, conditional.test.clone_in(self.allocator));
        let memo = self.memo_call(span, condition_test);
        let memo_statement = self.variable_statement(span, &condition_id, memo);
        let condition_call = self.call_identifier(span, &condition_id, std::vec::Vec::new());
        let conditional = self.ast().expression_conditional(
            span,
            condition_call,
            self.inline_condition_expression(span, conditional.consequent.clone_in(self.allocator)),
            self.inline_condition_expression(span, conditional.alternate.clone_in(self.allocator)),
        );
        let reactive = self.arrow_return_expression(span, conditional);
        let mut statements = self.ast().vec();
        statements.push(memo_statement);
        statements.push(self.ast().statement_return(span, Some(reactive)));
        let iife = self.arrow_iife(span, statements);
        self.call_expression(span, iife, std::vec::Vec::new())
    }

    pub(crate) fn dom_child_expression(
        &mut self,
        span: Span,
        value: Expression<'a>,
    ) -> Expression<'a> {
        let value = self.condition_child_expression(span, value);
        if is_condition_iife(&value) {
            return value;
        }
        if let Expression::CallExpression(call) = &value {
            if call.arguments.is_empty() {
                if let Expression::Identifier(_) = &call.callee {
                    return call.callee.clone_in(self.allocator);
                }
            }
        }
        if is_dynamic_expression(&value) {
            return self.arrow_return_expression(span, value);
        }
        value
    }

    fn logical_child_expression(&mut self, span: Span, value: Expression<'a>) -> Expression<'a> {
        if !self.wrap_conditionals {
            return value;
        }
        let Expression::LogicalExpression(logical) = value else {
            return value;
        };
        if logical.operator != LogicalOperator::And
            || !is_dynamic_condition_test(&logical.left)
            || !is_dynamic_condition_branch(&logical.right)
        {
            if logical.operator == LogicalOperator::Or {
                return self.logical_or_child_expression(
                    span,
                    logical.left.clone_in(self.allocator),
                    logical.right.clone_in(self.allocator),
                );
            }
            return Expression::LogicalExpression(logical);
        }

        let condition_id = self.next_condition_id();
        let condition_test =
            self.boolean_condition_expression(span, logical.left.clone_in(self.allocator));
        let memo = self.memo_call(span, condition_test);
        let memo_statement = self.variable_statement(span, &condition_id, memo);
        let condition_call = self.call_identifier(span, &condition_id, std::vec::Vec::new());
        // `left && right` is exactly `left ? right : left`. Branch on the
        // memoized truthiness but return the raw left in the alternate so the
        // expression keeps JS value semantics (`0`/`""`/`undefined` flow
        // through instead of collapsing to `false`), matching the
        // untransformed ssr output. Statically boolean lefts keep the logical
        // form — the memo's value is the expression's value, no second
        // evaluation needed. Mirrors the Babel plugin (#532).
        let right = self.inline_condition_expression(span, logical.right.clone_in(self.allocator));
        let wrapped = if is_boolean_expression(&logical.left) {
            self.ast()
                .expression_logical(span, condition_call, LogicalOperator::And, right)
        } else {
            self.ast().expression_conditional(
                span,
                condition_call,
                right,
                logical.left.clone_in(self.allocator),
            )
        };
        let reactive = self.arrow_return_expression(span, wrapped);
        let mut statements = self.ast().vec();
        statements.push(memo_statement);
        statements.push(self.ast().statement_return(span, Some(reactive)));
        let iife = self.arrow_iife(span, statements);
        self.call_expression(span, iife, std::vec::Vec::new())
    }

    fn logical_or_child_expression(
        &mut self,
        span: Span,
        left: Expression<'a>,
        right: Expression<'a>,
    ) -> Expression<'a> {
        let Expression::LogicalExpression(left_logical) = left else {
            return self
                .ast()
                .expression_logical(span, left, LogicalOperator::Or, right);
        };
        if left_logical.operator != LogicalOperator::And
            || !is_dynamic_condition_test(&left_logical.left)
            || !is_dynamic_condition_branch(&left_logical.right)
        {
            return self.ast().expression_logical(
                span,
                Expression::LogicalExpression(left_logical),
                LogicalOperator::Or,
                right,
            );
        }

        let condition_id = self.next_condition_id();
        let condition_test =
            self.boolean_condition_expression(span, left_logical.left.clone_in(self.allocator));
        let memo = self.memo_call(span, condition_test);
        let memo_statement = self.variable_statement(span, &condition_id, memo);
        let condition_call = self.call_identifier(span, &condition_id, std::vec::Vec::new());
        // Same `&&` → ternary rewrite as `logical_child_expression` (logical
        // form when the left is statically boolean).
        let and_right =
            self.inline_condition_expression(span, left_logical.right.clone_in(self.allocator));
        let left = if is_boolean_expression(&left_logical.left) {
            self.ast()
                .expression_logical(span, condition_call, LogicalOperator::And, and_right)
        } else {
            self.ast().expression_conditional(
                span,
                condition_call,
                and_right,
                left_logical.left.clone_in(self.allocator),
            )
        };
        let logical = self.ast().expression_logical(
            span,
            left,
            LogicalOperator::Or,
            self.inline_condition_expression(span, right),
        );
        let reactive = self.arrow_return_expression(span, logical);
        let mut statements = self.ast().vec();
        statements.push(memo_statement);
        statements.push(self.ast().statement_return(span, Some(reactive)));
        let iife = self.arrow_iife(span, statements);
        self.call_expression(span, iife, std::vec::Vec::new())
    }

    pub(crate) fn condition_component_expression(
        &mut self,
        span: Span,
        value: Expression<'a>,
    ) -> Expression<'a> {
        if !self.wrap_conditionals {
            return value;
        }
        let Expression::ConditionalExpression(conditional) = value else {
            return self.inline_condition_expression(span, value);
        };

        if !is_dynamic_condition_branch(&conditional.consequent)
            && !is_dynamic_condition_branch(&conditional.alternate)
        {
            return Expression::ConditionalExpression(conditional);
        }
        if !is_dynamic_condition_test(&conditional.test) {
            return Expression::ConditionalExpression(conditional);
        }

        let condition_test =
            self.boolean_condition_expression(span, conditional.test.clone_in(self.allocator));
        let memo = self.memo_call(span, condition_test);
        let condition_call = self.call_expression(span, memo, std::vec::Vec::new());
        self.ast().expression_conditional(
            span,
            condition_call,
            self.inline_condition_expression(span, conditional.consequent.clone_in(self.allocator)),
            self.inline_condition_expression(span, conditional.alternate.clone_in(self.allocator)),
        )
    }

    pub(crate) fn memoized_dynamic_expression(
        &mut self,
        span: Span,
        value: Expression<'a>,
    ) -> Expression<'a> {
        match &value {
            Expression::CallExpression(call) if call.arguments.is_empty() => {
                if let Expression::Identifier(_) = &call.callee {
                    if !self.memo_wrapper {
                        return call.callee.clone_in(self.allocator);
                    }
                    self.template_state.uses_memo = true;
                    return self.call_identifier(
                        span,
                        "_$memo",
                        vec![call.callee.clone_in(self.allocator)],
                    );
                }
            }
            _ => {}
        }

        if !self.memo_wrapper {
            return self.arrow_return_expression(span, value);
        }

        self.template_state.uses_memo = true;
        self.call_identifier(
            span,
            "_$memo",
            vec![self.arrow_return_expression(span, value)],
        )
    }

    pub(crate) fn should_memoize_dynamic_expression(&self, value: &Expression<'_>) -> bool {
        matches!(
            value,
            Expression::CallExpression(_)
                | Expression::StaticMemberExpression(_)
                | Expression::ComputedMemberExpression(_)
                | Expression::ChainExpression(_)
                | Expression::ConditionalExpression(_)
                | Expression::LogicalExpression(_)
        )
    }

    fn inline_condition_expression(&mut self, span: Span, value: Expression<'a>) -> Expression<'a> {
        match value {
            Expression::ConditionalExpression(conditional)
                if is_dynamic_condition_test(&conditional.test)
                    && (is_dynamic_condition_branch(&conditional.consequent)
                        || is_dynamic_condition_branch(&conditional.alternate)) =>
            {
                let condition_test = self
                    .boolean_condition_expression(span, conditional.test.clone_in(self.allocator));
                let memo = self.memo_call(span, condition_test);
                let condition_call = self.call_expression(span, memo, std::vec::Vec::new());
                self.ast().expression_conditional(
                    span,
                    condition_call,
                    self.inline_condition_expression(
                        span,
                        conditional.consequent.clone_in(self.allocator),
                    ),
                    self.inline_condition_expression(
                        span,
                        conditional.alternate.clone_in(self.allocator),
                    ),
                )
            }
            Expression::LogicalExpression(logical)
                if logical.operator == LogicalOperator::And
                    && is_dynamic_condition_test(&logical.left)
                    && is_dynamic_condition_branch(&logical.right) =>
            {
                let condition_test =
                    self.boolean_condition_expression(span, logical.left.clone_in(self.allocator));
                let memo = self.memo_call(span, condition_test);
                let condition_call = self.call_expression(span, memo, std::vec::Vec::new());
                // `&&` → ternary with the raw left as alternate (JS value
                // semantics); logical form when the left is statically boolean.
                let right =
                    self.inline_condition_expression(span, logical.right.clone_in(self.allocator));
                if is_boolean_expression(&logical.left) {
                    self.ast()
                        .expression_logical(span, condition_call, LogicalOperator::And, right)
                } else {
                    self.ast().expression_conditional(
                        span,
                        condition_call,
                        right,
                        logical.left.clone_in(self.allocator),
                    )
                }
            }
            Expression::LogicalExpression(logical) => self.ast().expression_logical(
                span,
                self.inline_condition_expression(span, logical.left.clone_in(self.allocator)),
                logical.operator,
                self.inline_condition_expression(span, logical.right.clone_in(self.allocator)),
            ),
            _ => value,
        }
    }

    fn memo_call(&mut self, span: Span, condition: Expression<'a>) -> Expression<'a> {
        if !self.memo_wrapper {
            return self.arrow_return_expression(span, condition);
        }
        self.template_state.uses_memo = true;
        self.call_identifier(
            span,
            "_$memo",
            vec![self.arrow_return_expression(span, condition)],
        )
    }

    fn boolean_condition_expression(&self, span: Span, value: Expression<'a>) -> Expression<'a> {
        if is_boolean_expression(&value) {
            return value;
        }
        let first = self
            .ast()
            .expression_unary(span, UnaryOperator::LogicalNot, value);
        self.ast()
            .expression_unary(span, UnaryOperator::LogicalNot, first)
    }
}

/// Statically guaranteed to evaluate to a boolean: the memoized value IS the
/// expression's value (`false` is the only falsy boolean), so no `!!` coercion
/// is needed and the `&&` wrap keeps the logical form instead of the
/// value-preserving ternary. Mirrors the Babel plugin's `isBooleanExpression`.
fn is_boolean_expression(value: &Expression<'_>) -> bool {
    match value {
        Expression::BinaryExpression(binary) => matches!(
            binary.operator,
            BinaryOperator::Equality
                | BinaryOperator::Inequality
                | BinaryOperator::StrictEquality
                | BinaryOperator::StrictInequality
                | BinaryOperator::LessThan
                | BinaryOperator::LessEqualThan
                | BinaryOperator::GreaterThan
                | BinaryOperator::GreaterEqualThan
                | BinaryOperator::Instanceof
                | BinaryOperator::In
        ),
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::LogicalNot,
        _ => false,
    }
}

fn is_dynamic_condition_branch(value: &Expression<'_>) -> bool {
    match value {
        Expression::CallExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::ChainExpression(_) => true,
        Expression::ConditionalExpression(conditional) => {
            is_dynamic_condition_branch(&conditional.test)
                || is_dynamic_condition_branch(&conditional.consequent)
                || is_dynamic_condition_branch(&conditional.alternate)
        }
        Expression::LogicalExpression(logical) => {
            is_dynamic_condition_branch(&logical.left)
                || is_dynamic_condition_branch(&logical.right)
        }
        Expression::UnaryExpression(unary) => is_dynamic_condition_branch(&unary.argument),
        _ => false,
    }
}

fn is_dynamic_condition_test(value: &Expression<'_>) -> bool {
    match value {
        Expression::CallExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::ChainExpression(_) => true,
        Expression::BinaryExpression(binary) => {
            is_dynamic_condition_test(&binary.left) || is_dynamic_condition_test(&binary.right)
        }
        Expression::LogicalExpression(logical) => {
            is_dynamic_condition_test(&logical.left) || is_dynamic_condition_test(&logical.right)
        }
        Expression::ConditionalExpression(conditional) => {
            is_dynamic_condition_test(&conditional.test)
                || is_dynamic_condition_test(&conditional.consequent)
                || is_dynamic_condition_test(&conditional.alternate)
        }
        // Babel's isDynamic traverses generically, so `!state.hidden` counts —
        // unary arguments must be walked or the generates desync.
        Expression::UnaryExpression(unary) => is_dynamic_condition_test(&unary.argument),
        _ => false,
    }
}

fn is_dynamic_expression(value: &Expression<'_>) -> bool {
    match value {
        Expression::CallExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::ChainExpression(_) => true,
        Expression::ConditionalExpression(conditional) => {
            is_dynamic_expression(&conditional.test)
                || is_dynamic_expression(&conditional.consequent)
                || is_dynamic_expression(&conditional.alternate)
        }
        Expression::LogicalExpression(logical) => {
            is_dynamic_expression(&logical.left) || is_dynamic_expression(&logical.right)
        }
        _ => false,
    }
}

fn is_condition_iife(value: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = value else {
        return false;
    };
    matches!(call.callee, Expression::ArrowFunctionExpression(_))
}
