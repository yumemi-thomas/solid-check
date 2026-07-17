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
	request := validRequest(source)

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

func TestValidateRequestRejectsUnsupportedOrMalformedInputs(t *testing.T) {
	t.Parallel()
	valid := validRequest([]byte("const view = <div />;"))
	tests := []struct {
		name   string
		mutate func(*compilerfacts.AnalysisRequest)
		want   string
	}{
		{name: "protocol", mutate: func(request *compilerfacts.AnalysisRequest) { request.CompilerFactsProtocol++ }, want: "protocol"},
		{name: "path", mutate: func(request *compilerfacts.AnalysisRequest) { request.Path = "" }, want: "path"},
		{name: "module", mutate: func(request *compilerfacts.AnalysisRequest) { request.CompilerOptions.ModuleName = "" }, want: "moduleName"},
		{name: "generate", mutate: func(request *compilerfacts.AnalysisRequest) { request.CompilerOptions.Generate = "ssr" }, want: "DOM"},
		{name: "hash", mutate: func(request *compilerfacts.AnalysisRequest) { request.SourceHash = "sha256:stale" }, want: "source hash"},
		{name: "UTF-8", mutate: func(request *compilerfacts.AnalysisRequest) {
			request.Source = string([]byte{0xff})
			request.SourceHash = compilerfacts.HashSource([]byte(request.Source))
		}, want: "UTF-8"},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			request := valid
			test.mutate(&request)
			if err := compilerfacts.ValidateRequest(request); err == nil || !strings.Contains(err.Error(), test.want) {
				t.Fatalf("ValidateRequest() error = %v, want %q", err, test.want)
			}
		})
	}
}

func TestValidateRejectsUnknownKindsSplitUTF8AndNondeterministicFacts(t *testing.T) {
	t.Parallel()
	source := []byte("const 東京 = '😀'; const view = <div>{東京}</div>;")
	request := validRequest(source)
	valid := compilerfacts.ExecutionMap{
		CompilerFactsProtocol: compilerfacts.ProtocolVersion,
		SourceHash:            request.SourceHash,
	}
	tests := []struct {
		name   string
		mutate func(*compilerfacts.ExecutionMap)
		want   string
	}{
		{name: "unknown region", mutate: func(facts *compilerfacts.ExecutionMap) {
			facts.TrackedRegions = []compilerfacts.ExecutionRegion{{Span: compilerfacts.Span{Start: 0, End: 1}, Reason: "future-region"}}
		}, want: "reason"},
		{name: "unknown callback", mutate: func(facts *compilerfacts.ExecutionMap) {
			facts.CallbackRoles = []compilerfacts.CallbackRole{{Span: compilerfacts.Span{Start: 0, End: 1}, Role: "future-callback"}}
		}, want: "role"},
		{name: "unknown operation", mutate: func(facts *compilerfacts.ExecutionMap) {
			facts.JsxOperations = []compilerfacts.JsxOperation{{Span: compilerfacts.Span{Start: 0, End: 1}, Kind: "future-operation"}}
		}, want: "kind"},
		{name: "split UTF-8", mutate: func(facts *compilerfacts.ExecutionMap) {
			start := strings.Index(string(source), "東")
			facts.TrackedRegions = []compilerfacts.ExecutionRegion{{Span: compilerfacts.Span{Start: start + 1, End: start + len("東京")}, Reason: compilerfacts.RegionJSXChild}}
		}, want: "UTF-8"},
		{name: "unsorted", mutate: func(facts *compilerfacts.ExecutionMap) {
			facts.TrackedRegions = []compilerfacts.ExecutionRegion{
				{Span: compilerfacts.Span{Start: 3, End: 4}, Reason: compilerfacts.RegionJSXChild},
				{Span: compilerfacts.Span{Start: 1, End: 2}, Reason: compilerfacts.RegionJSXAttribute},
			}
		}, want: "sorted"},
		{name: "duplicate", mutate: func(facts *compilerfacts.ExecutionMap) {
			facts.CallbackRoles = []compilerfacts.CallbackRole{
				{Span: compilerfacts.Span{Start: 1, End: 2}, Role: compilerfacts.CallbackEventHandler},
				{Span: compilerfacts.Span{Start: 1, End: 2}, Role: compilerfacts.CallbackEventHandler},
			}
		}, want: "duplicate"},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			facts := valid
			test.mutate(&facts)
			if err := compilerfacts.Validate(request, facts); err == nil || !strings.Contains(err.Error(), test.want) {
				t.Fatalf("Validate() error = %v, want %q", err, test.want)
			}
		})
	}
}

func TestUncoveredJSXExpressionsEnforcesTheCompletenessInvariant(t *testing.T) {
	t.Parallel()

	expression := compilerfacts.Span{Start: 19, End: 26}
	operation := []compilerfacts.JsxOperation{{Span: expression, Kind: "jsx-expression"}}
	tests := []struct {
		name  string
		facts compilerfacts.ExecutionMap
		want  int
	}{
		{
			name:  "uncovered expression is reported",
			facts: compilerfacts.ExecutionMap{JsxOperations: operation},
			want:  1,
		},
		{
			name: "tracked region covers",
			facts: compilerfacts.ExecutionMap{
				TrackedRegions: []compilerfacts.ExecutionRegion{{Span: expression, Reason: compilerfacts.RegionJSXChild}},
				JsxOperations:  operation,
			},
		},
		{
			name: "untracked region covers",
			facts: compilerfacts.ExecutionMap{
				UntrackedRegions: []compilerfacts.ExecutionRegion{{Span: expression, Reason: compilerfacts.RegionJSXChild}},
				JsxOperations:    operation,
			},
		},
		{
			name: "callback role covers",
			facts: compilerfacts.ExecutionMap{
				CallbackRoles: []compilerfacts.CallbackRole{{Span: expression, Role: compilerfacts.CallbackEventHandler}},
				JsxOperations: operation,
			},
		},
		{
			name: "component-property operation covers",
			facts: compilerfacts.ExecutionMap{
				JsxOperations: []compilerfacts.JsxOperation{
					{Span: expression, Kind: "component-property"},
					{Span: expression, Kind: "jsx-expression"},
				},
			},
		},
		{
			name: "region overlapping but not containing does not cover",
			facts: compilerfacts.ExecutionMap{
				UntrackedRegions: []compilerfacts.ExecutionRegion{{
					Span:   compilerfacts.Span{Start: expression.Start + 1, End: expression.End},
					Reason: compilerfacts.RegionJSXChild,
				}},
				JsxOperations: operation,
			},
			want: 1,
		},
		{
			name: "non-expression operations carry no obligation",
			facts: compilerfacts.ExecutionMap{
				JsxOperations: []compilerfacts.JsxOperation{{Span: expression, Kind: "insert"}},
			},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			uncovered := compilerfacts.UncoveredJSXExpressions(test.facts)
			if len(uncovered) != test.want {
				t.Fatalf("UncoveredJSXExpressions() = %v, want %d spans", uncovered, test.want)
			}
			if test.want == 1 && uncovered[0] != expression {
				t.Fatalf("uncovered span = %v, want %v", uncovered[0], expression)
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

func TestNewRequestCanonicalizesBuiltInsAndValidationRejectsAmbiguousLists(t *testing.T) {
	t.Parallel()
	request := compilerfacts.NewRequest("App.tsx", []byte("const view = <div />;"), compilerfacts.CompilerOptions{
		ModuleName: "dom",
		Generate:   "dom",
		BuiltIns:   []string{"Show", "For"},
	})
	if got := strings.Join(request.CompilerOptions.BuiltIns, ","); got != "For,Show" {
		t.Fatalf("builtIns = %q, want canonical order", got)
	}
	request.CompilerOptions.BuiltIns = []string{"For", "For"}
	if err := compilerfacts.ValidateRequest(request); err == nil || !strings.Contains(err.Error(), "duplicate") {
		t.Fatalf("duplicate builtIns error = %v", err)
	}
}

func validRequest(source []byte) compilerfacts.AnalysisRequest {
	return compilerfacts.NewRequest("/workspace/App.tsx", source, compilerfacts.CompilerOptions{
		ModuleName: "dom",
		Generate:   "dom",
	})
}
