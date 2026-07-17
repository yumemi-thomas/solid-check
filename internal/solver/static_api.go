package solver

import (
	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func StaticAPIDiagnostics(program reactiveir.Program) []certification.Finding {
	findings := make([]certification.Finding, 0, len(program.StaticViolations))
	for _, violation := range program.StaticViolations {
		primary := violation.Location
		evidence := "the invalid API shape is statically present at this call"
		if violation.Rule == "component-props-destructure" {
			evidence = "the destructuring pattern is bound to proven component props"
		}
		findings = append(findings, certification.Finding{
			ID: violation.ID, Rule: violation.Rule,
			Kind: certification.FindingViolation, Severity: certification.SeverityError,
			Message: violation.Message, AnalysisContext: violation.AnalysisContext, PrimaryLocation: &primary,
			Evidence: []certification.EvidenceStep{{Message: evidence, Location: &primary}},
			Fixes:    violation.Fixes,
		})
	}
	return findings
}
