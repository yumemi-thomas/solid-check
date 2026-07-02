use napi::bindgen_prelude::*;
use oxc_ast::{ast::Expression, AstBuilder};
use oxc_span::Span;

use crate::shared::component_callee::ComponentCalleeContext;
use crate::universal::transform::AstUniversalTransform;

impl<'a> ComponentCalleeContext<'a> for AstUniversalTransform<'a, '_> {
    fn ast(&self) -> AstBuilder<'a> {
        self.ast()
    }

    fn is_built_in(&self, name: &str) -> bool {
        self.built_ins.iter().any(|built_in| built_in == name)
    }

    fn register_built_in(&mut self, name: &str) {
        if !self
            .built_in_imports
            .iter()
            .any(|built_in| built_in == name)
        {
            self.built_in_imports.push(name.to_string());
        }
    }

    fn capture_this_callee(&mut self, _span: Span) -> Result<Expression<'a>> {
        Err(Error::from_reason(
            "Universal this-component callees are not implemented in the AST-native milestone yet",
        ))
    }
}
