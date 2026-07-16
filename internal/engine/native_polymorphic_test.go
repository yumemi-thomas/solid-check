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

func TestNativeSessionInstantiatesGenericAndOverloadedSummaries(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "polymorphic")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return polymorphicAnalyzer{}, nil
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
	if len(snapshot.Findings) != 2 {
		t.Fatalf("findings = %#v, want generic and overloaded violations", snapshot.Findings)
	}
	messages := snapshot.Findings[0].Message + snapshot.Findings[1].Message
	if !strings.Contains(messages, "readGeneric") || !strings.Contains(messages, "overloaded") {
		t.Fatalf("messages = %q, want both helper names", messages)
	}
}

type polymorphicAnalyzer struct{}

func (polymorphicAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	tracked := make([]compilerfacts.ExecutionRegion, 0)
	for _, expression := range []string{`readGeneric("safe")`, "overloaded(1)"} {
		if index := strings.Index(request.Source, expression); index >= 0 {
			tracked = append(tracked, compilerfacts.ExecutionRegion{
				Span:   compilerfacts.Span{Start: index, End: index + len(expression)},
				Reason: compilerfacts.RegionJSXChild,
			})
		}
	}
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
		TrackedRegions:        tracked,
	}, nil
}

func (polymorphicAnalyzer) Close() error { return nil }
