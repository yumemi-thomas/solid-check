package engine_test

import (
	"context"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
)

func TestNativeSessionProvesAsyncReadAndLoadingBoundaryRules(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "async-boundary")
	session, err := (engine.NativeEngine{
		OpenTypeFacts:     tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) { return asyncBoundaryAnalyzer{}, nil },
	}).OpenProject(context.Background(), engine.ProjectConfig{ConfigPath: filepath.Join(fixture, "tsconfig.json")})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })
	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	source, err := os.ReadFile(filepath.Join(fixture, "App.tsx"))
	if err != nil {
		t.Fatal(err)
	}
	goodWrapperStart := strings.Index(string(source), "export function GoodWrapperBoundary")
	badWrapperStart := strings.Index(string(source), "export function BadWrapperBoundary")
	goodWrapperFinding, badWrapperFinding := false, false
	want := map[string]int{
		"pending-async-untracked-read":   1,
		"pending-async-forbidden-scope":  1,
		"async-outside-loading-boundary": 7,
	}
	for _, finding := range snapshot.Findings {
		if finding.Rule == "async-outside-loading-boundary" && strings.Contains(finding.Message, "syncUser") {
			t.Errorf("sync computation classified async: %#v", finding)
		}
		if finding.Rule == "async-outside-loading-boundary" && finding.PrimaryLocation != nil {
			offset := finding.PrimaryLocation.StartByte
			goodWrapperFinding = goodWrapperFinding || offset >= goodWrapperStart && offset < badWrapperStart
			badWrapperFinding = badWrapperFinding || offset >= badWrapperStart
		}
		if _, ok := want[finding.Rule]; ok {
			want[finding.Rule]--
		}
	}
	for rule, remaining := range want {
		if remaining != 0 {
			t.Errorf("%s remaining = %d; findings = %#v", rule, remaining, snapshot.Findings)
		}
	}
	if goodWrapperFinding || !badWrapperFinding {
		t.Errorf("wrapper boundary findings: good=%t bad=%t; want good=false bad=true; findings=%#v", goodWrapperFinding, badWrapperFinding, snapshot.Findings)
	}
}

type asyncBoundaryAnalyzer struct{}

func (asyncBoundaryAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	tracked := make([]compilerfacts.ExecutionRegion, 0)
	for _, expression := range []string{"user().name", "fetchedUser().name", "promisedUser().name", "syncUser().name", "signalUser().name", "storeUser.name", "projectedUser.name"} {
		for offset := 0; ; {
			index := strings.Index(request.Source[offset:], expression)
			if index < 0 {
				break
			}
			start := offset + index
			if wrapperStart := strings.Index(request.Source, "export function GoodWrapperBoundary"); wrapperStart >= 0 && start > wrapperStart {
				offset = start + len(expression)
				continue
			}
			if start > 0 && request.Source[start-1] == '{' {
				tracked = append(tracked, compilerfacts.ExecutionRegion{Span: compilerfacts.Span{Start: start, End: start + len(expression)}, Reason: compilerfacts.RegionJSXChild})
			}
			offset = start + len(expression)
		}
	}
	sort.Slice(tracked, func(i, j int) bool { return tracked[i].Span.Start < tracked[j].Span.Start })
	operations := make([]compilerfacts.JsxOperation, 0)
	for _, name := range []string{"Loading", "Await", "LoadingWrapper", "WrongLoadingWrapper"} {
		for offset := 0; ; {
			index := strings.Index(request.Source[offset:], "<"+name)
			if index < 0 {
				break
			}
			start := offset + index
			closing := "</" + name + ">"
			end := strings.Index(request.Source[start:], closing) + start + len(closing)
			operations = append(operations, compilerfacts.JsxOperation{Span: compilerfacts.Span{Start: start, End: end}, Kind: "component-invocation"})
			if strings.HasSuffix(name, "LoadingWrapper") {
				if child := strings.Index(request.Source[start:end], "user().name"); child >= 0 {
					child += start
					operations = append(operations, compilerfacts.JsxOperation{Span: compilerfacts.Span{Start: child, End: child + len("user().name")}, Kind: "component-property"})
				}
			}
			offset = end
		}
	}
	profileStart := strings.Index(request.Source, "<Profile")
	if profileStart >= 0 {
		operations = append(operations, compilerfacts.JsxOperation{Span: compilerfacts.Span{Start: profileStart, End: profileStart + len("<Profile />")}, Kind: "component-invocation"})
	}
	sort.Slice(operations, func(i, j int) bool { return operations[i].Span.Start < operations[j].Span.Start })
	return compilerfacts.ExecutionMap{CompilerFactsProtocol: compilerfacts.ProtocolVersion, SourceHash: request.SourceHash, TrackedRegions: tracked, JsxOperations: operations}, nil
}
func (asyncBoundaryAnalyzer) Close() error { return nil }
