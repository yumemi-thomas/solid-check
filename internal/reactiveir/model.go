// Package reactiveir contains the AST-independent operations consumed by the
// whole-project solver.
package reactiveir

import "github.com/yumemi-thomas/solid-check/pkg/certification"

type ExecutionRole string
type ReactiveValueKind string

const (
	ExecutionInline             ExecutionRole = "inline"
	ExecutionTrackedJSX         ExecutionRole = "tracked-jsx"
	ExecutionDeferredCallback   ExecutionRole = "deferred-callback"
	ExecutionUntrackedRendering ExecutionRole = "untracked-rendering"
)

const (
	ReactiveAccessor  ReactiveValueKind = "accessor"
	ReactiveStorePath ReactiveValueKind = "store-path"
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

type FunctionID string

type FunctionCall struct {
	Target     FunctionID
	TargetName string
	Arguments  []FunctionID
	Location   certification.SourceLocation
	Execution  ExecutionRole
	Context    string
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
	ReturnedReads       []ReactiveRead
	Calls               []FunctionCall
	CallbackInvocations []CallbackInvocation
}

type Program struct {
	Reads          []ReactiveRead
	Functions      []Function
	ExportedValues []string
	ExportAliases  map[string]FunctionID
	Unresolved     []UnresolvedObligation
}

type UnresolvedObligation struct {
	Message  string
	Location certification.SourceLocation
}
