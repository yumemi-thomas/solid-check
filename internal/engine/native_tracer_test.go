package engine_test

import (
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func TestNativeSessionReportsCrossFileUntrackedSignalRead(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "tracer")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return tracerAnalyzer{}, nil
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
	var strictRead *certification.Finding
	for index := range snapshot.Findings {
		if snapshot.Findings[index].Rule == "strict-read-untracked" {
			strictRead = &snapshot.Findings[index]
			break
		}
	}
	if strictRead == nil {
		t.Fatalf("findings = %#v, want strict-read-untracked", snapshot.Findings)
	}
	if strictRead.PrimaryLocation == nil || filepath.Base(strictRead.PrimaryLocation.Path) != "App.tsx" {
		t.Fatalf("primary location = %#v", strictRead.PrimaryLocation)
	}
	if len(strictRead.RelatedLocations) != 1 || filepath.Base(strictRead.RelatedLocations[0].Path) != "source.ts" {
		t.Fatalf("related locations = %#v", strictRead.RelatedLocations)
	}
	if len(strictRead.Evidence) != 3 {
		t.Fatalf("evidence = %#v", strictRead.Evidence)
	}

	appPath := filepath.Join(fixture, "App.tsx")
	source, err := os.ReadFile(appPath)
	if err != nil {
		t.Fatal(err)
	}
	corrected := strings.Replace(
		string(source),
		"  const value = count();\n  return <div>{value}</div>;",
		"  return <div>{count()}</div>;",
		1,
	)
	if corrected == string(source) {
		t.Fatal("failed to construct corrected tracer fixture")
	}
	if _, err := session.Update(context.Background(), []engine.FileChange{{
		Path: appPath, Version: 1, Source: []byte(corrected),
	}}); err != nil {
		t.Fatal(err)
	}
	correctedSnapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	if correctedSnapshot.Status != certification.StatusCertified {
		t.Fatalf("corrected status = %q, want certified", correctedSnapshot.Status)
	}
	if len(correctedSnapshot.Findings) != 0 {
		t.Fatalf("corrected findings = %#v, want none", correctedSnapshot.Findings)
	}
	if correctedSnapshot.Metrics.UnresolvedObligations != 0 {
		t.Fatalf("corrected unresolved obligations = %d, want 0", correctedSnapshot.Metrics.UnresolvedObligations)
	}
}

type tracerAnalyzer struct{}

func (tracerAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	tracked := make([]compilerfacts.ExecutionRegion, 0)
	remaining := request.Source
	base := 0
	for {
		index := strings.Index(remaining, "count()")
		if index < 0 {
			break
		}
		start := base + index
		if start > 0 && request.Source[start-1] == '{' {
			tracked = append(tracked, compilerfacts.ExecutionRegion{
				Span:   compilerfacts.Span{Start: start, End: start + len("count()")},
				Reason: compilerfacts.RegionJSXChild,
			})
		}
		base = start + len("count()")
		remaining = request.Source[base:]
	}
	callbackText := "() => count()"
	callbackStart := strings.LastIndex(request.Source, callbackText)
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
		TrackedRegions:        tracked,
		CallbackRoles: []compilerfacts.CallbackRole{{
			Span: compilerfacts.Span{Start: callbackStart, End: callbackStart + len(callbackText)},
			Role: compilerfacts.CallbackEventHandler,
		}},
	}, nil
}

func (tracerAnalyzer) Close() error { return nil }
