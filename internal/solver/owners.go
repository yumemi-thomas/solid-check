package solver

import (
	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func OwnerPresence(program reactiveir.Program) []certification.Finding {
	functions := make(map[reactiveir.FunctionID]reactiveir.Function, len(program.Functions))
	for _, function := range program.Functions {
		functions[function.ID] = function
	}
	reachable := make(map[reactiveir.FunctionID]bool)
	var visit func(reactiveir.FunctionID)
	visit = func(id reactiveir.FunctionID) {
		if reachable[id] {
			return
		}
		reachable[id] = true
		for _, call := range functions[id].Calls {
			if !call.Owned {
				visit(call.Target)
			}
		}
	}
	for _, call := range program.ModuleCalls {
		if !call.Owned {
			visit(call.Target)
		}
	}
	for _, function := range program.Functions {
		if !function.Rendering {
			continue
		}
		for _, call := range function.Calls {
			if call.Unowned {
				visit(call.Target)
			}
		}
	}
	findings := make([]certification.Finding, 0, len(program.MissingOwners))
	for _, requirement := range program.MissingOwners {
		uncertain := false
		if requirement.Function != "" {
			function, known := functions[requirement.Function]
			if !known || function.Rendering && !requirement.Unowned {
				continue
			}
			if !function.Rendering && !reachable[requirement.Function] {
				if !function.Exported {
					continue
				}
				uncertain = true
			}
		}
		primary := requirement.Location
		id, rule, message := "SC4001", "no-owner-effect", "effect created without a reactive owner will never be disposed"
		switch requirement.Operation {
		case "cleanup":
			id, rule, message = "SC4002", "no-owner-cleanup", "onCleanup called without a reactive owner will never run"
		case "boundary":
			id, rule, message = "SC4003", "no-owner-boundary", "boundary created without a reactive owner will never be disposed"
		case "settled-cleanup":
			id, rule, message = "SC3005", "settled-cleanup-unowned", "onSettled returns a cleanup in an unowned or children-forbidden scope, so the cleanup cannot be honored"
		}
		kind, severity := certification.FindingViolation, certification.SeverityWarning
		if requirement.Operation == "settled-cleanup" {
			severity = certification.SeverityError
		}
		if uncertain {
			kind, severity = certification.FindingUncertifiable, certification.SeverityError
			message += "; caller ownership for this exported function cannot be proven inside the project"
		}
		findings = append(findings, certification.Finding{
			ID: id, Rule: rule, Kind: kind, Severity: severity,
			Message: message, PrimaryLocation: &primary,
			Evidence: []certification.EvidenceStep{{Message: "no containing component, computation, or root owner dominates this operation", Location: &primary}},
		})
	}
	return findings
}
