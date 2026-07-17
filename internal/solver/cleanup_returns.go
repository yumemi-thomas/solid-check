package solver

import (
	"fmt"

	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func CleanupReturns(program reactiveir.Program) []certification.Finding {
	findings := make([]certification.Finding, 0, len(program.InvalidCleanupReturns))
	for _, invalid := range program.InvalidCleanupReturns {
		primary := invalid.Location
		findings = append(findings, certification.Finding{
			ID: "SC3004", Rule: "invalid-cleanup-return",
			Kind: certification.FindingViolation, Severity: certification.SeverityError,
			Message:         fmt.Sprintf("%s callback returns a non-function cleanup value; return a cleanup function or undefined", invalid.Primitive),
			PrimaryLocation: &primary,
			Evidence:        []certification.EvidenceStep{{Message: "the callback statically returns a non-function value, including an implicit Promise from an async callback", Location: &primary}},
		})
	}
	return findings
}
