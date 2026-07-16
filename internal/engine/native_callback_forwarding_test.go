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

func TestNativeSessionPropagatesForwardedCallbackEffects(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "callback-forwarding")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return callbackForwardingAnalyzer{}, nil
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
	if len(snapshot.Findings) != 1 || !strings.Contains(snapshot.Findings[0].Message, "invoke") {
		t.Fatalf("findings = %#v, want one forwarded invoke violation", snapshot.Findings)
	}
	if len(snapshot.Findings[0].Evidence) < 2 || !strings.Contains(snapshot.Findings[0].Evidence[1].Message, "readCount") {
		t.Fatalf("evidence = %#v, want readCount as the underlying reader", snapshot.Findings[0].Evidence)
	}
}

type callbackForwardingAnalyzer struct{}

func (callbackForwardingAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	tracked := make([]compilerfacts.ExecutionRegion, 0)
	expression := "invoke(readCount)"
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

func (callbackForwardingAnalyzer) Close() error { return nil }
