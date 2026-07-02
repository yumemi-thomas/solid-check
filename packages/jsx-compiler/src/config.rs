use napi::bindgen_prelude::*;
use napi_derive::napi;
use oxc_span::SourceType;

#[napi(object)]
#[derive(Default)]
pub struct RendererOption {
    pub name: String,
    pub module_name: Option<String>,
    pub elements: Vec<String>,
}

#[napi(object)]
#[derive(Default)]
pub struct TransformOptions {
    pub filename: Option<String>,
    pub module_name: Option<String>,
    pub generate: Option<String>,
    pub hydratable: Option<bool>,
    pub dev: Option<bool>,
    pub source_map: Option<bool>,
    pub context_to_custom_elements: Option<bool>,
    pub delegate_events: Option<bool>,
    pub delegated_events: Option<Vec<String>>,
    pub omit_quotes: Option<bool>,
    pub omit_attribute_spacing: Option<bool>,
    pub inline_styles: Option<bool>,
    pub effect_wrapper: Option<bool>,
    pub wrap_conditionals: Option<bool>,
    pub memo_wrapper: Option<bool>,
    pub static_marker: Option<String>,
    pub omit_nested_closing_tags: Option<bool>,
    pub omit_last_closing_tag: Option<bool>,
    pub built_ins: Option<Vec<String>>,
    pub renderers: Option<Vec<RendererOption>>,
}

#[napi(object)]
pub struct TransformResult {
    pub code: String,
    pub map: Option<String>,
}

pub(crate) fn source_type_for_filename(filename: Option<&str>) -> Result<SourceType> {
    filename
        .map(SourceType::from_path)
        .transpose()
        .map_err(|error| Error::from_reason(error.to_string()))?
        .map_or_else(|| Ok(SourceType::tsx()), Ok)
}
