// Package compilerfacts defines the versioned boundary between the Go
// checker and the controlled DOM Expressions/Oxc compiler analysis mode.
package compilerfacts

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
)

const ProtocolVersion = 1

type Analyzer interface {
	Analyze(context.Context, AnalysisRequest) (ExecutionMap, error)
	Close() error
}

type OpenFunc func(context.Context) (Analyzer, error)

type CompilerOptions struct {
	ModuleName string `json:"moduleName,omitempty"`
	Generate   string `json:"generate,omitempty"`
	Hydratable bool   `json:"hydratable,omitempty"`
	Dev        bool   `json:"dev,omitempty"`
}

type AnalysisRequest struct {
	CompilerFactsProtocol int             `json:"compilerFactsProtocol"`
	Path                  string          `json:"path"`
	Source                string          `json:"source"`
	SourceHash            string          `json:"sourceHash"`
	CompilerOptions       CompilerOptions `json:"compilerOptions"`
}

type Span struct {
	Start int `json:"start"`
	End   int `json:"end"`
}

type RegionReason string

const (
	RegionJSXChild        RegionReason = "jsx-child"
	RegionJSXAttribute    RegionReason = "jsx-attribute"
	RegionComponentGetter RegionReason = "component-getter"
)

type ExecutionRegion struct {
	Span   Span         `json:"span"`
	Reason RegionReason `json:"reason"`
}

type OwnershipRegion struct {
	Span Span   `json:"span"`
	Kind string `json:"kind"`
}

type CallbackRoleKind string

const (
	CallbackEventHandler CallbackRoleKind = "event-handler"
	CallbackRender       CallbackRoleKind = "render"
	CallbackDeferred     CallbackRoleKind = "deferred"
)

type CallbackRole struct {
	Span Span             `json:"span"`
	Role CallbackRoleKind `json:"role"`
}

type JsxOperation struct {
	Span Span   `json:"span"`
	Kind string `json:"kind"`
}

type ExecutionMap struct {
	CompilerFactsProtocol int               `json:"compilerFactsProtocol"`
	SourceHash            string            `json:"sourceHash"`
	TrackedRegions        []ExecutionRegion `json:"trackedRegions"`
	UntrackedRegions      []ExecutionRegion `json:"untrackedRegions"`
	OwnershipRegions      []OwnershipRegion `json:"ownershipRegions"`
	CallbackRoles         []CallbackRole    `json:"callbackRoles"`
	JsxOperations         []JsxOperation    `json:"jsxOperations"`
}

func NewRequest(path string, source []byte, options CompilerOptions) AnalysisRequest {
	return AnalysisRequest{
		CompilerFactsProtocol: ProtocolVersion,
		Path:                  path,
		Source:                string(source),
		SourceHash:            HashSource(source),
		CompilerOptions:       options,
	}
}

func HashSource(source []byte) string {
	sum := sha256.Sum256(source)
	return "sha256:" + hex.EncodeToString(sum[:])
}

func Validate(request AnalysisRequest, facts ExecutionMap) error {
	if request.CompilerFactsProtocol != ProtocolVersion {
		return fmt.Errorf("request compiler facts protocol %d is unsupported", request.CompilerFactsProtocol)
	}
	if actual := HashSource([]byte(request.Source)); request.SourceHash != actual {
		return fmt.Errorf("request source hash %q does not match exact source bytes", request.SourceHash)
	}
	if facts.CompilerFactsProtocol != ProtocolVersion {
		return fmt.Errorf("response compiler facts protocol %d is unsupported", facts.CompilerFactsProtocol)
	}
	if facts.SourceHash != request.SourceHash {
		return fmt.Errorf("response source hash %q does not match request source hash %q", facts.SourceHash, request.SourceHash)
	}

	sourceLength := len(request.Source)
	for _, region := range facts.TrackedRegions {
		if err := validateSpan("tracked region", region.Span, sourceLength); err != nil {
			return err
		}
	}
	for _, region := range facts.UntrackedRegions {
		if err := validateSpan("untracked region", region.Span, sourceLength); err != nil {
			return err
		}
	}
	for _, region := range facts.OwnershipRegions {
		if err := validateSpan("ownership region", region.Span, sourceLength); err != nil {
			return err
		}
	}
	for _, role := range facts.CallbackRoles {
		if err := validateSpan("callback role", role.Span, sourceLength); err != nil {
			return err
		}
	}
	for _, operation := range facts.JsxOperations {
		if err := validateSpan("JSX operation", operation.Span, sourceLength); err != nil {
			return err
		}
	}
	return nil
}

func validateSpan(kind string, span Span, sourceLength int) error {
	if span.Start < 0 || span.End < span.Start || span.End > sourceLength {
		return fmt.Errorf("%s span [%d,%d) is outside source byte range [0,%d)", kind, span.Start, span.End, sourceLength)
	}
	return nil
}
