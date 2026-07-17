package solver

import (
	"fmt"

	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func AsyncBoundaries(program reactiveir.Program) []certification.Finding {
	findings := make([]certification.Finding, 0)
	for _, read := range program.AsyncReads {
		primary, declaration := read.Location, read.Declaration
		id, rule, severity, message := "", "", certification.SeverityError, ""
		switch {
		case read.LeafOwner != "":
			id, rule, severity = "SC5002", "pending-async-forbidden-scope", certification.SeverityWarning
			message = fmt.Sprintf("pending async accessor %q is read inside %s, which cannot suspend", read.Accessor, read.LeafOwner)
		case read.Execution == reactiveir.ExecutionUntrackedRendering:
			id, rule = "SC5001", "pending-async-untracked-read"
			message = fmt.Sprintf("pending async accessor %q is read outside a tracking scope", read.Accessor)
		case read.Execution == reactiveir.ExecutionTrackedJSX && !read.UnderLoading:
			id, rule = "SC5003", "async-outside-loading-boundary"
			message = fmt.Sprintf("async accessor %q is rendered without a dominating Loading boundary", read.Accessor)
		default:
			continue
		}
		findings = append(findings, certification.Finding{
			ID: id, Rule: rule, Kind: certification.FindingViolation, Severity: severity,
			Message: message, PrimaryLocation: &primary, RelatedLocations: []certification.SourceLocation{declaration},
			Evidence: []certification.EvidenceStep{
				{Message: "the accessor is returned by an async computation", Location: &declaration},
				{Message: message, Location: &primary},
			},
		})
	}
	return findings
}
