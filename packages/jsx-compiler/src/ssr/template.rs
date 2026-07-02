use oxc_ast::ast::Expression;

pub(super) struct SsrTemplate<'a> {
    pub(super) parts: std::vec::Vec<String>,
    pub(super) values: std::vec::Vec<Expression<'a>>,
}

impl<'a> SsrTemplate<'a> {
    pub(super) fn new(initial: String) -> Self {
        Self {
            parts: vec![initial],
            values: std::vec::Vec::new(),
        }
    }

    pub(super) fn current_mut(&mut self) -> &mut String {
        self.parts
            .last_mut()
            .expect("SSR template always has a current part")
    }

    pub(super) fn push_expr(&mut self, value: Expression<'a>) {
        self.values.push(value);
        self.parts.push(String::new());
    }

    pub(super) fn append_template(&mut self, child: SsrTemplate<'a>) {
        let mut child_parts = child.parts.into_iter();
        if let Some(first) = child_parts.next() {
            self.current_mut().push_str(&first);
        }
        for (value, next_part) in child.values.into_iter().zip(child_parts) {
            self.push_expr(value);
            self.current_mut().push_str(&next_part);
        }
    }
}
