package compilerfacts_test

import (
	"strings"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
)

func TestValidateAcceptsFactsForTheRequestedSource(t *testing.T) {
	t.Parallel()

	source := []byte("const view = <div>{count()}</div>;")
	start := strings.Index(string(source), "count()")
	request := compilerfacts.NewRequest("/workspace/App.tsx", source, compilerfacts.CompilerOptions{
		ModuleName: "dom",
		Generate:   "dom",
	})
	facts := compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
		TrackedRegions: []compilerfacts.ExecutionRegion{{
			Span:   compilerfacts.Span{Start: start, End: start + len("count()")},
			Reason: compilerfacts.RegionJSXChild,
		}},
	}

	if err := compilerfacts.Validate(request, facts); err != nil {
		t.Fatalf("Validate() error = %v", err)
	}
}

func TestValidateRejectsStaleOrStructurallyInvalidFacts(t *testing.T) {
	t.Parallel()

	source := []byte("const view = <div>{count()}</div>;")
	request := compilerfacts.NewRequest("/workspace/App.tsx", source, compilerfacts.CompilerOptions{})

	tests := []struct {
		name  string
		facts compilerfacts.ExecutionMap
		want  string
	}{
		{
			name: "protocol mismatch",
			facts: compilerfacts.ExecutionMap{
				CompilerFactsProtocol: compilerfacts.ProtocolVersion + 1,
				SourceHash:            request.SourceHash,
			},
			want: "protocol",
		},
		{
			name: "source hash mismatch",
			facts: compilerfacts.ExecutionMap{
				CompilerFactsProtocol: compilerfacts.ProtocolVersion,
				SourceHash:            "sha256:stale",
			},
			want: "source hash",
		},
		{
			name: "span outside source",
			facts: compilerfacts.ExecutionMap{
				CompilerFactsProtocol: compilerfacts.ProtocolVersion,
				SourceHash:            request.SourceHash,
				CallbackRoles: []compilerfacts.CallbackRole{{
					Span: compilerfacts.Span{Start: 0, End: len(source) + 1},
					Role: compilerfacts.CallbackEventHandler,
				}},
			},
			want: "span",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			err := compilerfacts.Validate(request, test.facts)
			if err == nil || !strings.Contains(err.Error(), test.want) {
				t.Fatalf("Validate() error = %v, want error containing %q", err, test.want)
			}
		})
	}
}

func TestNewRequestHashesTheExactUTF8Bytes(t *testing.T) {
	t.Parallel()

	first := compilerfacts.NewRequest("App.tsx", []byte("const label = '東京';"), compilerfacts.CompilerOptions{})
	second := compilerfacts.NewRequest("App.tsx", []byte("const label = '東京!';"), compilerfacts.CompilerOptions{})

	if !strings.HasPrefix(first.SourceHash, "sha256:") {
		t.Fatalf("SourceHash = %q, want sha256 prefix", first.SourceHash)
	}
	if first.SourceHash == second.SourceHash {
		t.Fatal("different UTF-8 source bytes produced the same source hash")
	}
}
