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

func TestNativeSessionDistinguishesComputeAndImperativeReadPhases(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "execution-phases")
	session, err := (engine.NativeEngine{
		OpenTypeFacts:     tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) { return executionPhaseAnalyzer{}, nil },
	}).OpenProject(context.Background(), engine.ProjectConfig{ConfigPath: filepath.Join(fixture, "tsconfig.json")})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })
	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	strictReads := 0
	for _, finding := range snapshot.Findings {
		if finding.Rule == "strict-read-untracked" {
			strictReads++
			if finding.AnalysisContext != "createEffect apply callback" {
				t.Errorf("analysis context = %q, want createEffect apply callback", finding.AnalysisContext)
			}
		}
	}
	if strictReads != 1 {
		t.Fatalf("strict read findings = %d, want only the effect-apply read; findings = %#v", strictReads, snapshot.Findings)
	}
}

type executionPhaseAnalyzer struct{}

func (executionPhaseAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	expression := "count()"
	start := strings.LastIndex(request.Source, expression)
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
		TrackedRegions: []compilerfacts.ExecutionRegion{{
			Span: compilerfacts.Span{Start: start, End: start + len(expression)}, Reason: compilerfacts.RegionJSXChild,
		}},
	}, nil
}
func (executionPhaseAnalyzer) Close() error { return nil }
