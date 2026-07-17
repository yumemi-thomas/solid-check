package solver

import (
	"fmt"

	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func ActionCalls(program reactiveir.Program) []certification.Finding {
	invocations := append([]reactiveir.ActionInvocation{}, program.ActionCalls...)
	functions := make(map[reactiveir.FunctionID]reactiveir.Function, len(program.Functions))
	for _, function := range program.Functions {
		functions[function.ID] = function
	}
	for _, function := range program.Functions {
		if function.Rendering {
			invocations = append(invocations, reachableActionCalls(function, functions, map[reactiveir.FunctionID]bool{}, "")...)
		}
	}
	for _, call := range program.ModuleCalls {
		if call.Owned {
			if target, ok := functions[call.Target]; ok {
				invocations = append(invocations, reachableActionCalls(target, functions, map[reactiveir.FunctionID]bool{}, "")...)
			}
		}
	}
	findings := make([]certification.Finding, 0)
	for _, invocation := range invocations {
		if invocation.AllowedBy != "" {
			continue
		}
		primary := invocation.Location
		findings = append(findings, certification.Finding{
			ID: "SC2002", Rule: "action-called-in-owned-scope",
			Kind: certification.FindingViolation, Severity: certification.SeverityError,
			Message:         fmt.Sprintf("action %q is called inside owned scope %s; invoke it from an event, effect callback, onSettled, or another imperative scope", invocation.Action, invocation.Context),
			PrimaryLocation: &primary,
			Evidence:        []certification.EvidenceStep{{Message: "invoking an action starts a write transaction while an owner is active", Location: &primary}},
		})
	}
	return findings
}

func reachableActionCalls(function reactiveir.Function, functions map[reactiveir.FunctionID]reactiveir.Function, visiting map[reactiveir.FunctionID]bool, allowedBy string) []reactiveir.ActionInvocation {
	if visiting[function.ID] {
		return nil
	}
	visiting[function.ID] = true
	defer delete(visiting, function.ID)
	result := make([]reactiveir.ActionInvocation, 0, len(function.ActionCalls))
	for _, invocation := range function.ActionCalls {
		copy := invocation
		if allowedBy != "" {
			copy.AllowedBy = allowedBy
		}
		result = append(result, copy)
	}
	for _, call := range function.Calls {
		nextAllowed := allowedBy
		if call.Execution == reactiveir.ExecutionEventCallback || call.Execution == reactiveir.ExecutionDirectiveApply {
			nextAllowed = string(call.Execution)
		}
		if target, ok := functions[call.Target]; ok {
			result = append(result, reachableActionCalls(target, functions, visiting, nextAllowed)...)
		}
	}
	return result
}
