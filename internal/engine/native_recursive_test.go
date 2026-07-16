package engine_test

import (
	"context"
	"path/filepath"
	"strings"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func TestNativeSessionSolvesRecursiveFunctionSummaries(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "recursive")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return recursiveAnalyzer{}, nil
		},
	}).OpenProject(context.Background(), engine.ProjectConfig{
		ConfigPath: filepath.Join(fixture, "tsconfig.json"),
	})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })

	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	if snapshot.Status != certification.StatusViolation {
		t.Fatalf("status = %q, want violation; metrics = %#v", snapshot.Status, snapshot.Metrics)
	}
	if len(snapshot.Findings) != 1 || !strings.Contains(snapshot.Findings[0].Message, "readA") {
		t.Fatalf("findings = %#v, want one recursive readA violation", snapshot.Findings)
	}
}

type recursiveAnalyzer struct{}

func (recursiveAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	tracked := make([]compilerfacts.ExecutionRegion, 0)
	needle := "{readA(2)}"
	if index := strings.Index(request.Source, needle); index >= 0 {
		start := index + 1
		tracked = append(tracked, compilerfacts.ExecutionRegion{
			Span:   compilerfacts.Span{Start: start, End: start + len("readA(2)")},
			Reason: compilerfacts.RegionJSXChild,
		})
	}
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
		TrackedRegions:        tracked,
	}, nil
}

func (recursiveAnalyzer) Close() error { return nil }
