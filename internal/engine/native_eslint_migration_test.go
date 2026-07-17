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

func TestNativeEngineMigratesReactivityV2SemanticSourceFixtures(t *testing.T) {
	fixture := filepath.Join("testdata", "eslint-reactivity-v2")
	session, err := (engine.NativeEngine{
		OpenTypeFacts: tsgo.OpenProject,
		OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) {
			return eslintMigrationAnalyzer{}, nil
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

	want := map[string]string{
		"leaf-reexport-flush.tsx":                   "flush-in-forbidden-scope",
		"leaf-reexport-cleanup.tsx":                 "cleanup-in-forbidden-scope",
		"owned-reexport-memo.tsx":                   "reactive-write-in-owned-scope",
		"owned-reexport-refresh.tsx":                "reactive-write-in-owned-scope",
		"owned-reexport-action.tsx":                 "action-called-in-owned-scope",
		"effect-apply-parameter.tsx":                "strict-read-untracked",
		"effect-apply-member.tsx":                   "strict-read-untracked",
		"after-await-member.tsx":                    "reactive-read-after-await",
		"after-await-parameter.tsx":                 "reactive-read-after-await",
		"after-await-namespace.tsx":                 "reactive-read-after-await",
		"after-await-callback-before-read.tsx":      "reactive-read-after-await",
		"after-await-named-callback.tsx":            "reactive-read-after-await",
		"after-await-aliased-callback.tsx":          "reactive-read-after-await",
		"after-await-same-expression.tsx":           "reactive-read-after-await",
		"after-await-both-branches.tsx":             "reactive-read-after-await",
		"after-await-try-finally.tsx":               "reactive-read-after-await",
		"imported-after-await-definition.ts":        "reactive-read-after-await",
		"component-props-read.tsx":                  "strict-read-untracked",
		"component-props-alias.tsx":                 "strict-read-untracked",
		"component-props-merge-alias.tsx":           "strict-read-untracked",
		"component-reactive-early-return.tsx":       "strict-read-untracked",
		"component-reactive-conditional-return.tsx": "strict-read-untracked",
		"component-props-parameter-destructure.tsx": "component-props-destructure",
		"component-props-body-destructure.tsx":      "component-props-destructure",
		"derived-signal-in-effect.tsx":              "reactive-write-in-owned-scope",
	}
	soundNegatives := map[string]string{
		"effect-apply-plain-function.tsx":      "strict-read-untracked",
		"effect-apply-structural-store.tsx":    "strict-read-untracked",
		"after-await-plain-function.tsx":       "reactive-read-after-await",
		"after-await-local-accessor.tsx":       "reactive-read-after-await",
		"before-await-accessor.tsx":            "reactive-read-after-await",
		"conditional-await-accessor.tsx":       "reactive-read-after-await",
		"nested-after-await-accessor.tsx":      "reactive-read-after-await",
		"loop-await-accessor.tsx":              "reactive-read-after-await",
		"component-props-tracked.tsx":          "strict-read-untracked",
		"noncomponent-object-read.ts":          "strict-read-untracked",
		"noncomponent-object-destructure.ts":   "component-props-destructure",
		"component-props-passthrough.tsx":      "strict-read-untracked",
		"component-props-local-merge.tsx":      "strict-read-untracked",
		"component-props-unknown-callback.tsx": "strict-read-untracked",
		"component-static-early-return.tsx":    "strict-read-untracked",
		"signal-write-in-effect-apply.tsx":     "reactive-write-in-owned-scope",
	}
	complexDestructureFindings := 0
	for _, finding := range snapshot.Findings {
		if finding.PrimaryLocation == nil {
			continue
		}
		name := filepath.Base(finding.PrimaryLocation.Path)
		if rule, ok := want[name]; ok && finding.Rule == rule {
			if (name == "component-reactive-early-return.tsx" || name == "component-reactive-conditional-return.tsx") && finding.AnalysisContext != "Card conditional return" {
				continue
			}
			if name == "derived-signal-in-effect.tsx" && finding.AnalysisContext != "createEffect compute" {
				continue
			}
			delete(want, name)
		}
		if absentRule, soundNegative := soundNegatives[name]; soundNegative && finding.Rule == absentRule {
			t.Errorf("sound-negative fixture %s produced %#v", name, finding)
		}
		if name == "component-props-parameter-destructure.tsx" && finding.Rule == "component-props-destructure" {
			if len(finding.Fixes) != 1 || finding.Fixes[0].Applicability != certification.FixSafe {
				t.Errorf("parameter destructure fix = %#v, want one safe fix", finding.Fixes)
			} else if finding.Fixes[0].Message != "Fix: Keep component props reactive" {
				t.Errorf("parameter destructure fix title = %q", finding.Fixes[0].Message)
			} else {
				edits := finding.Fixes[0].Edits
				if len(edits) != 2 || edits[0].NewText != "props" || edits[1].NewText != "props.title" {
					t.Errorf("parameter destructure edits = %#v", edits)
				}
			}
		}
		if name == "component-props-parameter-complex-destructure.tsx" && finding.Rule == "component-props-destructure" {
			complexDestructureFindings++
			if len(finding.Fixes) != 0 {
				t.Errorf("complex destructure unexpectedly has a fix: %#v", finding.Fixes)
			}
		}
	}
	if complexDestructureFindings != 3 {
		t.Errorf("complex destructure findings = %d, want 3", complexDestructureFindings)
	}
	for name, rule := range want {
		t.Errorf("%s missing canonical finding %s; snapshot = %#v", name, rule, snapshot.Findings)
	}
}

type eslintMigrationAnalyzer struct{}

func (eslintMigrationAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	result := compilerfacts.ExecutionMap{CompilerFactsProtocol: compilerfacts.ProtocolVersion, SourceHash: request.SourceHash}
	if filepath.Base(request.Path) == "component-props-parameter-destructure.tsx" {
		start := strings.Index(request.Source, "{title}")
		if start >= 0 {
			result.JsxOperations = []compilerfacts.JsxOperation{{
				Span: compilerfacts.Span{Start: start + 1, End: start + len("{title}") - 1},
				Kind: "jsx-expression",
			}}
		}
	}
	return result, nil
}
func (eslintMigrationAnalyzer) Close() error { return nil }
