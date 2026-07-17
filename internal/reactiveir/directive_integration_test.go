package reactiveir_test

import (
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
)

func TestBuildPreservesDirectiveReturnedClosurePhases(t *testing.T) {
	fixture := filepath.Join("testdata", "directive-phases")
	project, err := tsgo.OpenProject(context.Background(), filepath.Join(fixture, "tsconfig.json"))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = project.Close() })
	sources, _ := project.SourceFiles(context.Background())
	path, _ := filepath.Abs(filepath.Join(fixture, "App.tsx"))
	source, _ := os.ReadFile(path)
	start := strings.LastIndex(string(source), "directive()")
	program, err := reactiveir.Build(context.Background(), project, sources, map[string]compilerfacts.ExecutionMap{path: {
		CallbackRoles: []compilerfacts.CallbackRole{{Span: compilerfacts.Span{Start: start, End: start + len("directive()")}, Role: compilerfacts.CallbackDirectiveApply}},
	}})
	if err != nil {
		t.Fatal(err)
	}
	for _, function := range program.Functions {
		if function.Name == "directive" {
			if len(function.Writes) != 2 || !function.Writes[1].InReturnedClosure || len(function.PrimitiveCreations) != 1 || !function.PrimitiveCreations[0].InReturnedClosure {
				t.Fatalf("directive function = %#v; all functions = %#v", function, program.Functions)
			}
		}
		if function.Name == "App" && len(function.Calls) == 0 {
			t.Fatalf("App calls = %#v; all functions = %#v", function.Calls, program.Functions)
		}
	}
}
