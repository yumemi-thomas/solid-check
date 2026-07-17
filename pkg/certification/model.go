// Package certification defines the stable, AST-independent result model
// shared by the CLI, LSP, tests, and compatibility adapters.
package certification

import (
	"fmt"
	"sort"
)

// Status is the project-level certification outcome.
type Status string

const (
	StatusCertified     Status = "certified"
	StatusViolation     Status = "violation"
	StatusUncertifiable Status = "uncertifiable"
)

// FindingKind distinguishes proven rule breaches from unresolved proof
// obligations. A checker defect or missing analyzer must be uncertifiable.
type FindingKind string

const (
	FindingViolation     FindingKind = "violation"
	FindingUncertifiable FindingKind = "uncertifiable"
)

// Severity is presentation metadata. It does not determine certification.
type Severity string

const (
	SeverityWarning Severity = "warning"
	SeverityError   Severity = "error"
)

// SourceLocation uses UTF-16 line/column coordinates for editor compatibility.
// Offsets are zero-based; Line and Column are one-based.
type SourceLocation struct {
	Path      string `json:"path"`
	StartByte int    `json:"startByte"`
	EndByte   int    `json:"endByte"`
	Line      int    `json:"line"`
	Column    int    `json:"column"`
}

// EvidenceStep is one link in a replayable proof explanation.
type EvidenceStep struct {
	Message  string          `json:"message"`
	Location *SourceLocation `json:"location,omitempty"`
}

type FixApplicability string

const FixSafe FixApplicability = "safe"

type TextEdit struct {
	Location SourceLocation `json:"location"`
	NewText  string         `json:"newText"`
}

type Fix struct {
	Message       string           `json:"message"`
	Applicability FixApplicability `json:"applicability"`
	Edits         []TextEdit       `json:"edits"`
}

// Finding is either a proven violation or an unresolved proof obligation.
type Finding struct {
	ID               string           `json:"id"`
	Rule             string           `json:"rule"`
	Kind             FindingKind      `json:"kind"`
	Severity         Severity         `json:"severity"`
	Message          string           `json:"message"`
	AnalysisContext  string           `json:"analysisContext,omitempty"`
	SubjectKind      string           `json:"subjectKind,omitempty"`
	PrimaryLocation  *SourceLocation  `json:"primaryLocation,omitempty"`
	RelatedLocations []SourceLocation `json:"relatedLocations,omitempty"`
	Evidence         []EvidenceStep   `json:"evidence,omitempty"`
	Fixes            []Fix            `json:"fixes,omitempty"`
}

// PackageSummary describes contract evidence used for one dependency.
type PackageSummary struct {
	Name            string `json:"name"`
	Version         string `json:"version,omitempty"`
	ContractHash    string `json:"contractHash,omitempty"`
	Evidence        string `json:"evidence"`
	ExportsAnalyzed int    `json:"exportsAnalyzed"`
}

// Metrics contains stable counters useful to adapters and CI.
type Metrics struct {
	FilesAnalyzed         int `json:"filesAnalyzed"`
	FunctionsAnalyzed     int `json:"functionsAnalyzed"`
	ProofObligations      int `json:"proofObligations"`
	CachedSummaries       int `json:"cachedSummaries"`
	UnresolvedObligations int `json:"unresolvedObligations"`
}

// Snapshot is an immutable project result. NewSnapshot copies and sorts all
// caller-owned slices so adapter output is deterministic.
type Snapshot struct {
	Status           Status           `json:"status"`
	Findings         []Finding        `json:"findings"`
	PackageSummaries []PackageSummary `json:"packageSummaries"`
	Metrics          Metrics          `json:"metrics"`
}

// NewSnapshot validates findings and derives status. A proven violation takes
// precedence when both finding kinds exist; certification mode rejects either.
func NewSnapshot(findings []Finding, packages []PackageSummary, metrics Metrics) (Snapshot, error) {
	ownedFindings := append([]Finding{}, findings...)
	ownedPackages := append([]PackageSummary{}, packages...)
	status := StatusCertified

	for i := range ownedFindings {
		finding := &ownedFindings[i]
		if finding.ID == "" || finding.Rule == "" || finding.Message == "" {
			return Snapshot{}, fmt.Errorf("finding %d requires id, rule, and message", i)
		}
		switch finding.Kind {
		case FindingViolation:
			status = StatusViolation
		case FindingUncertifiable:
			if status != StatusViolation {
				status = StatusUncertifiable
			}
		default:
			return Snapshot{}, fmt.Errorf("finding %q has invalid kind %q", finding.ID, finding.Kind)
		}
		if finding.Severity != SeverityWarning && finding.Severity != SeverityError {
			return Snapshot{}, fmt.Errorf("finding %q has invalid severity %q", finding.ID, finding.Severity)
		}
		finding.RelatedLocations = append([]SourceLocation(nil), finding.RelatedLocations...)
		finding.Evidence = append([]EvidenceStep(nil), finding.Evidence...)
		finding.Fixes = append([]Fix(nil), finding.Fixes...)
		for fixIndex := range finding.Fixes {
			fix := &finding.Fixes[fixIndex]
			if fix.Message == "" || fix.Applicability != FixSafe || len(fix.Edits) == 0 {
				return Snapshot{}, fmt.Errorf("finding %q has invalid fix %d", finding.ID, fixIndex)
			}
			fix.Edits = append([]TextEdit(nil), fix.Edits...)
		}
	}

	sort.SliceStable(ownedFindings, func(i, j int) bool {
		left, right := ownedFindings[i], ownedFindings[j]
		if left.PrimaryLocation != nil && right.PrimaryLocation != nil {
			if left.PrimaryLocation.Path != right.PrimaryLocation.Path {
				return left.PrimaryLocation.Path < right.PrimaryLocation.Path
			}
			if left.PrimaryLocation.StartByte != right.PrimaryLocation.StartByte {
				return left.PrimaryLocation.StartByte < right.PrimaryLocation.StartByte
			}
		}
		return left.ID < right.ID
	})
	sort.SliceStable(ownedPackages, func(i, j int) bool {
		return ownedPackages[i].Name < ownedPackages[j].Name
	})

	return Snapshot{
		Status:           status,
		Findings:         ownedFindings,
		PackageSummaries: ownedPackages,
		Metrics:          metrics,
	}, nil
}
