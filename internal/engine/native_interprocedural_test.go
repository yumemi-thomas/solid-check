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

func TestNativeSessionPropagatesCrossFileHelperReads(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "interprocedural")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return interproceduralAnalyzer{}, nil
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
	if len(snapshot.Findings) != 1 || snapshot.Findings[0].Rule != "strict-read-untracked" {
		t.Fatalf("findings = %#v, want one propagated strict-read violation", snapshot.Findings)
	}
	finding := snapshot.Findings[0]
	if finding.PrimaryLocation == nil || filepath.Base(finding.PrimaryLocation.Path) != "App.tsx" {
		t.Fatalf("primary location = %#v, want the untracked helper call in App.tsx", finding.PrimaryLocation)
	}
	if !strings.Contains(finding.Message, "readCount") {
		t.Fatalf("message = %q, want helper name", finding.Message)
	}
	if snapshot.Metrics.FunctionsAnalyzed != 3 {
		t.Fatalf("functions analyzed = %d, want 3", snapshot.Metrics.FunctionsAnalyzed)
	}
	if snapshot.Metrics.ProofObligations != 2 {
		t.Fatalf("proof obligations = %d, want tracked and untracked helper instantiations", snapshot.Metrics.ProofObligations)
	}

	sourcePath := filepath.Join(fixture, "source.ts")
	source, err := os.ReadFile(sourcePath)
	if err != nil {
		t.Fatal(err)
	}
	updated := strings.Replace(string(source), "return count();", "return 1;", 1)
	if updated == string(source) {
		t.Fatal("failed to remove helper reactive read")
	}
	if _, err := session.Update(context.Background(), []engine.FileChange{{
		Path: sourcePath, Version: 1, Source: []byte(updated),
	}}); err != nil {
		t.Fatal(err)
	}
	updatedSnapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	if updatedSnapshot.Status != certification.StatusCertified {
		t.Fatalf("updated status = %q, want dependency-invalidated certification", updatedSnapshot.Status)
	}
}

type interproceduralAnalyzer struct{}

func (interproceduralAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	tracked := make([]compilerfacts.ExecutionRegion, 0)
	needle := "{readCount()}"
	for remaining, base := request.Source, 0; ; {
		index := strings.Index(remaining, needle)
		if index < 0 {
			break
		}
		start := base + index + 1
		tracked = append(tracked, compilerfacts.ExecutionRegion{
			Span:   compilerfacts.Span{Start: start, End: start + len("readCount()")},
			Reason: compilerfacts.RegionJSXChild,
		})
		base = start + len("readCount()")
		remaining = request.Source[base:]
	}
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
		TrackedRegions:        tracked,
	}, nil
}

func (interproceduralAnalyzer) Close() error { return nil }
