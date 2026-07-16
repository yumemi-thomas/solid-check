package certification

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestNewSnapshotDerivesFailClosedStatus(t *testing.T) {
	tests := []struct {
		name     string
		findings []Finding
		want     Status
	}{
		{name: "no findings", want: StatusCertified},
		{name: "unsupported boundary", findings: []Finding{{
			ID: "SC0001", Rule: "unsupported", Message: "unresolved", Kind: FindingUncertifiable, Severity: SeverityError,
		}}, want: StatusUncertifiable},
		{name: "violation wins", findings: []Finding{
			{ID: "SC0001", Rule: "unsupported", Message: "unresolved", Kind: FindingUncertifiable, Severity: SeverityError},
			{ID: "SC1001", Rule: "strict-read-untracked", Message: "bad read", Kind: FindingViolation, Severity: SeverityWarning},
		}, want: StatusViolation},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			snapshot, err := NewSnapshot(test.findings, nil, Metrics{})
			if err != nil {
				t.Fatal(err)
			}
			if snapshot.Status != test.want {
				t.Fatalf("status = %q, want %q", snapshot.Status, test.want)
			}
		})
	}
}

func TestNewSnapshotRejectsMalformedFinding(t *testing.T) {
	_, err := NewSnapshot([]Finding{{Kind: FindingViolation}}, nil, Metrics{})
	if err == nil {
		t.Fatal("expected malformed finding to be rejected")
	}
}

func TestNewSnapshotSerializesEmptyCollectionsAsArrays(t *testing.T) {
	snapshot, err := NewSnapshot(nil, nil, Metrics{})
	if err != nil {
		t.Fatal(err)
	}
	encoded, err := json.Marshal(snapshot)
	if err != nil {
		t.Fatal(err)
	}
	jsonText := string(encoded)
	for _, field := range []string{`"findings":[]`, `"packageSummaries":[]`} {
		if !strings.Contains(jsonText, field) {
			t.Fatalf("snapshot JSON %s does not contain %s", jsonText, field)
		}
	}
}
