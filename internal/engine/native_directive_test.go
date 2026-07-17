package engine_test

import (
	"context"
	"path/filepath"
	"strings"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
)

func TestNativeSessionEnforcesDirectiveSetupAndApplicationPhases(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "directive-phases")
	session, err := (engine.NativeEngine{
		OpenTypeFacts:     tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) { return directiveAnalyzer{}, nil },
	}).OpenProject(context.Background(), engine.ProjectConfig{ConfigPath: filepath.Join(fixture, "tsconfig.json")})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })
	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	want := map[string]int{"reactive-write-in-owned-scope": 1, "primitive-in-directive-application": 3}
	for _, finding := range snapshot.Findings {
		if _, ok := want[finding.Rule]; ok {
			want[finding.Rule]--
		}
	}
	for rule, remaining := range want {
		if remaining != 0 {
			t.Errorf("%s remaining = %d; findings = %#v", rule, remaining, snapshot.Findings)
		}
	}
}

type directiveAnalyzer struct{}

func (directiveAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	expressions := []string{"directive()", "forwardedDirective()"}
	callbacks := make([]compilerfacts.CallbackRole, 0, 3)
	operations := make([]compilerfacts.JsxOperation, 0, 6)
	for _, expression := range expressions {
		start := strings.LastIndex(request.Source, expression)
		span := compilerfacts.Span{Start: start, End: start + len(expression)}
		callbacks = append(callbacks, compilerfacts.CallbackRole{Span: span, Role: compilerfacts.CallbackDirectiveApply})
		operations = append(operations,
			compilerfacts.JsxOperation{Span: span, Kind: "directive-apply"},
			compilerfacts.JsxOperation{Span: span, Kind: "directive-setup"},
		)
	}
	arrowStart := strings.LastIndex(request.Source, "element => {")
	arrowEnd := strings.Index(request.Source[arrowStart:], "}]}") + arrowStart + 1
	arrowSpan := compilerfacts.Span{Start: arrowStart, End: arrowEnd}
	callbacks = append(callbacks, compilerfacts.CallbackRole{Span: arrowSpan, Role: compilerfacts.CallbackDirectiveApply})
	operations = append(operations, compilerfacts.JsxOperation{Span: arrowSpan, Kind: "directive-apply"})
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion, SourceHash: request.SourceHash,
		CallbackRoles: callbacks,
		JsxOperations: operations,
	}, nil
}
func (directiveAnalyzer) Close() error { return nil }
