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

func TestNativeSessionRejectsOwnedSignalWriteButAllowsEventWrite(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "write-scope")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return writeScopeAnalyzer{}, nil
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
	var writes []certification.Finding
	for _, finding := range snapshot.Findings {
		if finding.Rule == "reactive-write-in-owned-scope" {
			writes = append(writes, finding)
		}
	}
	if len(writes) != 12 {
		t.Fatalf("write findings = %#v, want canonical, aliased, namespaced, memo, effect-compute, and reachable helper writes", writes)
	}
	for _, write := range writes {
		if write.ID != "SC2001" || write.Severity != certification.SeverityError {
			t.Fatalf("write finding = %#v", write)
		}
	}
	actions := 0
	for _, finding := range snapshot.Findings {
		if finding.Rule == "action-called-in-owned-scope" {
			actions++
		}
	}
	if actions != 3 {
		t.Fatalf("owned action findings = %d, want component, helper, and memo callback calls; findings = %#v", actions, snapshot.Findings)
	}
	if writes[0].PrimaryLocation == nil || filepath.Base(writes[0].PrimaryLocation.Path) != "App.tsx" {
		t.Fatalf("primary location = %#v", writes[0].PrimaryLocation)
	}
}

type writeScopeAnalyzer struct{}

func (writeScopeAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	callback := "() => { setCount(previous => previous + 1); save(); }"
	start := strings.Index(request.Source, callback)
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
		CallbackRoles: []compilerfacts.CallbackRole{{
			Span: compilerfacts.Span{Start: start, End: start + len(callback)},
			Role: compilerfacts.CallbackEventHandler,
		}},
	}, nil
}

func (writeScopeAnalyzer) Close() error { return nil }
