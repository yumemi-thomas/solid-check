use dom_expressions_jsx_compiler::{analyze_execution_map, transform, TransformOptions};

#[test]
fn analysis_only_execution_map_matches_full_transform() {
    let source = r#"
        import { createSignal, Show } from "solid-js";
        const [count] = createSignal(0);
        export function App() {
            return <Show when={count()}>{() => <button onClick={() => count()}>{count()}</button>}</Show>;
        }
    "#;
    let options = || TransformOptions {
        filename: Some("App.tsx".into()),
        module_name: Some("dom".into()),
        generate: Some("dom".into()),
        compiler_facts: Some(true),
        ..TransformOptions::default()
    };
    let analysis = analyze_execution_map(source, &options()).unwrap();
    let transformed = transform(source.into(), Some(options()))
        .unwrap()
        .execution_map
        .unwrap();
    let analysis: serde_json::Value = serde_json::from_str(&analysis).unwrap();
    let transformed: serde_json::Value = serde_json::from_str(&transformed).unwrap();
    assert_eq!(analysis, transformed);
}
