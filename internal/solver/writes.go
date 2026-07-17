package solver

import (
	"fmt"

	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func SignalWrites(program reactiveir.Program) []certification.Finding {
	writes := append([]reactiveir.ReactiveWrite{}, program.Writes...)
	functions := make(map[reactiveir.FunctionID]reactiveir.Function, len(program.Functions))
	for _, function := range program.Functions {
		functions[function.ID] = function
	}
	for _, function := range program.Functions {
		if function.Rendering {
			writes = append(writes, reachableWrites(function, functions, map[reactiveir.FunctionID]bool{}, "", false)...)
		}
	}
	for _, call := range program.ModuleCalls {
		if !call.Owned {
			continue
		}
		if target, ok := functions[call.Target]; ok {
			writes = append(writes, reachableWrites(target, functions, map[reactiveir.FunctionID]bool{}, "", false)...)
		}
	}
	findings := make([]certification.Finding, 0)
	for _, write := range writes {
		if write.AllowedBy != "" {
			continue
		}
		primary := write.Location
		declaration := write.Declaration
		context := write.Context
		if context == "" {
			context = "owned scope"
		}
		operation := fmt.Sprintf("signal setter %q", write.Setter)
		provenance := fmt.Sprintf("%q is the setter returned by createSignal or createStore", write.Setter)
		if write.Setter == "refresh" {
			operation = "refresh()"
			provenance = "the refresh target is a proven Solid source accessor or store"
		}
		findings = append(findings, certification.Finding{
			ID: "SC2001", Rule: "reactive-write-in-owned-scope",
			Kind: certification.FindingViolation, Severity: certification.SeverityError,
			Message: fmt.Sprintf("%s is called inside owned scope %s; move the write to an event handler, action, onSettled, tracked effect, or untracked callback", operation, context), AnalysisContext: context,
			PrimaryLocation: &primary, RelatedLocations: []certification.SourceLocation{declaration},
			Evidence: []certification.EvidenceStep{
				{Message: provenance, Location: &declaration},
				{Message: "the call executes in an owned scope with no allowed write role", Location: &primary},
			},
		})
	}
	return findings
}

func reachableWrites(function reactiveir.Function, functions map[reactiveir.FunctionID]reactiveir.Function, visiting map[reactiveir.FunctionID]bool, allowedBy string, directiveApplication bool) []reactiveir.ReactiveWrite {
	if visiting[function.ID] {
		return nil
	}
	visiting[function.ID] = true
	defer delete(visiting, function.ID)
	result := make([]reactiveir.ReactiveWrite, 0, len(function.Writes))
	for _, write := range function.Writes {
		copy := write
		if allowedBy != "" && (!directiveApplication || write.InReturnedClosure) {
			copy.AllowedBy = allowedBy
		}
		result = append(result, copy)
	}
	for _, call := range function.Calls {
		nextAllowed := allowedBy
		nextDirective := directiveApplication
		if call.Execution == reactiveir.ExecutionEventCallback {
			nextAllowed = "event-handler"
		} else if call.Execution == reactiveir.ExecutionDirectiveApply {
			nextAllowed = "directive-apply"
			nextDirective = true
		}
		if target, ok := functions[call.Target]; ok {
			result = append(result, reachableWrites(target, functions, visiting, nextAllowed, nextDirective)...)
		}
	}
	return result
}
