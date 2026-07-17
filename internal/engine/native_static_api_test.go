package engine_test

import (
	"context"
	"path/filepath"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
)

func TestNativeSessionReportsStaticallyInvalidSolidAPIShapes(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "static-api")
	session, err := (engine.NativeEngine{
		OpenTypeFacts:     tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) { return emptyExecutionAnalyzer{}, nil },
	}).OpenProject(context.Background(), engine.ProjectConfig{ConfigPath: filepath.Join(fixture, "tsconfig.json")})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })
	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	want := map[string]int{
		"missing-effect-function":       2,
		"sync-node-received-async":      6,
		"invalid-refresh-target":        2,
		"invalid-affects-target":        2,
		"reactive-write-in-owned-scope": 1,
	}
	for _, finding := range snapshot.Findings {
		if _, ok := want[finding.Rule]; ok {
			want[finding.Rule]--
		}
	}
	for rule, remaining := range want {
		if remaining != 0 {
			t.Errorf("%s remaining = %d; findings = %#v", rule, remaining, snapshot.Findings)
		}
	}
}
