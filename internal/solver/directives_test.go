package solver_test

import (
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/internal/solver"
)

func TestDirectiveApplicationObligationsCountReturnedClosurePrimitives(t *testing.T) {
	t.Parallel()

	program := reactiveir.Program{Functions: []reactiveir.Function{
		{
			ID: "component", Rendering: true,
			Calls: []reactiveir.FunctionCall{{Target: "directive", Execution: reactiveir.ExecutionDirectiveApply}},
		},
		{
			ID: "directive",
			PrimitiveCreations: []reactiveir.PrimitiveCreation{
				{Primitive: "createSignal"},
				{Primitive: "createMemo", InReturnedClosure: true},
			},
		},
	}}

	if got := solver.DirectiveApplicationObligations(program); got != 1 {
		t.Fatalf("directive application obligations = %d, want 1", got)
	}
}
