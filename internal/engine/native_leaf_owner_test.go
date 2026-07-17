package engine_test

import (
	"context"
	"path/filepath"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func TestNativeSessionRejectsForbiddenOperationsInLeafOwners(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "leaf-owner")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return emptyExecutionAnalyzer{}, nil
		},
	}).OpenProject(context.Background(), engine.ProjectConfig{ConfigPath: filepath.Join(fixture, "tsconfig.json")})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })
	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	if snapshot.Status != certification.StatusViolation {
		t.Fatalf("status = %q", snapshot.Status)
	}
	want := map[string]int{
		"cleanup-in-forbidden-scope": 3,
		"primitive-in-leaf-owner":    3,
		"flush-in-forbidden-scope":   2,
		"invalid-cleanup-return":     6,
	}
	for _, finding := range snapshot.Findings {
		if _, ok := want[finding.Rule]; ok {
			want[finding.Rule]--
			if finding.Severity != certification.SeverityError || finding.PrimaryLocation == nil {
				t.Fatalf("finding = %#v", finding)
			}
		}
	}
	for rule, remaining := range want {
		if remaining != 0 {
			t.Errorf("%s remaining = %d; findings = %#v", rule, remaining, snapshot.Findings)
		}
	}
	fixes := 0
	for _, finding := range snapshot.Findings {
		for _, fix := range finding.Fixes {
			fixes++
			if fix.Applicability != certification.FixSafe || len(fix.Edits) != 1 || fix.Edits[0].NewText != `return () => console.log("disposed")` {
				t.Errorf("fix = %#v", fix)
			}
		}
	}
	if fixes != 1 {
		t.Errorf("safe fixes = %d, want 1", fixes)
	}
	unresolvedCleanup := 0
	for _, finding := range snapshot.Findings {
		if finding.Rule == "cleanup-return-unresolved" {
			unresolvedCleanup++
		}
	}
	if unresolvedCleanup != 0 {
		t.Errorf("unresolved cleanup returns = %d, want local cleanup identifier proved as a function", unresolvedCleanup)
	}
}

type emptyExecutionAnalyzer struct{}

func (emptyExecutionAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	return compilerfacts.ExecutionMap{CompilerFactsProtocol: compilerfacts.ProtocolVersion, SourceHash: request.SourceHash}, nil
}
func (emptyExecutionAnalyzer) Close() error { return nil }
