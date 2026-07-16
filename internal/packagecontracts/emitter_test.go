package packagecontracts_test

import (
	"encoding/json"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/packagecontracts"
	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/pkg/contracts"
)

func TestEmitSerializesSolvedEffectsForExportedFunctions(t *testing.T) {
	program := reactiveir.Program{Functions: []reactiveir.Function{
		{
			ID: "read-count", Name: "readCount", Exported: true,
			Reads: []reactiveir.ReactiveRead{{
				Kind: reactiveir.ReactiveAccessor, Accessor: "count", Execution: reactiveir.ExecutionInline,
			}},
		},
		{
			ID: "private-helper", Name: "privateHelper",
			Reads: []reactiveir.ReactiveRead{{
				Kind: reactiveir.ReactiveAccessor, Accessor: "count", Execution: reactiveir.ExecutionInline,
			}},
		},
	}}

	contract, err := packagecontracts.Emit(program, packagecontracts.EmitOptions{
		Package:               contracts.PackageIdentity{Name: "reactive-package", Version: "1.0.0"},
		CompilerFactsProtocol: 1,
	})
	if err != nil {
		t.Fatal(err)
	}
	if len(contract.Exports) != 1 {
		t.Fatalf("exports = %#v, want only exported readCount", contract.Exports)
	}
	summary := contract.Exports["readCount"]
	if len(summary.ReactiveReads) != 1 || summary.ReactiveReads[0].Label != "count" {
		t.Fatalf("readCount summary = %#v", summary)
	}
}

func TestEmitSerializesReturnedAccessor(t *testing.T) {
	program := reactiveir.Program{Functions: []reactiveir.Function{{
		ID: "create-count", Name: "createCount", Exported: true,
		ReturnedReads: []reactiveir.ReactiveRead{{
			Kind: reactiveir.ReactiveAccessor, Accessor: "count", Execution: reactiveir.ExecutionInline,
		}},
	}}}

	contract, err := packagecontracts.Emit(program, packagecontracts.EmitOptions{
		Package: contracts.PackageIdentity{Name: "reactive-package"},
	})
	if err != nil {
		t.Fatal(err)
	}
	data, err := json.Marshal(contract)
	if err != nil {
		t.Fatal(err)
	}
	var document map[string]any
	if err := json.Unmarshal(data, &document); err != nil {
		t.Fatal(err)
	}
	exports := document["exports"].(map[string]any)
	summary := exports["createCount"].(map[string]any)
	returned, ok := summary["returns"].(map[string]any)
	if !ok || returned["kind"] != "accessor" {
		t.Fatalf("createCount summary = %#v, want returned accessor", summary)
	}
}

func TestEmitSerializesCallbackExecution(t *testing.T) {
	program := reactiveir.Program{Functions: []reactiveir.Function{{
		ID: "track", Name: "track", Exported: true,
		CallbackInvocations: []reactiveir.CallbackInvocation{{
			Parameter: 0, Execution: reactiveir.ExecutionTrackedJSX,
		}},
	}}}

	contract, err := packagecontracts.Emit(program, packagecontracts.EmitOptions{
		Package: contracts.PackageIdentity{Name: "reactive-package"},
	})
	if err != nil {
		t.Fatal(err)
	}
	data, err := json.Marshal(contract)
	if err != nil {
		t.Fatal(err)
	}
	var document map[string]any
	if err := json.Unmarshal(data, &document); err != nil {
		t.Fatal(err)
	}
	summary := document["exports"].(map[string]any)["track"].(map[string]any)
	callbacks, ok := summary["callbacks"].([]any)
	if !ok || len(callbacks) != 1 || callbacks[0].(map[string]any)["execution"] != "tracked" {
		t.Fatalf("track summary = %#v, want tracked parameter 0", summary)
	}
}

func TestEmitRejectsUnresolvedPackageEffects(t *testing.T) {
	program := reactiveir.Program{
		Functions:  []reactiveir.Function{{ID: "value", Name: "value", Exported: true}},
		Unresolved: []reactiveir.UnresolvedObligation{{Message: "missing dependency export"}},
	}

	_, err := packagecontracts.Emit(program, packagecontracts.EmitOptions{
		Package: contracts.PackageIdentity{Name: "reactive-package"},
	})
	if err == nil {
		t.Fatal("Emit() succeeded with unresolved package effects")
	}
}
