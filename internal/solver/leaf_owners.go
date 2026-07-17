package solver

import (
	"fmt"

	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func LeafOwnerRestrictions(program reactiveir.Program) []certification.Finding {
	findings := make([]certification.Finding, 0, len(program.LeafOperations))
	for _, operation := range program.LeafOperations {
		primary := operation.Location
		id, rule, message := "SC3002", "primitive-in-leaf-owner", fmt.Sprintf("cannot create reactive primitive %s inside leaf owner %s", operation.Primitive, operation.Owner)
		switch operation.Primitive {
		case "onCleanup":
			id, rule = "SC3001", "cleanup-in-forbidden-scope"
			message = fmt.Sprintf("onCleanup cannot be used inside %s; return a cleanup function instead", operation.Owner)
		case "flush":
			id, rule = "SC3003", "flush-in-forbidden-scope"
			message = fmt.Sprintf("flush cannot be called inside %s because the leaf owner is not reentrant", operation.Owner)
		}
		findings = append(findings, certification.Finding{
			ID: id, Rule: rule, Kind: certification.FindingViolation, Severity: certification.SeverityError,
			Message: message, PrimaryLocation: &primary,
			Evidence: []certification.EvidenceStep{{Message: fmt.Sprintf("the call is lexically contained by the %s callback", operation.Owner), Location: &primary}},
			Fixes: func() []certification.Fix {
				if operation.Fix == nil {
					return nil
				}
				return []certification.Fix{*operation.Fix}
			}(),
		})
	}
	return findings
}
