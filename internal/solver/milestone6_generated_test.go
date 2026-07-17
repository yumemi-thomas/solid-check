package solver_test

import (
	"fmt"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/internal/solver"
)

// TestGeneratedMilestone6ExecutionRoleConformance generates the role matrix
// independently of the hand-written source fixtures. It guards the solver's
// positive and negative classification for every execution category that can
// change write, action, async, or directive legality.
func TestGeneratedMilestone6ExecutionRoleConformance(t *testing.T) {
	t.Parallel()

	allowedRoles := []string{"event-handler", "action", "untracked-callback", "on-settled", "tracked-effect", "effect-apply", "directive-apply", "owned-write-option"}
	for _, allowedBy := range append([]string{""}, allowedRoles...) {
		allowedBy := allowedBy
		t.Run("write/"+roleName(allowedBy), func(t *testing.T) {
			program := reactiveir.Program{Writes: []reactiveir.ReactiveWrite{{Setter: "setValue", AllowedBy: allowedBy}}}
			got := len(solver.SignalWrites(program))
			want := 1
			if allowedBy != "" {
				want = 0
			}
			if got != want {
				t.Fatalf("write findings = %d, want %d", got, want)
			}
		})
	}

	for _, allowedBy := range append([]string{""}, allowedRoles[:7]...) {
		allowedBy := allowedBy
		t.Run("action/"+roleName(allowedBy), func(t *testing.T) {
			program := reactiveir.Program{ActionCalls: []reactiveir.ActionInvocation{{Action: "save", AllowedBy: allowedBy}}}
			got := len(solver.ActionCalls(program))
			want := 1
			if allowedBy != "" {
				want = 0
			}
			if got != want {
				t.Fatalf("action findings = %d, want %d", got, want)
			}
		})
	}

	asyncCases := []struct {
		name         string
		read         reactiveir.AsyncRead
		wantRule     string
		wantFindings int
	}{
		{"tracked-compute", reactiveir.AsyncRead{Execution: reactiveir.ExecutionTrackedComputation}, "", 0},
		{"tracked-jsx-loading", reactiveir.AsyncRead{Execution: reactiveir.ExecutionTrackedJSX, UnderLoading: true}, "", 0},
		{"tracked-jsx-no-loading", reactiveir.AsyncRead{Execution: reactiveir.ExecutionTrackedJSX}, "async-outside-loading-boundary", 1},
		{"untracked", reactiveir.AsyncRead{Execution: reactiveir.ExecutionUntrackedRendering}, "pending-async-untracked-read", 1},
		{"leaf", reactiveir.AsyncRead{Execution: reactiveir.ExecutionTrackedComputation, LeafOwner: "createTrackedEffect"}, "pending-async-forbidden-scope", 1},
	}
	for _, test := range asyncCases {
		test := test
		t.Run("async/"+test.name, func(t *testing.T) {
			findings := solver.AsyncBoundaries(reactiveir.Program{AsyncReads: []reactiveir.AsyncRead{test.read}})
			if len(findings) != test.wantFindings {
				t.Fatalf("async findings = %#v", findings)
			}
			if len(findings) == 1 && findings[0].Rule != test.wantRule {
				t.Fatalf("async rule = %q, want %q", findings[0].Rule, test.wantRule)
			}
		})
	}

	for _, returned := range []bool{false, true} {
		returned := returned
		t.Run(fmt.Sprintf("directive/returned=%t", returned), func(t *testing.T) {
			program := reactiveir.Program{Functions: []reactiveir.Function{
				{ID: "component", Rendering: true, Calls: []reactiveir.FunctionCall{{Target: "directive", Execution: reactiveir.ExecutionDirectiveApply}}},
				{ID: "directive", PrimitiveCreations: []reactiveir.PrimitiveCreation{{Primitive: "createSignal", InReturnedClosure: returned}}},
			}}
			got := len(solver.DirectiveApplications(program))
			want := 0
			if returned {
				want = 1
			}
			if got != want {
				t.Fatalf("directive findings = %d, want %d", got, want)
			}
		})
	}
}

func roleName(role string) string {
	if role == "" {
		return "owned"
	}
	return role
}
