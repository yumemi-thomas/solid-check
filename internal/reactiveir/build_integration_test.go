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

func TestBuildJoinsCrossFileSignalProvenanceWithCompilerRegions(t *testing.T) {
	fixture := filepath.Join("testdata", "tracer")
	project, err := tsgo.OpenProject(context.Background(), filepath.Join(fixture, "tsconfig.json"))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = project.Close() })
	sources, err := project.SourceFiles(context.Background())
	if err != nil {
		t.Fatal(err)
	}

	appPath, err := filepath.Abs(filepath.Join(fixture, "App.tsx"))
	if err != nil {
		t.Fatal(err)
	}
	appSource, err := os.ReadFile(appPath)
	if err != nil {
		t.Fatal(err)
	}
	goodStart := strings.Index(string(appSource), "count()")
	eventCallback := "() => count()"
	eventStart := strings.LastIndex(string(appSource), eventCallback)
	maps := map[string]compilerfacts.ExecutionMap{
		appPath: {
			CompilerFactsProtocol: compilerfacts.ProtocolVersion,
			SourceHash:            compilerfacts.HashSource(appSource),
			TrackedRegions: []compilerfacts.ExecutionRegion{{
				Span:   compilerfacts.Span{Start: goodStart, End: goodStart + len("count()")},
				Reason: compilerfacts.RegionJSXChild,
			}},
			CallbackRoles: []compilerfacts.CallbackRole{{
				Span: compilerfacts.Span{Start: eventStart, End: eventStart + len(eventCallback)},
				Role: compilerfacts.CallbackEventHandler,
			}},
		},
	}

	program, err := reactiveir.Build(context.Background(), project, sources, maps)
	if err != nil {
		t.Fatal(err)
	}
	if len(program.Reads) != 3 {
		t.Fatalf("reads = %#v, want tracked, untracked, and deferred reads", program.Reads)
	}
	wantRoles := []reactiveir.ExecutionRole{
		reactiveir.ExecutionTrackedJSX,
		reactiveir.ExecutionUntrackedRendering,
		reactiveir.ExecutionDeferredCallback,
	}
	wantContexts := []string{"Good", "Bad", "Events"}
	for index := range wantRoles {
		if program.Reads[index].Execution != wantRoles[index] {
			t.Errorf("read %d role = %q, want %q", index, program.Reads[index].Execution, wantRoles[index])
		}
		if program.Reads[index].Context != wantContexts[index] {
			t.Errorf("read %d context = %q, want %q", index, program.Reads[index].Context, wantContexts[index])
		}
		if filepath.Base(program.Reads[index].Declaration.Path) != "source.ts" {
			t.Errorf("read %d declaration = %#v", index, program.Reads[index].Declaration)
		}
	}
}
