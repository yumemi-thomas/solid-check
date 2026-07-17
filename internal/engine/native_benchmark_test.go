package engine_test

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
)

var benchmarkSnapshotStatus string

func BenchmarkNativeEngineColdSnapshot(b *testing.B) {
	b.ReportAllocs()
	for b.Loop() {
		session := openBenchmarkSession(b)
		snapshot, err := session.Snapshot(context.Background(), nil)
		if err != nil {
			_ = session.Close()
			b.Fatal(err)
		}
		benchmarkSnapshotStatus = string(snapshot.Status)
		if err := session.Close(); err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkNativeEngineSnapshot(b *testing.B) {
	session := openBenchmarkSession(b)
	b.Cleanup(func() { _ = session.Close() })

	b.ReportAllocs()
	b.ResetTimer()
	for b.Loop() {
		snapshot, err := session.Snapshot(context.Background(), nil)
		if err != nil {
			b.Fatal(err)
		}
		benchmarkSnapshotStatus = string(snapshot.Status)
	}
}

func BenchmarkNativeEngineIncrementalUpdate(b *testing.B) {
	session := openBenchmarkSession(b)
	b.Cleanup(func() { _ = session.Close() })
	path, variants := benchmarkEdit(b)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; b.Loop(); i++ {
		if _, err := session.Update(context.Background(), []engine.FileChange{{
			Path: path, Version: uint64(i + 1), Source: variants[i%len(variants)],
		}}); err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkNativeEngineIncrementalSnapshot(b *testing.B) {
	session := openBenchmarkSession(b)
	b.Cleanup(func() { _ = session.Close() })
	path, variants := benchmarkEdit(b)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; b.Loop(); i++ {
		if _, err := session.Update(context.Background(), []engine.FileChange{{
			Path: path, Version: uint64(i + 1), Source: variants[i%len(variants)],
		}}); err != nil {
			b.Fatal(err)
		}
		snapshot, err := session.Snapshot(context.Background(), nil)
		if err != nil {
			b.Fatal(err)
		}
		benchmarkSnapshotStatus = string(snapshot.Status)
	}
}

func openBenchmarkSession(b *testing.B) engine.ProjectSession {
	b.Helper()
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return benchmarkAnalyzer{}, nil
		},
	}).OpenProject(context.Background(), engine.ProjectConfig{
		ConfigPath: filepath.Join("testdata", "eslint-reactivity-v2", "tsconfig.json"),
	})
	if err != nil {
		b.Fatal(err)
	}
	return session
}

func benchmarkEdit(b *testing.B) (string, [2][]byte) {
	b.Helper()
	path := filepath.Join("testdata", "eslint-reactivity-v2", "component-props-read.tsx")
	source, err := os.ReadFile(path)
	if err != nil {
		b.Fatal(err)
	}
	return path, [2][]byte{source, append(append([]byte(nil), source...), '\n')}
}

type benchmarkAnalyzer struct{}

func (benchmarkAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	return compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
	}, nil
}

func (benchmarkAnalyzer) Close() error { return nil }
