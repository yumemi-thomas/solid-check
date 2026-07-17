package reactiveir_test

import (
	"context"
	"path/filepath"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
)

func TestBuildClassifiesInvalidCleanupReturns(t *testing.T) {
	fixture := filepath.Join("testdata", "leaf-owner")
	project, err := tsgo.OpenProject(context.Background(), filepath.Join(fixture, "tsconfig.json"))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = project.Close() })
	sources, err := project.SourceFiles(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	program, err := reactiveir.Build(context.Background(), project, sources, nil)
	if err != nil {
		t.Fatal(err)
	}
	if len(program.InvalidCleanupReturns) != 6 || len(program.Unresolved) != 0 {
		t.Fatalf("invalid cleanup returns = %#v; unresolved = %#v", program.InvalidCleanupReturns, program.Unresolved)
	}
}
