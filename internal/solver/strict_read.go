// Package solver proves rules over the AST-independent Reactive IR.
package solver

import (
	"fmt"

	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

// StrictReads implements the first Solid 2 proof rule: reactive reads in the
// immediate structure-building body of a rendering function do not track.
func StrictReads(program reactiveir.Program) []certification.Finding {
	return SolveStrictReads(program).Findings
}

type StrictReadResult struct {
	Findings              []certification.Finding
	ProofObligations      int
	UnresolvedObligations int
}

// SolveStrictReads expands function summaries to rendering entrypoints and
// returns both diagnostics and the number of concrete read obligations proven.
func SolveStrictReads(program reactiveir.Program) StrictReadResult {
	reads := append([]reactiveir.ReactiveRead{}, program.Reads...)
	summaries := solveFunctionReads(program.Functions)
	for _, function := range program.Functions {
		if function.Rendering {
			reads = append(reads, summaries[function.ID]...)
		}
	}
	findings := make([]certification.Finding, 0)
	for _, unresolved := range program.Unresolved {
		location := unresolved.Location
		id, rule := unresolved.ID, unresolved.Rule
		evidence := unresolved.Message
		if id == "" {
			id = "SC9001"
			evidence = "the imported package has a contract, but this export has no effect summary"
		}
		if rule == "" {
			rule = "package-contract-export-missing"
		}
		findings = append(findings, certification.Finding{
			ID: id, Rule: rule,
			Kind: certification.FindingUncertifiable, Severity: certification.SeverityError,
			Message: unresolved.Message, PrimaryLocation: &location,
			Evidence: []certification.EvidenceStep{{
				Message:  evidence,
				Location: &location,
			}},
		})
	}
	for _, read := range reads {
		if read.Execution != reactiveir.ExecutionUntrackedRendering {
			continue
		}
		primary := read.Location
		declaration := read.Declaration
		context := read.Context
		if context == "" {
			context = "rendering function"
		}
		label := reactiveValueLabel(read.Kind)
		message := fmt.Sprintf(
			"%s %q is read directly in %s and will not update; move the read into tracked JSX, a memo, or an effect compute function",
			label,
			read.Accessor,
			context,
		)
		related := []certification.SourceLocation{declaration}
		evidence := []certification.EvidenceStep{
			{Message: fmt.Sprintf("%q is a %s", read.Accessor, label), Location: &declaration},
			{Message: "the cross-file reference resolves to that accessor declaration", Location: &primary},
			{Message: "the read is outside every compiler-tracked JSX region and deferred callback", Location: &primary},
		}
		if read.Via != "" && read.Origin != nil {
			origin := *read.Origin
			originContext := read.OriginContext
			if originContext == "" {
				originContext = read.Via
			}
			message = fmt.Sprintf(
				"%s %q is read through %s in %s and will not update; move the call into tracked JSX, a memo, or an effect compute function",
				label,
				read.Accessor,
				read.Via,
				context,
			)
			related = append(related, origin)
			evidence = []certification.EvidenceStep{
				{Message: fmt.Sprintf("%q is a %s", read.Accessor, label), Location: &declaration},
				{Message: fmt.Sprintf("%s reads the %s", originContext, label), Location: &origin},
				{Message: fmt.Sprintf("the call to %s propagates that read into %s", read.Via, context), Location: &primary},
				{Message: "the call is outside every compiler-tracked JSX region and deferred callback", Location: &primary},
			}
		}
		findings = append(findings, certification.Finding{
			ID:               "SC1001",
			Rule:             "strict-read-untracked",
			Kind:             certification.FindingViolation,
			Severity:         certification.SeverityWarning,
			Message:          message,
			AnalysisContext:  read.Context,
			SubjectKind:      string(read.Kind),
			PrimaryLocation:  &primary,
			RelatedLocations: related,
			Evidence:         evidence,
		})
	}
	return StrictReadResult{
		Findings: findings, ProofObligations: len(reads) + len(program.Unresolved),
		UnresolvedObligations: len(program.Unresolved),
	}
}

func reactiveValueLabel(kind reactiveir.ReactiveValueKind) string {
	if kind == reactiveir.ReactiveStorePath {
		return "reactive store path"
	}
	if kind == reactiveir.ReactiveProps {
		return "component prop"
	}
	return "reactive accessor"
}

// FunctionReadSummaries returns solved read effects by function identity.
// Returned slices are detached from solver-owned storage.
func FunctionReadSummaries(program reactiveir.Program) map[reactiveir.FunctionID][]reactiveir.ReactiveRead {
	solved := solveFunctionReads(program.Functions)
	result := make(map[reactiveir.FunctionID][]reactiveir.ReactiveRead, len(solved))
	for id, reads := range solved {
		result[id] = append([]reactiveir.ReactiveRead{}, reads...)
	}
	return result
}

func solveFunctionReads(functions []reactiveir.Function) map[reactiveir.FunctionID][]reactiveir.ReactiveRead {
	summaries := make(map[reactiveir.FunctionID][]reactiveir.ReactiveRead, len(functions))
	functionsByID := make(map[reactiveir.FunctionID]reactiveir.Function, len(functions))
	for _, function := range functions {
		summaries[function.ID] = append([]reactiveir.ReactiveRead{}, function.Reads...)
		functionsByID[function.ID] = function
	}
	for _, component := range stronglyConnectedComponents(functions, functionsByID) {
		for changed := true; changed; {
			changed = false
			for _, id := range component {
				if expandFunctionReads(functionsByID[id], functionsByID, summaries) {
					changed = true
				}
			}
		}
	}
	return summaries
}

func expandFunctionReads(
	function reactiveir.Function,
	functionsByID map[reactiveir.FunctionID]reactiveir.Function,
	summaries map[reactiveir.FunctionID][]reactiveir.ReactiveRead,
) bool {
	changed := false
	for _, call := range function.Calls {
		for _, read := range summaries[call.Target] {
			if appendUniqueRead(summaries, function.ID, propagateRead(read, call)) {
				changed = true
			}
		}
		target := functionsByID[call.Target]
		for _, invocation := range target.CallbackInvocations {
			if invocation.Parameter >= len(call.Arguments) {
				continue
			}
			argument := call.Arguments[invocation.Parameter]
			if argument == "" {
				continue
			}
			argumentName := functionsByID[argument].Name
			for _, read := range summaries[argument] {
				throughCallback := propagateAt(read, invocation.Execution, invocation.Location, invocation.Context, argumentName)
				if appendUniqueRead(summaries, function.ID, propagateRead(throughCallback, call)) {
					changed = true
				}
			}
		}
	}
	return changed
}

func stronglyConnectedComponents(
	functions []reactiveir.Function,
	functionsByID map[reactiveir.FunctionID]reactiveir.Function,
) [][]reactiveir.FunctionID {
	index := 0
	indices := make(map[reactiveir.FunctionID]int, len(functions))
	lowlinks := make(map[reactiveir.FunctionID]int, len(functions))
	onStack := make(map[reactiveir.FunctionID]bool, len(functions))
	stack := make([]reactiveir.FunctionID, 0, len(functions))
	components := make([][]reactiveir.FunctionID, 0)
	var visit func(reactiveir.FunctionID)
	visit = func(id reactiveir.FunctionID) {
		indices[id] = index
		lowlinks[id] = index
		index++
		stack = append(stack, id)
		onStack[id] = true
		for _, dependency := range functionDependencies(functionsByID[id], functionsByID) {
			if _, seen := indices[dependency]; !seen {
				visit(dependency)
				lowlinks[id] = min(lowlinks[id], lowlinks[dependency])
			} else if onStack[dependency] {
				lowlinks[id] = min(lowlinks[id], indices[dependency])
			}
		}
		if lowlinks[id] != indices[id] {
			return
		}
		component := make([]reactiveir.FunctionID, 0)
		for {
			last := len(stack) - 1
			member := stack[last]
			stack = stack[:last]
			onStack[member] = false
			component = append(component, member)
			if member == id {
				break
			}
		}
		components = append(components, component)
	}
	for _, function := range functions {
		if _, seen := indices[function.ID]; !seen {
			visit(function.ID)
		}
	}
	return components
}

func functionDependencies(
	function reactiveir.Function,
	functionsByID map[reactiveir.FunctionID]reactiveir.Function,
) []reactiveir.FunctionID {
	dependencies := make([]reactiveir.FunctionID, 0)
	seen := make(map[reactiveir.FunctionID]struct{})
	for _, call := range function.Calls {
		if _, ok := functionsByID[call.Target]; ok {
			seen[call.Target] = struct{}{}
		}
		target := functionsByID[call.Target]
		for _, invocation := range target.CallbackInvocations {
			if invocation.Parameter < len(call.Arguments) && call.Arguments[invocation.Parameter] != "" {
				argument := call.Arguments[invocation.Parameter]
				if _, ok := functionsByID[argument]; ok {
					seen[argument] = struct{}{}
				}
			}
		}
	}
	for dependency := range seen {
		dependencies = append(dependencies, dependency)
	}
	return dependencies
}

func propagateRead(read reactiveir.ReactiveRead, call reactiveir.FunctionCall) reactiveir.ReactiveRead {
	return propagateAt(read, call.Execution, call.Location, call.Context, call.TargetName)
}

func propagateAt(
	read reactiveir.ReactiveRead,
	execution reactiveir.ExecutionRole,
	location certification.SourceLocation,
	context, via string,
) reactiveir.ReactiveRead {
	if read.Execution != reactiveir.ExecutionInline || execution == reactiveir.ExecutionInline {
		return read
	}
	origin := read.Location
	originContext := read.Context
	if read.Origin != nil {
		origin = *read.Origin
		originContext = read.OriginContext
	}
	read.Execution = execution
	read.Location = location
	read.Context = context
	read.Via = via
	read.Origin = &origin
	read.OriginContext = originContext
	return read
}

func appendUniqueRead(summaries map[reactiveir.FunctionID][]reactiveir.ReactiveRead, id reactiveir.FunctionID, candidate reactiveir.ReactiveRead) bool {
	for _, existing := range summaries[id] {
		if existing.Accessor == candidate.Accessor &&
			existing.Execution == candidate.Execution &&
			existing.Location.Path == candidate.Location.Path &&
			existing.Location.StartByte == candidate.Location.StartByte &&
			existing.Declaration.Path == candidate.Declaration.Path &&
			existing.Declaration.StartByte == candidate.Declaration.StartByte {
			return false
		}
	}
	summaries[id] = append(summaries[id], candidate)
	return true
}
