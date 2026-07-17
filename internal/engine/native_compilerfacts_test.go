package engine_test

import (
	"context"
	"errors"
	"path/filepath"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/typefacts"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func TestNativeSessionAnalyzesChangedTSXWithCompilerFacts(t *testing.T) {
	analyzer := &recordingAnalyzer{}
	session, err := (engine.NativeEngine{
		OpenTypeFacts: func(context.Context, string) (typefacts.Project, error) {
			return &fakeTypeFacts{}, nil
		},
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return analyzer, nil
		},
	}).OpenProject(context.Background(), engine.ProjectConfig{ConfigPath: "tsconfig.json"})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })

	source := []byte("const view = <div>{count()}</div>;")
	if _, err := session.Update(context.Background(), []engine.FileChange{
		{Path: "/workspace/plain.ts", Version: 1, Source: []byte("export {}")},
		{Path: "/workspace/App.tsx", Version: 1, Source: source},
	}); err != nil {
		t.Fatal(err)
	}
	if len(analyzer.requests) != 1 {
		t.Fatalf("compiler requests = %d, want 1 changed TSX request", len(analyzer.requests))
	}
	if analyzer.requests[0].Path != "/workspace/App.tsx" {
		t.Errorf("compiler request path = %q", analyzer.requests[0].Path)
	}
	if analyzer.requests[0].SourceHash != compilerfacts.HashSource(source) {
		t.Errorf("compiler request source hash = %q", analyzer.requests[0].SourceHash)
	}

	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	if snapshot.Status != certification.StatusCertified {
		t.Fatalf("snapshot status = %q, want certified", snapshot.Status)
	}
	if len(snapshot.Findings) != 0 {
		t.Fatalf("snapshot findings = %#v, want none", snapshot.Findings)
	}
	if snapshot.Metrics.FilesAnalyzed != 1 {
		t.Fatalf("files analyzed = %d, want 1", snapshot.Metrics.FilesAnalyzed)
	}
}

func TestNativeSessionCanonicalizesUpdatePathsBeforeForwarding(t *testing.T) {
	facts := &fakeTypeFacts{}
	session, err := (engine.NativeEngine{
		OpenTypeFacts: func(context.Context, string) (typefacts.Project, error) {
			return facts, nil
		},
	}).OpenProject(context.Background(), engine.ProjectConfig{ConfigPath: "tsconfig.json"})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })

	relativePath := filepath.Join("testdata", "App.ts")
	if _, err := session.Update(context.Background(), []engine.FileChange{{
		Path: relativePath, Version: 1, Source: []byte("export {}"),
	}}); err != nil {
		t.Fatal(err)
	}
	want, err := filepath.Abs(relativePath)
	if err != nil {
		t.Fatal(err)
	}
	if len(facts.changes) != 1 || facts.changes[0].Path != want {
		t.Fatalf("forwarded changes = %#v, want canonical path %q", facts.changes, want)
	}
}

type recordingAnalyzer struct {
	requests []compilerfacts.AnalysisRequest
	closed   bool
}

func (a *recordingAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	a.requests = append(a.requests, request)
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
	}, nil
}

func (a *recordingAnalyzer) Close() error {
	a.closed = true
	return nil
}

type fakeTypeFacts struct {
	changes []typefacts.FileChange
}

func (*fakeTypeFacts) SourceFiles(context.Context) ([]typefacts.SourceFile, error) {
	return []typefacts.SourceFile{}, nil
}

func (f *fakeTypeFacts) Update(_ context.Context, changes []typefacts.FileChange) (typefacts.AffectedSet, error) {
	f.changes = append([]typefacts.FileChange(nil), changes...)
	paths := make([]string, len(changes))
	for index, change := range changes {
		paths[index] = change.Path
	}
	return typefacts.AffectedSet{Files: paths}, nil
}

func (*fakeTypeFacts) SymbolAt(context.Context, typefacts.Location) (typefacts.SymbolID, error) {
	return "", typefacts.ErrNotFound
}
func (*fakeTypeFacts) ResolveAlias(context.Context, typefacts.SymbolID) (typefacts.SymbolID, error) {
	return "", errors.New("unused")
}
func (*fakeTypeFacts) Declarations(context.Context, typefacts.SymbolID) ([]typefacts.Declaration, error) {
	return nil, errors.New("unused")
}
func (*fakeTypeFacts) References(context.Context, typefacts.SymbolID) ([]typefacts.Location, error) {
	return nil, errors.New("unused")
}
func (*fakeTypeFacts) TypeAt(context.Context, typefacts.Location) (typefacts.TypeID, error) {
	return "", errors.New("unused")
}
func (*fakeTypeFacts) ResolvedCall(context.Context, typefacts.Location) (typefacts.Call, error) {
	return typefacts.Call{}, errors.New("unused")
}
func (*fakeTypeFacts) Close() error { return nil }
