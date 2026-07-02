mod config;
mod dom;
mod shared;
mod ssr;
mod universal;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use oxc_allocator::Allocator;
use oxc_ast_visit::VisitMut;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;

use config::source_type_for_filename;
use config::RendererOption;
pub use config::{TransformOptions, TransformResult};
use dom::element::{AstDomTransform, DomTransformConfig};
use ssr::transform::AstSsrTransform;
use universal::transform::{AstUniversalTransform, DynamicDomConfig};

#[napi]
pub fn transform(code: String, options: Option<TransformOptions>) -> Result<TransformResult> {
    let options = options.unwrap_or_default();
    let source_type = source_type_for_filename(options.filename.as_deref())?;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &code, source_type).parse();

    if let Some(error) = parsed.errors.into_iter().next() {
        return Err(Error::from_reason(error.to_string()));
    }

    let module_name = options
        .module_name
        .as_deref()
        .ok_or_else(|| Error::from_reason("AST-native transform requires a `moduleName` option"))?;

    let mut program = parsed.program;
    match options.generate.as_deref().unwrap_or("dom") {
        "dom" => {
            let mut transform = AstDomTransform::new(
                &allocator,
                &code,
                module_name,
                dom_transform_config(&options, built_ins(&options)),
            );
            transform.visit_program(&mut program);
            if let Some(error) = transform.error {
                return Err(Error::from_reason(error));
            }
            transform.prepend_helpers(&mut program)?;
        }
        "dynamic" => {
            if let Some(renderer) = dom_renderer(options.renderers.as_deref()) {
                let mut transform = AstUniversalTransform::new_dynamic(
                    &allocator,
                    &code,
                    module_name,
                    built_ins(&options),
                    dynamic_dom_config(&options, renderer, module_name),
                );
                transform.visit_program(&mut program);
                if let Some(error) = transform.error {
                    return Err(Error::from_reason(error));
                }
                transform.prepend_helpers(&mut program);
                if let Some(error) = transform.error {
                    return Err(Error::from_reason(error));
                }
            } else {
                let mut transform = AstUniversalTransform::new(
                    &allocator,
                    &code,
                    module_name,
                    built_ins(&options),
                    static_marker(&options),
                );
                transform.visit_program(&mut program);
                if let Some(error) = transform.error {
                    return Err(Error::from_reason(error));
                }
                transform.prepend_helpers(&mut program);
            }
        }
        "ssr" => {
            let mut transform = AstSsrTransform::new(
                &allocator,
                &code,
                module_name,
                options.hydratable.unwrap_or(false),
                static_marker(&options),
                built_ins(&options),
            );
            transform.visit_program(&mut program);
            if let Some(error) = transform.error {
                return Err(Error::from_reason(error));
            }
            transform.prepend_helpers(&mut program);
        }
        "universal" => {
            let mut transform = AstUniversalTransform::new(
                &allocator,
                &code,
                module_name,
                built_ins(&options),
                static_marker(&options),
            );
            transform.visit_program(&mut program);
            if let Some(error) = transform.error {
                return Err(Error::from_reason(error));
            }
            transform.prepend_helpers(&mut program);
        }
        _ => {
            return Err(Error::from_reason(
                "The jsx-dom-expressions-compiler backend implements DOM, SSR, universal, and dynamic modes only",
            ));
        }
    }

    let build = Codegen::new()
        .with_options(CodegenOptions {
            source_map_path: options.source_map.unwrap_or(false).then(|| {
                std::path::PathBuf::from(options.filename.as_deref().unwrap_or("input.jsx"))
            }),
            ..CodegenOptions::default()
        })
        .build(&program);

    Ok(TransformResult {
        code: build.code,
        map: build.map.map(|map| map.to_json_string()),
    })
}

fn dom_transform_config(options: &TransformOptions, built_ins: Vec<String>) -> DomTransformConfig {
    DomTransformConfig {
        hydratable: options.hydratable.unwrap_or(false),
        dev: options.dev.unwrap_or(false),
        context_to_custom_elements: options.context_to_custom_elements.unwrap_or(true),
        delegate_events: options.delegate_events.unwrap_or(true),
        delegated_events: options.delegated_events.clone().unwrap_or_default(),
        omit_quotes: options.omit_quotes.unwrap_or(true),
        omit_attribute_spacing: options.omit_attribute_spacing.unwrap_or(true),
        inline_styles: options.inline_styles.unwrap_or(true),
        effect_wrapper: options.effect_wrapper.unwrap_or(true),
        wrap_conditionals: options.wrap_conditionals.unwrap_or(true),
        memo_wrapper: options.memo_wrapper.unwrap_or(true),
        static_marker: static_marker(options),
        omit_nested_closing_tags: options.omit_nested_closing_tags.unwrap_or(false),
        omit_last_closing_tag: options.omit_last_closing_tag.unwrap_or(true),
        built_ins,
    }
}

fn dynamic_dom_config<'source>(
    options: &TransformOptions,
    renderer: &'source RendererOption,
    default_module_name: &'source str,
) -> DynamicDomConfig<'source> {
    let dom = dom_transform_config(options, std::vec::Vec::new());
    DynamicDomConfig {
        module_name: renderer
            .module_name
            .as_deref()
            .unwrap_or(default_module_name),
        elements: renderer.elements.clone(),
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
        static_marker: dom.static_marker,
        omit_nested_closing_tags: dom.omit_nested_closing_tags,
        omit_last_closing_tag: dom.omit_last_closing_tag,
    }
}

fn static_marker(options: &TransformOptions) -> String {
    options
        .static_marker
        .clone()
        .unwrap_or_else(|| "@static".to_string())
}

fn built_ins(options: &TransformOptions) -> Vec<String> {
    options.built_ins.clone().unwrap_or_default()
}

fn dom_renderer(renderers: Option<&[RendererOption]>) -> Option<&RendererOption> {
    renderers
        .unwrap_or(&[])
        .iter()
        .find(|renderer| renderer.name == "dom")
}
