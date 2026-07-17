package solver

import (
	"fmt"

	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func DirectiveApplications(program reactiveir.Program) []certification.Finding {
	functions := make(map[reactiveir.FunctionID]reactiveir.Function, len(program.Functions))
	for _, function := range program.Functions {
		functions[function.ID] = function
	}
	findings := make([]certification.Finding, 0)
	for _, function := range program.Functions {
		if !function.Rendering {
			continue
		}
		for _, call := range function.Calls {
			if call.Execution != reactiveir.ExecutionDirectiveApply {
				continue
			}
			target, ok := functions[call.Target]
			if !ok {
				continue
			}
			for _, creation := range target.PrimitiveCreations {
				if !creation.InReturnedClosure {
					continue
				}
				primary := creation.Location
				findings = append(findings, certification.Finding{
					ID: "SC6001", Rule: "primitive-in-directive-application",
					Kind: certification.FindingViolation, Severity: certification.SeverityError,
					Message:         fmt.Sprintf("cannot create reactive primitive %s in a directive application callback; create it during directive setup", creation.Primitive),
					PrimaryLocation: &primary,
					Evidence:        []certification.EvidenceStep{{Message: "the primitive is created inside the callback returned to a compiler-recognized ref application", Location: &primary}},
				})
			}
		}
	}
	for _, creation := range program.DirectiveCreations {
		findings = append(findings, directivePrimitiveFinding(creation))
	}
	return findings
}

func directivePrimitiveFinding(creation reactiveir.PrimitiveCreation) certification.Finding {
	primary := creation.Location
	return certification.Finding{
		ID: "SC6001", Rule: "primitive-in-directive-application",
		Kind: certification.FindingViolation, Severity: certification.SeverityError,
		Message:         fmt.Sprintf("cannot create reactive primitive %s in a directive application callback; create it during directive setup", creation.Primitive),
		PrimaryLocation: &primary,
		Evidence:        []certification.EvidenceStep{{Message: "the primitive is created inside a compiler-recognized ref application callback", Location: &primary}},
	}
}

// DirectiveApplicationObligations reports the primitive-creation candidates
// checked by DirectiveApplications so snapshot metrics include this rule slice.
func DirectiveApplicationObligations(program reactiveir.Program) int {
	functions := make(map[reactiveir.FunctionID]reactiveir.Function, len(program.Functions))
	for _, function := range program.Functions {
		functions[function.ID] = function
	}
	count := len(program.DirectiveCreations)
	for _, function := range program.Functions {
		if !function.Rendering {
			continue
		}
		for _, call := range function.Calls {
			if call.Execution != reactiveir.ExecutionDirectiveApply {
				continue
			}
			for _, creation := range functions[call.Target].PrimitiveCreations {
				if creation.InReturnedClosure {
					count++
				}
			}
		}
	}
	return count
}
