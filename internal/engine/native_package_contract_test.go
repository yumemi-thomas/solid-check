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

func TestNativeSessionConsumesPackageContractWithoutImplementationSource(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "package-consumer")
	contractPath := filepath.Join(fixture, "node_modules", "reactive-package", "solid-reactivity.json")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return packageContractAnalyzer{}, nil
		},
	}).OpenProject(context.Background(), engine.ProjectConfig{
		ConfigPath:    filepath.Join(fixture, "tsconfig.json"),
		ContractPaths: []string{contractPath},
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
	if len(snapshot.Findings) != 1 || !strings.Contains(snapshot.Findings[0].Message, "readCount") {
		t.Fatalf("findings = %#v, want contracted readCount violation", snapshot.Findings)
	}
	if len(snapshot.PackageSummaries) != 1 || snapshot.PackageSummaries[0].Name != "reactive-package" {
		t.Fatalf("package summaries = %#v", snapshot.PackageSummaries)
	}
}

func TestNativeSessionConsumesReturnedAccessorFromPackageContract(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "package-return-consumer")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return returnedAccessorAnalyzer{}, nil
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
	if snapshot.Status != certification.StatusViolation || len(snapshot.Findings) != 1 {
		t.Fatalf("snapshot = %#v, want one untracked returned-accessor read", snapshot)
	}
	if !strings.Contains(snapshot.Findings[0].Message, "created count") {
		t.Fatalf("finding = %#v, want returned-accessor contract provenance", snapshot.Findings[0])
	}
}

func TestNativeSessionConsumesCallbackExecutionFromPackageContract(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "package-callback-consumer")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return emptyPackageContractAnalyzer{}, nil
		},
	}).OpenProject(context.Background(), engine.ProjectConfig{
		ConfigPath: filepath.Join(fixture, "tsconfig.json"),
		ContractPaths: []string{
			filepath.Join(fixture, "node_modules", "reactive-package", "solid-reactivity.json"),
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })

	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	if snapshot.Status != certification.StatusViolation || len(snapshot.Findings) != 1 {
		t.Fatalf("snapshot = %#v, want only inline callback violation", snapshot)
	}
	if !strings.Contains(snapshot.Findings[0].Message, "runInline") {
		t.Fatalf("finding = %#v, want runInline provenance", snapshot.Findings[0])
	}
}

func TestNativeSessionAutomaticallyUsesBundledSolidContract(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "bundled-solid-consumer")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return expressionPackageContractAnalyzer{expression: "doubled()"}, nil
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
	if snapshot.Status != certification.StatusViolation || len(snapshot.Findings) != 1 {
		t.Fatalf("snapshot = %#v, want one untracked memo-result read", snapshot)
	}
	if len(snapshot.PackageSummaries) != 1 || snapshot.PackageSummaries[0].Name != "solid-js" {
		t.Fatalf("package summaries = %#v, want bundled solid-js evidence", snapshot.PackageSummaries)
	}
}

func TestNativeSessionConsumesReturnedStoreFromPackageContract(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "package-store-consumer")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return expressionPackageContractAnalyzer{expression: "state.value"}, nil
		},
	}).OpenProject(context.Background(), engine.ProjectConfig{
		ConfigPath: filepath.Join(fixture, "tsconfig.json"),
		ContractPaths: []string{
			filepath.Join(fixture, "node_modules", "reactive-package", "solid-reactivity.json"),
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })

	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	if snapshot.Status != certification.StatusViolation || len(snapshot.Findings) != 1 {
		t.Fatalf("snapshot = %#v, want one untracked returned-store read", snapshot)
	}
	if !strings.Contains(snapshot.Findings[0].Message, "state.value") {
		t.Fatalf("finding = %#v, want nested store path", snapshot.Findings[0])
	}
}

func TestNativeSessionRejectsImportMissingFromPackageContract(t *testing.T) {
	fixture := filepath.Join("..", "reactiveir", "testdata", "package-unknown-export")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return emptyPackageContractAnalyzer{}, nil
		},
	}).OpenProject(context.Background(), engine.ProjectConfig{
		ConfigPath: filepath.Join(fixture, "tsconfig.json"),
		ContractPaths: []string{
			filepath.Join(fixture, "node_modules", "reactive-package", "solid-reactivity.json"),
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })

	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	if snapshot.Status != certification.StatusUncertifiable || len(snapshot.Findings) != 1 {
		t.Fatalf("snapshot = %#v, want one uncertifiable missing-export obligation", snapshot)
	}
	if !strings.Contains(snapshot.Findings[0].Message, "unknownPrimitive") {
		t.Fatalf("finding = %#v, want missing export name", snapshot.Findings[0])
	}
}

type packageContractAnalyzer struct{}

func (packageContractAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	tracked := make([]compilerfacts.ExecutionRegion, 0)
	expression := "readCount()"
	if index := strings.Index(request.Source, expression); index >= 0 {
		tracked = append(tracked, compilerfacts.ExecutionRegion{
			Span:   compilerfacts.Span{Start: index, End: index + len(expression)},
			Reason: compilerfacts.RegionJSXChild,
		})
	}
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
		TrackedRegions:        tracked,
	}, nil
}

func (packageContractAnalyzer) Close() error { return nil }

type returnedAccessorAnalyzer struct{}

func (returnedAccessorAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	tracked := make([]compilerfacts.ExecutionRegion, 0)
	expression := "count()"
	if index := strings.Index(request.Source, expression); index >= 0 {
		tracked = append(tracked, compilerfacts.ExecutionRegion{
			Span:   compilerfacts.Span{Start: index, End: index + len(expression)},
			Reason: compilerfacts.RegionJSXChild,
		})
	}
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
		TrackedRegions:        tracked,
	}, nil
}

func (returnedAccessorAnalyzer) Close() error { return nil }

type emptyPackageContractAnalyzer struct{}

func (emptyPackageContractAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
	}, nil
}

func (emptyPackageContractAnalyzer) Close() error { return nil }

type expressionPackageContractAnalyzer struct{ expression string }

func (analyzer expressionPackageContractAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	tracked := make([]compilerfacts.ExecutionRegion, 0)
	if index := strings.Index(request.Source, analyzer.expression); index >= 0 {
		tracked = append(tracked, compilerfacts.ExecutionRegion{
			Span:   compilerfacts.Span{Start: index, End: index + len(analyzer.expression)},
			Reason: compilerfacts.RegionJSXChild,
		})
	}
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
		TrackedRegions:        tracked,
	}, nil
}

func (expressionPackageContractAnalyzer) Close() error { return nil }
