use oxc_ast::ast::{
    BindingPattern, ImportDeclarationSpecifier, Statement, VariableDeclarationKind,
};

use crate::dom::element::AstDomTransform;
use crate::shared::utils::static_expression;

impl<'a> AstDomTransform<'a, '_> {
    pub(crate) fn collect_static_bindings(&mut self, statement: &Statement<'a>) {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    if declaration.kind == VariableDeclarationKind::Const {
                        collect_binding_names(&declarator.id, &mut self.const_bindings);
                    }
                    let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
                        continue;
                    };
                    let name = binding.name.to_string();
                    let Some(init) = &declarator.init else {
                        continue;
                    };
                    let Some(value) = static_expression(init, &self.static_bindings) else {
                        continue;
                    };
                    self.static_bindings
                        .retain(|(existing, _)| existing != &name);
                    self.static_bindings.push((name, value));
                }
            }
            Statement::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    push_unique(&mut self.function_bindings, &id.name);
                }
            }
            Statement::ImportDeclaration(import_declaration) => {
                if let Some(specifiers) = &import_declaration.specifiers {
                    for specifier in specifiers {
                        match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                                push_unique(&mut self.const_bindings, &specifier.local.name);
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                                push_unique(&mut self.const_bindings, &specifier.local.name);
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                                push_unique(&mut self.const_bindings, &specifier.local.name);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn push_unique(values: &mut std::vec::Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

fn collect_binding_names(pattern: &BindingPattern<'_>, names: &mut std::vec::Vec<String>) {
    match pattern {
        BindingPattern::BindingIdentifier(binding) => push_unique(names, &binding.name),
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_binding_names(element, names);
            }
            if let Some(rest) = &array.rest {
                collect_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_binding_names(&property.value, names);
            }
            if let Some(rest) = &object.rest {
                collect_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            collect_binding_names(&assignment.left, names);
        }
    }
}
