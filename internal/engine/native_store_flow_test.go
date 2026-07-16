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

func TestNativeSessionPropagatesStorePropertyReads(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "store-flow")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return storeFlowAnalyzer{}, nil
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
		t.Fatalf("status = %q, want violation", snapshot.Status)
	}
	if len(snapshot.Findings) != 1 || !strings.Contains(snapshot.Findings[0].Message, "state.count") {
		t.Fatalf("findings = %#v, want one propagated store-property violation", snapshot.Findings)
	}
	if !strings.Contains(snapshot.Findings[0].Message, "store path") {
		t.Fatalf("message = %q, want store-path terminology", snapshot.Findings[0].Message)
	}
}

type storeFlowAnalyzer struct{}

func (storeFlowAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	tracked := make([]compilerfacts.ExecutionRegion, 0)
	expression := "readCount()"
	if index := strings.Index(request.Source, expression); index >= 0 {
		tracked = append(tracked, compilerfacts.ExecutionRegion{
			Span:   compilerfacts.Span{Start: index, End: index + len(expression)},
			Reason: compilerfacts.RegionJSXChild,
		})
	}
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
		TrackedRegions:        tracked,
	}, nil
}

func (storeFlowAnalyzer) Close() error { return nil }
