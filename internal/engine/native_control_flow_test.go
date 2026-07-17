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
)

func TestNativeSessionTreatsControlFlowCallbackBodyAsUntrackedRendering(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "control-flow")
	session, err := (engine.NativeEngine{
		OpenTypeFacts:     tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) { return controlFlowAnalyzer{}, nil },
	}).OpenProject(context.Background(), engine.ProjectConfig{ConfigPath: filepath.Join(fixture, "tsconfig.json")})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })
	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	count := 0
	parameter := 0
	for _, finding := range snapshot.Findings {
		if finding.Rule == "strict-read-untracked" {
			if strings.Contains(finding.Message, "count") {
				count++
			}
			if strings.Contains(finding.Message, "value") {
				parameter++
			}
		}
	}
	if count != 1 {
		t.Fatalf("control-flow strict reads = %d, want only direct callback-body read; findings = %#v", count, snapshot.Findings)
	}
	if parameter != 1 {
		t.Fatalf("control-flow parameter strict reads = %d, want only direct callback-body read; findings = %#v", parameter, snapshot.Findings)
	}
}

type controlFlowAnalyzer struct{}

func (controlFlowAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	callbacks := make([]compilerfacts.CallbackRole, 0)
	operations := make([]compilerfacts.JsxOperation, 0)
	for offset := 0; ; {
		show := strings.Index(request.Source[offset:], "<Show")
		if show < 0 {
			break
		}
		showStart := offset + show
		showEnd := strings.Index(request.Source[showStart:], "</Show>") + showStart + len("</Show>")
		arrow := strings.Index(request.Source[showStart:showEnd], "=>")
		if arrow >= 0 {
			arrowAt := showStart + arrow
			callbackStart := arrowAt
			for callbackStart > showStart && request.Source[callbackStart-1] != '{' {
				callbackStart--
			}
			callbacks = append(callbacks, compilerfacts.CallbackRole{Span: compilerfacts.Span{Start: callbackStart, End: showEnd - len("</Show>")}, Role: compilerfacts.CallbackRender})
		}
		operations = append(operations, compilerfacts.JsxOperation{Span: compilerfacts.Span{Start: showStart, End: showEnd}, Kind: "component-invocation"})
		offset = showEnd
	}
	tracked := make([]compilerfacts.ExecutionRegion, 0)
	for _, expression := range []string{"count()", "value()"} {
		for offset := 0; ; {
			index := strings.Index(request.Source[offset:], expression)
			if index < 0 {
				break
			}
			start := offset + index
			if start > 0 && request.Source[start-1] == '{' {
				tracked = append(tracked, compilerfacts.ExecutionRegion{Span: compilerfacts.Span{Start: start, End: start + len(expression)}, Reason: compilerfacts.RegionJSXChild})
			}
			offset = start + len(expression)
		}
	}
	sort.Slice(tracked, func(i, j int) bool { return tracked[i].Span.Start < tracked[j].Span.Start })
	return compilerfacts.ExecutionMap{CompilerFactsProtocol: compilerfacts.ProtocolVersion, SourceHash: request.SourceHash, TrackedRegions: tracked, CallbackRoles: callbacks, JsxOperations: operations}, nil
}
func (controlFlowAnalyzer) Close() error { return nil }
