// Package reactiveir contains the AST-independent operations consumed by the
// whole-project solver.
package reactiveir

import "github.com/yumemi-thomas/solid-check/pkg/certification"

type ExecutionRole string
type ReactiveValueKind string

const (
	ExecutionInline             ExecutionRole = "inline"
	ExecutionTrackedJSX         ExecutionRole = "tracked-jsx"
	ExecutionTrackedComputation ExecutionRole = "tracked-computation"
	ExecutionDeferredCallback   ExecutionRole = "deferred-callback"
	ExecutionEventCallback      ExecutionRole = "event-callback"
	ExecutionDirectiveApply     ExecutionRole = "directive-apply"
	ExecutionUntrackedRendering ExecutionRole = "untracked-rendering"
)

const (
	ReactiveAccessor  ReactiveValueKind = "accessor"
	ReactiveStorePath ReactiveValueKind = "store-path"
	ReactiveProps     ReactiveValueKind = "component-props"
)

type ReactiveRead struct {
	Kind          ReactiveValueKind
	Accessor      string
	Location      certification.SourceLocation
	Declaration   certification.SourceLocation
	Execution     ExecutionRole
	Context       string
	Via           string
	Origin        *certification.SourceLocation
	OriginContext string
}

type ReactiveWrite struct {
	Setter            string
	Location          certification.SourceLocation
	Declaration       certification.SourceLocation
	Execution         ExecutionRole
	Context           string
	AllowedBy         string
	InReturnedClosure bool
}

type ActionInvocation struct {
	Action    string
	Location  certification.SourceLocation
	Context   string
	AllowedBy string
}

type PrimitiveCreation struct {
	Primitive         string
	Location          certification.SourceLocation
	InReturnedClosure bool
}

type LeafOwnerOperation struct {
	Primitive string
	Owner     string
	Location  certification.SourceLocation
	Fix       *certification.Fix
}

type OwnerRequirement struct {
	Operation string
	Location  certification.SourceLocation
	Function  FunctionID
	Unowned   bool
}

type AsyncRead struct {
	Kind         ReactiveValueKind
	Accessor     string
	Location     certification.SourceLocation
	Declaration  certification.SourceLocation
	Execution    ExecutionRole
	LeafOwner    string
	UnderLoading bool
	Function     FunctionID
}

type InvalidCleanupReturn struct {
	Primitive string
	Location  certification.SourceLocation
}

type StaticViolation struct {
	ID              string
	Rule            string
	Message         string
	AnalysisContext string
	Location        certification.SourceLocation
	Fixes           []certification.Fix
}

type FunctionID string

type FunctionCall struct {
	Target     FunctionID
	TargetName string
	Arguments  []FunctionID
	Location   certification.SourceLocation
	Execution  ExecutionRole
	Context    string
	Owned      bool
	Unowned    bool
}

type CallbackInvocation struct {
	Parameter int
	Location  certification.SourceLocation
	Execution ExecutionRole
	Context   string
}

type Function struct {
	ID                  FunctionID
	Name                string
	Exported            bool
	Async               bool
	Rendering           bool
	Reads               []ReactiveRead
	Writes              []ReactiveWrite
	ActionCalls         []ActionInvocation
	ReturnedReads       []ReactiveRead
	Calls               []FunctionCall
	CallbackInvocations []CallbackInvocation
	PrimitiveCreations  []PrimitiveCreation
}

type Program struct {
	Reads                 []ReactiveRead
	Writes                []ReactiveWrite
	ActionCalls           []ActionInvocation
	LeafOperations        []LeafOwnerOperation
	MissingOwners         []OwnerRequirement
	AsyncReads            []AsyncRead
	InvalidCleanupReturns []InvalidCleanupReturn
	DirectiveCreations    []PrimitiveCreation
	StaticViolations      []StaticViolation
	Functions             []Function
	ExportedValues        []string
	ExportAliases         map[string]FunctionID
	Unresolved            []UnresolvedObligation
	ModuleCalls           []FunctionCall
}

type UnresolvedObligation struct {
	Message  string
	Location certification.SourceLocation
	ID       string
	Rule     string
}
