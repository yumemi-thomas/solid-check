package packagecontracts

import (
	"errors"

	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/internal/solver"
	"github.com/yumemi-thomas/solid-check/pkg/contracts"
)

type EmitOptions struct {
	Package               contracts.PackageIdentity
	CompilerFactsProtocol int
	Artifacts             contracts.Artifacts
}

func Emit(program reactiveir.Program, options EmitOptions) (contracts.Contract, error) {
	if options.Package.Name == "" {
		return contracts.Contract{}, errors.New("emit package contract: package name is required")
	}
	if len(program.Unresolved) != 0 {
		return contracts.Contract{}, errors.New("emit package contract: unresolved effect: " + program.Unresolved[0].Message)
	}
	summaries := solver.FunctionReadSummaries(program)
	exports := make(map[string]contracts.ExportSummary)
	for _, name := range program.ExportedValues {
		exports[name] = contracts.ExportSummary{Kind: "value"}
	}
	summariesByID := make(map[reactiveir.FunctionID]contracts.ExportSummary, len(program.Functions))
	for _, function := range program.Functions {
		reads := summaries[function.ID]
		contractReads := make([]contracts.ReactiveRead, 0, len(reads))
		for _, read := range reads {
			kind := "accessor"
			if read.Kind == reactiveir.ReactiveStorePath {
				kind = "store-path"
			}
			contractReads = append(contractReads, contracts.ReactiveRead{Kind: kind, Label: read.Accessor})
		}
		summary := contracts.ExportSummary{Kind: "function", ReactiveReads: contractReads}
		if function.Async {
			summary.AsyncBehavior = "promise"
		}
		if len(function.ReturnedReads) != 0 {
			returned := function.ReturnedReads[0]
			kind := "accessor"
			if returned.Kind == reactiveir.ReactiveStorePath {
				kind = "store-path"
			}
			summary.Returns = &contracts.ReactiveReturn{Kind: kind, Label: returned.Accessor}
		}
		for _, invocation := range function.CallbackInvocations {
			execution := "inline"
			if invocation.Execution == reactiveir.ExecutionTrackedJSX {
				execution = "tracked"
			} else if invocation.Execution == reactiveir.ExecutionDeferredCallback {
				execution = "deferred"
			}
			summary.Callbacks = append(summary.Callbacks, contracts.CallbackSummary{
				Parameter: invocation.Parameter, Execution: execution,
			})
		}
		summariesByID[function.ID] = summary
		if function.Exported {
			exports[function.Name] = summary
		}
	}
	for name, id := range program.ExportAliases {
		if summary, ok := summariesByID[id]; ok {
			exports[name] = summary
		} else {
			exports[name] = contracts.ExportSummary{Kind: "value"}
		}
	}
	contract := contracts.Contract{
		SchemaVersion:         contracts.SchemaVersion,
		Package:               options.Package,
		CompilerFactsProtocol: options.CompilerFactsProtocol,
		Artifacts:             options.Artifacts,
		Exports:               exports,
		Evidence:              contracts.Evidence{Kind: "generated", Generator: "solid-check"},
	}
	if err := contract.Validate(); err != nil {
		return contracts.Contract{}, err
	}
	return contract, nil
}
