package solver_test

import (
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/internal/solver"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func TestStrictReadsReportsOnlyUntrackedRenderingReads(t *testing.T) {
	t.Parallel()

	declaration := certification.SourceLocation{Path: "/workspace/source.ts", StartByte: 14, EndByte: 19, Line: 1, Column: 15}
	program := reactiveir.Program{Reads: []reactiveir.ReactiveRead{
		{
			Accessor:    "count",
			Location:    certification.SourceLocation{Path: "/workspace/App.tsx", StartByte: 48, EndByte: 55, Line: 2, Column: 16},
			Declaration: declaration,
			Execution:   reactiveir.ExecutionTrackedJSX,
			Context:     "Good",
		},
		{
			Accessor:    "count",
			Location:    certification.SourceLocation{Path: "/workspace/App.tsx", StartByte: 92, EndByte: 99, Line: 5, Column: 17},
			Declaration: declaration,
			Execution:   reactiveir.ExecutionUntrackedRendering,
			Context:     "Bad",
		},
		{
			Accessor:    "count",
			Location:    certification.SourceLocation{Path: "/workspace/App.tsx", StartByte: 150, EndByte: 157, Line: 9, Column: 28},
			Declaration: declaration,
			Execution:   reactiveir.ExecutionDeferredCallback,
			Context:     "Events",
		},
	}}

	findings := solver.StrictReads(program)
	if len(findings) != 1 {
		t.Fatalf("findings = %#v, want one untracked read", findings)
	}
	finding := findings[0]
	if finding.Rule != "strict-read-untracked" || finding.Kind != certification.FindingViolation {
		t.Fatalf("finding = %#v", finding)
	}
	if finding.PrimaryLocation == nil || finding.PrimaryLocation.StartByte != 92 {
		t.Fatalf("primary location = %#v", finding.PrimaryLocation)
	}
	if len(finding.RelatedLocations) != 1 || finding.RelatedLocations[0] != declaration {
		t.Fatalf("related locations = %#v", finding.RelatedLocations)
	}
	if len(finding.Evidence) != 3 {
		t.Fatalf("evidence = %#v, want declaration, provenance, and execution steps", finding.Evidence)
	}
}

func TestStrictReadsReturnsNoFindingForTrackedAndDeferredReads(t *testing.T) {
	t.Parallel()

	program := reactiveir.Program{Reads: []reactiveir.ReactiveRead{
		{Accessor: "count", Execution: reactiveir.ExecutionTrackedJSX},
		{Accessor: "count", Execution: reactiveir.ExecutionDeferredCallback},
	}}
	if findings := solver.StrictReads(program); len(findings) != 0 {
		t.Fatalf("findings = %#v, want none", findings)
	}
}
