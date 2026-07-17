package engine_test

import (
	"context"
	"path/filepath"
	"sort"
	"strings"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func TestNativeSessionReportsOperationsCreatedWithoutOwner(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "owner-presence")
	session, err := (engine.NativeEngine{
		OpenTypeFacts:     tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) { return ownerPresenceAnalyzer{}, nil },
	}).OpenProject(context.Background(), engine.ProjectConfig{ConfigPath: filepath.Join(fixture, "tsconfig.json")})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })
	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	want := map[string]int{"no-owner-effect": 7, "no-owner-cleanup": 2, "no-owner-boundary": 3, "settled-cleanup-unowned": 2}
	for _, finding := range snapshot.Findings {
		if _, ok := want[finding.Rule]; ok {
			want[finding.Rule]--
			if finding.Rule != "settled-cleanup-unowned" && finding.Kind == certification.FindingViolation && finding.Severity != certification.SeverityWarning {
				t.Errorf("%s severity = %q", finding.Rule, finding.Severity)
			}
		}
	}
	for rule, remaining := range want {
		if remaining != 0 {
			t.Errorf("%s remaining = %d; findings = %#v", rule, remaining, snapshot.Findings)
		}
	}
	uncertain := 0
	for _, finding := range snapshot.Findings {
		if finding.Rule == "no-owner-effect" && finding.Kind == certification.FindingUncertifiable {
			uncertain++
		}
	}
	if uncertain != 1 {
		t.Errorf("uncertain exported owner findings = %d", uncertain)
	}
}

type ownerPresenceAnalyzer struct{}

func (ownerPresenceAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	operations := make([]compilerfacts.JsxOperation, 0)
	for _, name := range []string{"Loading", "Await"} {
		remaining, base := request.Source, 0
		for {
			index := strings.Index(remaining, "<"+name)
			if index < 0 {
				break
			}
			start := base + index
			closing := "</" + name + ">"
			end := strings.Index(request.Source[start:], closing) + start + len(closing)
			operations = append(operations, compilerfacts.JsxOperation{Span: compilerfacts.Span{Start: start, End: end}, Kind: "component-invocation"})
			base, remaining = end, request.Source[end:]
		}
	}
	sort.Slice(operations, func(i, j int) bool { return operations[i].Span.Start < operations[j].Span.Start })
	callbackText := "() => createEffect(() => 1, () => {})"
	callbackStart := strings.Index(request.Source, callbackText)
	namedCallback := "eventHelper"
	namedStart := strings.LastIndex(request.Source, namedCallback)
	callbacks := []compilerfacts.CallbackRole{
		{Span: compilerfacts.Span{Start: callbackStart, End: callbackStart + len(callbackText)}, Role: compilerfacts.CallbackEventHandler},
		{Span: compilerfacts.Span{Start: namedStart, End: namedStart + len(namedCallback)}, Role: compilerfacts.CallbackEventHandler},
	}
	return compilerfacts.ExecutionMap{CompilerFactsProtocol: compilerfacts.ProtocolVersion, SourceHash: request.SourceHash, JsxOperations: operations, CallbackRoles: callbacks}, nil
}
func (ownerPresenceAnalyzer) Close() error { return nil }
