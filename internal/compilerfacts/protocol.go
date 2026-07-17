// Package compilerfacts defines the versioned boundary between the Go
// checker and the controlled DOM Expressions/Oxc compiler analysis mode.
package compilerfacts

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"slices"
	"sort"
	"strings"
	"unicode/utf8"
)

const ProtocolVersion = 1

type Analyzer interface {
	Analyze(context.Context, AnalysisRequest) (ExecutionMap, error)
	Close() error
}

type OpenFunc func(context.Context) (Analyzer, error)

type CompilerOptions struct {
	ModuleName       string   `json:"moduleName,omitempty"`
	Generate         string   `json:"generate,omitempty"`
	Hydratable       bool     `json:"hydratable,omitempty"`
	Dev              bool     `json:"dev,omitempty"`
	EffectWrapper    *string  `json:"effectWrapper,omitempty"`
	WrapConditionals *bool    `json:"wrapConditionals,omitempty"`
	StaticMarker     string   `json:"staticMarker,omitempty"`
	BuiltIns         []string `json:"builtIns,omitempty"`
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
	CallbackEventHandler   CallbackRoleKind = "event-handler"
	CallbackRender         CallbackRoleKind = "render"
	CallbackDeferred       CallbackRoleKind = "deferred"
	CallbackDirectiveApply CallbackRoleKind = "directive-apply"
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
	options.BuiltIns = append([]string(nil), options.BuiltIns...)
	sort.Strings(options.BuiltIns)
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

func ValidateRequest(request AnalysisRequest) error {
	if request.CompilerFactsProtocol != ProtocolVersion {
		return fmt.Errorf("request compiler facts protocol %d is unsupported", request.CompilerFactsProtocol)
	}
	if strings.TrimSpace(request.Path) == "" || strings.ContainsRune(request.Path, '\x00') {
		return errors.New("request path is required and must not contain NUL")
	}
	if !utf8.ValidString(request.Source) {
		return errors.New("request source is not valid UTF-8")
	}
	if actual := HashSource([]byte(request.Source)); request.SourceHash != actual {
		return fmt.Errorf("request source hash %q does not match exact source bytes", request.SourceHash)
	}
	if strings.TrimSpace(request.CompilerOptions.ModuleName) == "" {
		return errors.New("request compiler option moduleName is required")
	}
	if request.CompilerOptions.Generate != "dom" {
		return fmt.Errorf("request compiler configuration supports DOM output only, got %q", request.CompilerOptions.Generate)
	}
	if request.CompilerOptions.EffectWrapper != nil && strings.ContainsRune(*request.CompilerOptions.EffectWrapper, '\x00') {
		return errors.New("request compiler option effectWrapper must not contain NUL")
	}
	if strings.ContainsRune(request.CompilerOptions.StaticMarker, '\x00') {
		return errors.New("request compiler option staticMarker must not contain NUL")
	}
	if !slices.IsSorted(request.CompilerOptions.BuiltIns) {
		return errors.New("request compiler option builtIns must be sorted")
	}
	for index, name := range request.CompilerOptions.BuiltIns {
		if strings.TrimSpace(name) == "" {
			return fmt.Errorf("request compiler option builtIns[%d] is empty", index)
		}
		if index > 0 && name == request.CompilerOptions.BuiltIns[index-1] {
			return fmt.Errorf("request compiler option builtIns contains duplicate %q", name)
		}
	}
	return nil
}

func Validate(request AnalysisRequest, facts ExecutionMap) error {
	if err := ValidateRequest(request); err != nil {
		return err
	}
	if facts.CompilerFactsProtocol != ProtocolVersion {
		return fmt.Errorf("response compiler facts protocol %d is unsupported", facts.CompilerFactsProtocol)
	}
	if facts.SourceHash != request.SourceHash {
		return fmt.Errorf("response source hash %q does not match request source hash %q", facts.SourceHash, request.SourceHash)
	}

	for index, region := range facts.TrackedRegions {
		if !knownRegionReason(region.Reason) {
			return fmt.Errorf("tracked region %d has unsupported reason %q", index, region.Reason)
		}
		if err := validateSpan("tracked region", region.Span, request.Source); err != nil {
			return err
		}
		if err := validateRegionOrder("tracked regions", facts.TrackedRegions, index); err != nil {
			return err
		}
	}
	for index, region := range facts.UntrackedRegions {
		if !knownRegionReason(region.Reason) {
			return fmt.Errorf("untracked region %d has unsupported reason %q", index, region.Reason)
		}
		if err := validateSpan("untracked region", region.Span, request.Source); err != nil {
			return err
		}
		if err := validateRegionOrder("untracked regions", facts.UntrackedRegions, index); err != nil {
			return err
		}
	}
	for index, region := range facts.OwnershipRegions {
		if strings.TrimSpace(region.Kind) == "" {
			return fmt.Errorf("ownership region %d has empty kind", index)
		}
		if err := validateSpan("ownership region", region.Span, request.Source); err != nil {
			return err
		}
		if err := validateOwnershipOrder(facts.OwnershipRegions, index); err != nil {
			return err
		}
	}
	for index, role := range facts.CallbackRoles {
		if !knownCallbackRole(role.Role) {
			return fmt.Errorf("callback role %d has unsupported role %q", index, role.Role)
		}
		if err := validateSpan("callback role", role.Span, request.Source); err != nil {
			return err
		}
		if err := validateCallbackOrder(facts.CallbackRoles, index); err != nil {
			return err
		}
	}
	for index, operation := range facts.JsxOperations {
		if !knownJSXOperation(operation.Kind) {
			return fmt.Errorf("JSX operation %d has unsupported kind %q", index, operation.Kind)
		}
		if err := validateSpan("JSX operation", operation.Span, request.Source); err != nil {
			return err
		}
		if err := validateOperationOrder(facts.JsxOperations, index); err != nil {
			return err
		}
	}
	return nil
}

// UncoveredJSXExpressions returns the spans of jsx-expression operations that
// no tracked region, untracked region, callback role, or component-property
// operation covers. Every JSX expression hole the compiler records must be
// classified by one of those facts; an uncovered hole means fact recording
// has no branch for the construct, so callers must fail closed instead of
// assuming the expression renders untracked.
func UncoveredJSXExpressions(facts ExecutionMap) []Span {
	uncovered := make([]Span, 0)
	for _, operation := range facts.JsxOperations {
		if operation.Kind != "jsx-expression" {
			continue
		}
		if jsxExpressionCovered(facts, operation.Span) {
			continue
		}
		uncovered = append(uncovered, operation.Span)
	}
	return uncovered
}

func jsxExpressionCovered(facts ExecutionMap, span Span) bool {
	for _, region := range facts.TrackedRegions {
		if spanContains(region.Span, span) {
			return true
		}
	}
	for _, region := range facts.UntrackedRegions {
		if spanContains(region.Span, span) {
			return true
		}
	}
	for _, callback := range facts.CallbackRoles {
		if spanContains(callback.Span, span) {
			return true
		}
	}
	for _, operation := range facts.JsxOperations {
		if operation.Kind == "component-property" && spanContains(operation.Span, span) {
			return true
		}
	}
	return false
}

func spanContains(outer, inner Span) bool {
	return outer.Start <= inner.Start && inner.End <= outer.End
}

func validateSpan(kind string, span Span, source string) error {
	sourceLength := len(source)
	if span.Start < 0 || span.End < span.Start || span.End > sourceLength {
		return fmt.Errorf("%s span [%d,%d) is outside source byte range [0,%d)", kind, span.Start, span.End, sourceLength)
	}
	if !byteBoundary(source, span.Start) || !byteBoundary(source, span.End) {
		return fmt.Errorf("%s span [%d,%d) does not fall on UTF-8 boundaries", kind, span.Start, span.End)
	}
	return nil
}

func byteBoundary(source string, offset int) bool {
	return offset == 0 || offset == len(source) || utf8.RuneStart(source[offset])
}

func knownRegionReason(reason RegionReason) bool {
	switch reason {
	case RegionJSXChild, RegionJSXAttribute, RegionComponentGetter:
		return true
	default:
		return false
	}
}

func knownCallbackRole(role CallbackRoleKind) bool {
	switch role {
	case CallbackEventHandler, CallbackRender, CallbackDeferred, CallbackDirectiveApply:
		return true
	default:
		return false
	}
}

func knownJSXOperation(kind string) bool {
	switch kind {
	case "dynamic-attribute", "event-listener", "insert", "component-invocation", "component-property", "directive-apply", "directive-setup", "jsx-expression":
		return true
	default:
		return false
	}
}

func compareFact(left Span, leftKind string, right Span, rightKind string) int {
	if left.Start != right.Start {
		return left.Start - right.Start
	}
	if left.End != right.End {
		return left.End - right.End
	}
	return strings.Compare(leftKind, rightKind)
}

func orderError(kind string, comparison int) error {
	if comparison == 0 {
		return fmt.Errorf("%s contain a duplicate fact", kind)
	}
	if comparison > 0 {
		return fmt.Errorf("%s are not sorted deterministically", kind)
	}
	return nil
}

func validateRegionOrder(kind string, regions []ExecutionRegion, index int) error {
	if index == 0 {
		return nil
	}
	previous, current := regions[index-1], regions[index]
	return orderError(kind, compareFact(previous.Span, string(previous.Reason), current.Span, string(current.Reason)))
}

func validateOwnershipOrder(regions []OwnershipRegion, index int) error {
	if index == 0 {
		return nil
	}
	previous, current := regions[index-1], regions[index]
	return orderError("ownership regions", compareFact(previous.Span, previous.Kind, current.Span, current.Kind))
}

func validateCallbackOrder(roles []CallbackRole, index int) error {
	if index == 0 {
		return nil
	}
	previous, current := roles[index-1], roles[index]
	return orderError("callback roles", compareFact(previous.Span, string(previous.Role), current.Span, string(current.Role)))
}

func validateOperationOrder(operations []JsxOperation, index int) error {
	if index == 0 {
		return nil
	}
	previous, current := operations[index-1], operations[index]
	return orderError("JSX operations", compareFact(previous.Span, previous.Kind, current.Span, current.Kind))
}
