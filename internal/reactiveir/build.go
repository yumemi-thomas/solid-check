package reactiveir

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"unicode/utf16"
	"unicode/utf8"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/typefacts"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
	"github.com/yumemi-thomas/solid-check/pkg/contracts"
)

var (
	solidCalleePattern       = `[A-Za-z_$][A-Za-z0-9_$]*(?:\s*\.\s*[A-Za-z_$][A-Za-z0-9_$]*)*`
	callCandidatePattern     = regexp.MustCompile(`\b(` + solidCalleePattern + `)\s*(?:<[^>\n]+>)?\s*\(`)
	signalBindingPattern     = regexp.MustCompile(`(?m)(?:export\s+)?const\s*\[\s*([A-Za-z_$][A-Za-z0-9_$]*)[^]]*\]\s*=\s*(` + solidCalleePattern + `)\s*(?:<[^>\n]+>)?\s*\(`)
	signalSetterPattern      = regexp.MustCompile(`(?m)(?:export\s+)?const\s*\[\s*(?:[A-Za-z_$][A-Za-z0-9_$]*)?\s*,\s*([A-Za-z_$][A-Za-z0-9_$]*)[^]]*\]\s*=\s*(` + solidCalleePattern + `)\s*(?:<[^>\n]+>)?\s*\(`)
	storeSetterPattern       = regexp.MustCompile(`(?m)(?:export\s+)?const\s*\[\s*[A-Za-z_$][A-Za-z0-9_$]*\s*,\s*([A-Za-z_$][A-Za-z0-9_$]*)[^]]*\]\s*=\s*(` + solidCalleePattern + `)\s*(?:<[^>\n]+>)?\s*\(`)
	asyncDirectPattern       = regexp.MustCompile(`(?m)(?:export\s+)?const\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*(` + solidCalleePattern + `)\s*(?:<[^>\n]+>)?\s*\(`)
	storeBindingPattern      = regexp.MustCompile(`(?m)(?:export\s+)?const\s*\[\s*([A-Za-z_$][A-Za-z0-9_$]*)[^]]*\]\s*=\s*(` + solidCalleePattern + `)\s*(?:<[^>\n]+>)?\s*\(`)
	functionPattern          = regexp.MustCompile(`(?m)(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*`)
	arrowFunctionPattern     = regexp.MustCompile(`(?m)(?:export\s+)?const\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*`)
	exportConstPattern       = regexp.MustCompile(`(?m)export\s+const\s+`)
	exportClassPattern       = regexp.MustCompile(`(?m)export\s+class\s+([A-Za-z_$][A-Za-z0-9_$]*)`)
	exportListPattern        = regexp.MustCompile(`(?m)export\s*\{([^}]*)\}`)
	parameterPattern         = regexp.MustCompile(`^\s*([A-Za-z_$][A-Za-z0-9_$]*)`)
	identifierPattern        = regexp.MustCompile(`^[A-Za-z_$][A-Za-z0-9_$]*$`)
	returnedArrowPattern     = regexp.MustCompile(`return\s+(?:async\s+)?(?:[A-Za-z_$][A-Za-z0-9_$]*|\([^)]*\))\s*=>`)
	factoryBindingPattern    = regexp.MustCompile(`(?m)(?:export\s+)?const\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\(`)
	namedImportPattern       = regexp.MustCompile(`(?m)import\s*\{([^}]*)\}\s*from\s*["']([^"']+)["']`)
	importSpecifierPattern   = regexp.MustCompile(`^\s*([A-Za-z_$][A-Za-z0-9_$]*)(?:\s+as\s+([A-Za-z_$][A-Za-z0-9_$]*))?\s*$`)
	jsxPattern               = regexp.MustCompile(`<[A-Za-z]`)
	literalExpressionPattern = regexp.MustCompile(`^(?:[-+]?\d|["'` + "`" + `]|\(\s*\{|true\b|false\b|null\b)`)
	bodyDestructurePattern   = regexp.MustCompile(`(?s)(?:const|let|var)\s*\{[^{}]*\}\s*=\s*$`)
	aliasAssignmentPattern   = regexp.MustCompile(`(?m)\bconst\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*;`)
	constAssignmentPattern   = regexp.MustCompile(`\bconst\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*$`)
	ifStatementPattern       = regexp.MustCompile(`\bif\s*\(`)
	functionKeywordPattern   = regexp.MustCompile(`\bfunction\b`)
	awaitKeywordPattern      = regexp.MustCompile(`\bawait\b`)
	controlStatementPattern  = regexp.MustCompile(`^(?:if|for|while|switch|try|catch)\b`)
	identifierCallPattern    = regexp.MustCompile(`\b[A-Za-z_$][A-Za-z0-9_$]*\s*\(`)
	syncTruePattern          = regexp.MustCompile(`\bsync\s*:\s*true\b`)
	ownedWriteTruePattern    = regexp.MustCompile(`\bownedWrite\s*:\s*true\b`)
)

// Build joins Type Facts provenance, function/callback flow, and original-source
// compiler execution regions without exposing either compiler's AST.
func Build(
	ctx context.Context,
	facts typefacts.Project,
	sourceFiles []typefacts.SourceFile,
	executionMaps map[string]compilerfacts.ExecutionMap,
) (Program, error) {
	return BuildWithContracts(ctx, facts, sourceFiles, executionMaps, nil)
}

func BuildWithContracts(
	ctx context.Context,
	facts typefacts.Project,
	sourceFiles []typefacts.SourceFile,
	executionMaps map[string]compilerfacts.ExecutionMap,
	packageContracts []contracts.Contract,
) (Program, error) {
	ctx = withSolidCallCache(ctx)
	sources := make(map[string][]byte, len(sourceFiles))
	functions := make(map[string][]sourceFunction, len(sourceFiles))
	program := Program{Reads: []ReactiveRead{}, Writes: []ReactiveWrite{}, LeafOperations: []LeafOwnerOperation{}, MissingOwners: []OwnerRequirement{}, AsyncReads: []AsyncRead{}, InvalidCleanupReturns: []InvalidCleanupReturn{}, DirectiveCreations: []PrimitiveCreation{}, StaticViolations: []StaticViolation{}, Functions: []Function{}, ExportAliases: map[string]FunctionID{}}
	for _, file := range sourceFiles {
		path := filepath.Clean(file.Path)
		sources[path] = append([]byte(nil), file.Source...)
		for _, match := range exportConstPattern.FindAllSubmatchIndex(file.Source, -1) {
			end := statementEnd(file.Source, match[1])
			if end < 0 {
				continue
			}
			for _, span := range splitArguments(file.Source, match[1], end) {
				start, finish, ok := declarationName(file.Source, span.start, span.end)
				if !ok {
					continue
				}
				if _, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: start, EndByte: finish}); err != nil {
					if errors.Is(err, typefacts.ErrNotFound) {
						continue
					}
					return Program{}, fmt.Errorf("resolve exported value in %s: %w", path, err)
				}
				program.ExportedValues = append(program.ExportedValues, string(file.Source[start:finish]))
			}
		}
		for _, match := range exportClassPattern.FindAllSubmatchIndex(file.Source, -1) {
			if _, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: match[2], EndByte: match[3]}); err != nil {
				if errors.Is(err, typefacts.ErrNotFound) {
					continue
				}
				return Program{}, fmt.Errorf("resolve exported class in %s: %w", path, err)
			}
			program.ExportedValues = append(program.ExportedValues, string(file.Source[match[2]:match[3]]))
		}
		for _, match := range exportListPattern.FindAllSubmatchIndex(file.Source, -1) {
			for _, span := range splitArguments(file.Source, match[2], match[3]) {
				specifier := importSpecifierPattern.FindSubmatchIndex(file.Source[span.start:span.end])
				if specifier == nil {
					continue
				}
				localStart := span.start + specifier[2]
				localEnd := span.start + specifier[3]
				exportName := string(file.Source[localStart:localEnd])
				if specifier[4] >= 0 {
					exportName = string(file.Source[span.start+specifier[4] : span.start+specifier[5]])
				}
				symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: localStart, EndByte: localEnd})
				if err != nil {
					if errors.Is(err, typefacts.ErrNotFound) {
						continue
					}
					return Program{}, fmt.Errorf("resolve re-export %s in %s: %w", exportName, path, err)
				}
				program.ExportAliases[exportName] = FunctionID(canonicalSymbol(ctx, facts, symbol))
			}
		}
		declared, err := declaredFunctions(ctx, facts, path, file.Source, len(program.Functions))
		if err != nil {
			return Program{}, err
		}
		functions[path] = declared
		for _, function := range declared {
			program.Functions = append(program.Functions, Function{
				ID: function.id, Name: function.name, Exported: function.exported, Async: function.async, Rendering: function.rendering,
				Reads: []ReactiveRead{}, Writes: []ReactiveWrite{}, ReturnedReads: []ReactiveRead{}, Calls: []FunctionCall{}, CallbackInvocations: []CallbackInvocation{},
			})
		}
	}
	addIncompleteExecutionMaps(sourceFiles, executionMaps, &program)
	computationRegions := make(map[string][]allowedWriteRegion, len(sourceFiles))
	effectApplyRegions := make(map[string][]compilerfacts.Span, len(sourceFiles))
	for _, file := range sourceFiles {
		path := filepath.Clean(file.Path)
		regions, err := readExecutionRegions(ctx, facts, path, file.Source)
		if err != nil {
			return Program{}, err
		}
		computationRegions[path] = regions
		for _, primitive := range []string{"createEffect", "createRenderEffect"} {
			apply, applyErr := solidCallArgumentRegions(ctx, facts, path, file.Source, primitive, 1)
			if applyErr != nil {
				return Program{}, applyErr
			}
			effectApplyRegions[path] = append(effectApplyRegions[path], apply...)
		}
	}

	for _, file := range sourceFiles {
		path := filepath.Clean(file.Path)
		matches := signalBindingPattern.FindAllSubmatchIndex(file.Source, -1)
		for _, match := range matches {
			accessorStart, accessorEnd := match[2], match[3]
			calleeStart, calleeEnd := match[4], match[5]
			call, err := facts.ResolvedCall(ctx, typefacts.Location{
				Path: path, StartByte: calleeStart, EndByte: calleeEnd,
			})
			if err != nil {
				if errors.Is(err, typefacts.ErrNotFound) {
					continue
				}
				return Program{}, fmt.Errorf("resolve createSignal candidate in %s: %w", path, err)
			}
			if !isSolidCreateSignal(ctx, facts, call.Target) {
				continue
			}

			accessorLocation := typefacts.Location{
				Path: path, StartByte: accessorStart, EndByte: accessorEnd,
			}
			accessorSymbol, err := facts.SymbolAt(ctx, accessorLocation)
			if err != nil {
				return Program{}, fmt.Errorf("resolve signal accessor in %s: %w", path, err)
			}
			declarations, err := facts.Declarations(ctx, accessorSymbol)
			if err != nil {
				return Program{}, fmt.Errorf("resolve signal accessor declaration in %s: %w", path, err)
			}
			if len(declarations) == 0 {
				return Program{}, fmt.Errorf("resolve signal accessor declaration in %s: no declarations", path)
			}
			declarationSource := sources[filepath.Clean(declarations[0].Location.Path)]
			declaration := sourceLocation(declarations[0].Location, declarationSource)
			accessor := string(file.Source[accessorStart:accessorEnd])

			references, err := facts.References(ctx, accessorSymbol)
			if err != nil {
				return Program{}, fmt.Errorf("find references to %s: %w", accessor, err)
			}
			for _, reference := range references {
				reference.Path = filepath.Clean(reference.Path)
				source, ok := sources[reference.Path]
				if !ok {
					continue
				}
				function, ok := functionContext(functions[reference.Path], reference.StartByte)
				if !ok {
					continue
				}
				readEnd, called := accessorCallEnd(source, reference.EndByte)
				if !called {
					if isReturnedIdentifier(source, reference.StartByte, reference.EndByte) {
						program.Functions[function.programIndex].ReturnedReads = append(
							program.Functions[function.programIndex].ReturnedReads,
							ReactiveRead{
								Kind: ReactiveAccessor, Accessor: accessor,
								Location: sourceLocation(reference, source), Declaration: declaration,
								Execution: ExecutionInline, Context: function.name,
							},
						)
					}
					continue
				}
				readSpan := compilerfacts.Span{Start: reference.StartByte, End: readEnd}
				execution := executionRoleWithComputations(executionMaps[reference.Path], computationRegions[reference.Path], readSpan)
				inEffectApply := containedByAny(effectApplyRegions[reference.Path], compilerfacts.Span{Start: reference.StartByte, End: reference.EndByte})
				if !function.rendering && execution == ExecutionUntrackedRendering && !inEffectApply {
					execution = ExecutionInline
				}
				read := ReactiveRead{
					Kind:        ReactiveAccessor,
					Accessor:    accessor,
					Location:    sourceLocation(typefacts.Location{Path: reference.Path, StartByte: reference.StartByte, EndByte: readEnd}, source),
					Declaration: declaration,
					Execution:   execution,
					Context:     function.name,
				}
				if inEffectApply {
					read.Context = "createEffect apply callback"
					read.Execution = ExecutionUntrackedRendering
				}
				if containsOffset(function.returnedClosures, reference.StartByte) {
					program.Functions[function.programIndex].ReturnedReads = append(program.Functions[function.programIndex].ReturnedReads, read)
				} else if function.rendering || inEffectApply {
					program.Reads = append(program.Reads, read)
				} else {
					program.Functions[function.programIndex].Reads = append(program.Functions[function.programIndex].Reads, read)
				}
			}
		}
	}
	if err := addSignalWrites(ctx, facts, sourceFiles, sources, functions, executionMaps, &program); err != nil {
		return Program{}, err
	}
	if err := addTypedAccessorReads(ctx, facts, sourceFiles, sources, functions, executionMaps, computationRegions, &program); err != nil {
		return Program{}, err
	}
	if err := addComponentPropReads(ctx, facts, sourceFiles, functions, executionMaps, computationRegions, &program); err != nil {
		return Program{}, err
	}
	markConditionalComponentReads(sourceFiles, functions, &program)
	if err := addReactiveReadsAfterAwait(ctx, facts, sourceFiles, sources, &program); err != nil {
		return Program{}, err
	}
	if err := addRefreshAndAffectsDiagnostics(ctx, facts, sourceFiles, sources, functions, executionMaps, &program); err != nil {
		return Program{}, err
	}
	if err := addActionInvocations(ctx, facts, sourceFiles, sources, functions, executionMaps, &program); err != nil {
		return Program{}, err
	}
	if err := addLeafOwnerOperations(ctx, facts, sourceFiles, &program); err != nil {
		return Program{}, err
	}
	if err := addMissingOwners(ctx, facts, sourceFiles, functions, executionMaps, &program); err != nil {
		return Program{}, err
	}
	if err := addAsyncReads(ctx, facts, sourceFiles, sources, functions, executionMaps, &program); err != nil {
		return Program{}, err
	}
	if err := addControlFlowParameterReads(ctx, facts, sourceFiles, sources, functions, executionMaps, &program); err != nil {
		return Program{}, err
	}
	if err := addCleanupReturnChecks(ctx, facts, sourceFiles, functions, executionMaps, &program); err != nil {
		return Program{}, err
	}
	if err := addPrimitiveCreations(ctx, facts, sourceFiles, functions, executionMaps, &program); err != nil {
		return Program{}, err
	}
	if err := addStaticAPIDiagnostics(ctx, facts, sourceFiles, &program); err != nil {
		return Program{}, err
	}
	for _, file := range sourceFiles {
		path := filepath.Clean(file.Path)
		matches := storeBindingPattern.FindAllSubmatchIndex(file.Source, -1)
		for _, match := range matches {
			storeStart, storeEnd := match[2], match[3]
			calleeStart, calleeEnd := match[4], match[5]
			call, err := facts.ResolvedCall(ctx, typefacts.Location{
				Path: path, StartByte: calleeStart, EndByte: calleeEnd,
			})
			if err != nil {
				if errors.Is(err, typefacts.ErrNotFound) {
					continue
				}
				return Program{}, fmt.Errorf("resolve createStore candidate in %s: %w", path, err)
			}
			if !isSolidPrimitive(ctx, facts, call.Target, "createStore") {
				continue
			}
			storeSymbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: storeStart, EndByte: storeEnd})
			if err != nil {
				return Program{}, fmt.Errorf("resolve store in %s: %w", path, err)
			}
			declarations, err := facts.Declarations(ctx, storeSymbol)
			if err != nil {
				return Program{}, fmt.Errorf("resolve store declaration in %s: %w", path, err)
			}
			if len(declarations) == 0 {
				return Program{}, fmt.Errorf("resolve store declaration in %s: no declarations", path)
			}
			declarationSource := sources[filepath.Clean(declarations[0].Location.Path)]
			declaration := sourceLocation(declarations[0].Location, declarationSource)
			references, err := facts.References(ctx, storeSymbol)
			if err != nil {
				return Program{}, fmt.Errorf("find references to store in %s: %w", path, err)
			}
			for _, reference := range references {
				reference.Path = filepath.Clean(reference.Path)
				source, ok := sources[reference.Path]
				if !ok {
					continue
				}
				function, ok := functionContext(functions[reference.Path], reference.StartByte)
				if !ok {
					continue
				}
				readEnd, property := propertyAccessEnd(source, reference.EndByte)
				if !property {
					if isReturnedIdentifier(source, reference.StartByte, reference.EndByte) {
						program.Functions[function.programIndex].ReturnedReads = append(
							program.Functions[function.programIndex].ReturnedReads,
							ReactiveRead{
								Kind: ReactiveStorePath, Accessor: string(file.Source[storeStart:storeEnd]),
								Location: sourceLocation(reference, source), Declaration: declaration,
								Execution: ExecutionInline, Context: function.name,
							},
						)
					}
					continue
				}
				readSpan := compilerfacts.Span{Start: reference.StartByte, End: readEnd}
				execution := executionRoleWithComputations(executionMaps[reference.Path], computationRegions[reference.Path], readSpan)
				inEffectApply := containedByAny(effectApplyRegions[reference.Path], compilerfacts.Span{Start: reference.StartByte, End: reference.EndByte})
				if !function.rendering && execution == ExecutionUntrackedRendering && !inEffectApply {
					execution = ExecutionInline
				}
				read := ReactiveRead{
					Kind:        ReactiveStorePath,
					Accessor:    string(source[reference.StartByte:readEnd]),
					Location:    sourceLocation(typefacts.Location{Path: reference.Path, StartByte: reference.StartByte, EndByte: readEnd}, source),
					Declaration: declaration,
					Execution:   execution,
					Context:     function.name,
				}
				if inEffectApply {
					read.Context = "createEffect apply callback"
					read.Execution = ExecutionUntrackedRendering
				}
				if function.rendering || inEffectApply {
					program.Reads = append(program.Reads, read)
				} else {
					program.Functions[function.programIndex].Reads = append(program.Functions[function.programIndex].Reads, read)
				}
			}
		}
	}
	targets := declaredCallableTargets(functions)
	contractTargets, err := addContractExports(ctx, facts, sourceFiles, packageContracts, &program)
	if err != nil {
		return Program{}, err
	}
	targets = append(targets, contractTargets...)
	if err := propagateReturnedFactoryCalls(ctx, facts, sources, functions, targets, &program); err != nil {
		return Program{}, err
	}
	factoryTargets, err := addFactoryInstances(ctx, facts, sourceFiles, sources, functions, executionMaps, &program)
	if err != nil {
		return Program{}, err
	}
	targets = append(targets, factoryTargets...)
	if err := addFunctionCalls(ctx, facts, sources, functions, executionMaps, targets, &program); err != nil {
		return Program{}, err
	}
	sort.Slice(program.Reads, func(i, j int) bool {
		left, right := program.Reads[i].Location, program.Reads[j].Location
		if left.Path != right.Path {
			return left.Path < right.Path
		}
		return left.StartByte < right.StartByte
	})
	return program, nil
}

func addControlFlowParameterReads(ctx context.Context, facts typefacts.Project, files []typefacts.SourceFile, sources map[string][]byte, functions map[string][]sourceFunction, executionMaps map[string]compilerfacts.ExecutionMap, program *Program) error {
	for _, file := range files {
		path := filepath.Clean(file.Path)
		for _, callback := range executionMaps[path].CallbackRoles {
			if callback.Role != compilerfacts.CallbackRender {
				continue
			}
			builtIn := ""
			for _, operation := range executionMaps[path].JsxOperations {
				if operation.Kind != "component-invocation" || !contains(operation.Span, callback.Span) || operation.Span.Start+1 >= len(file.Source) {
					continue
				}
				builtIn = jsxSolidPrimitive(ctx, facts, path, file.Source, operation.Span, "For", "Show", "Match")
				break
			}
			count := 0
			switch builtIn {
			case "For":
				count = 2
			case "Show", "Match":
				count = 1
			}
			if count == 0 {
				continue
			}
			parameters := arrowParameterSpans(file.Source, callback.Span)
			if len(parameters) > count {
				parameters = parameters[:count]
			}
			for _, parameter := range parameters {
				symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: parameter.start, EndByte: parameter.end})
				if err != nil {
					continue
				}
				declaration := sourceLocation(typefacts.Location{Path: path, StartByte: parameter.start, EndByte: parameter.end}, file.Source)
				references, err := facts.References(ctx, symbol)
				if err != nil {
					return fmt.Errorf("find control-flow parameter references in %s: %w", path, err)
				}
				for _, reference := range references {
					reference.Path = filepath.Clean(reference.Path)
					if reference.Path != path || reference.StartByte < callback.Span.Start || reference.StartByte >= callback.Span.End {
						continue
					}
					source := sources[path]
					end, called := accessorCallEnd(source, reference.EndByte)
					if !called {
						continue
					}
					span := compilerfacts.Span{Start: reference.StartByte, End: end}
					function, ok := functionContext(functions[path], reference.StartByte)
					if !ok {
						continue
					}
					program.Reads = append(program.Reads, ReactiveRead{
						Kind: ReactiveAccessor, Accessor: string(source[reference.StartByte:reference.EndByte]),
						Location:    sourceLocation(typefacts.Location{Path: path, StartByte: reference.StartByte, EndByte: end}, source),
						Declaration: declaration, Execution: executionRole(executionMaps[path], span), Context: function.name,
					})
				}
			}
		}
	}
	return nil
}

func arrowParameterSpans(source []byte, callback compilerfacts.Span) []byteSpan {
	if callback.Start < 0 || callback.End > len(source) {
		return nil
	}
	arrowRelative := bytes.Index(source[callback.Start:callback.End], []byte("=>"))
	if arrowRelative < 0 {
		return nil
	}
	start, end := callback.Start, callback.Start+arrowRelative
	for start < end && (source[start] == ' ' || source[start] == '\t' || source[start] == '\r' || source[start] == '\n' || source[start] == '{') {
		start++
	}
	for end > start && (source[end-1] == ' ' || source[end-1] == '\t' || source[end-1] == '\r' || source[end-1] == '\n') {
		end--
	}
	if start < end && source[start] == '(' && source[end-1] == ')' {
		return splitArguments(source, start+1, end-1)
	}
	if start < end {
		return []byteSpan{{start: start, end: end}}
	}
	return nil
}

func addCleanupReturnChecks(ctx context.Context, facts typefacts.Project, files []typefacts.SourceFile, functions map[string][]sourceFunction, executionMaps map[string]compilerfacts.ExecutionMap, program *Program) error {
	callbacks := []struct {
		name     string
		argument int
	}{
		{name: "onSettled", argument: 0},
		{name: "createTrackedEffect", argument: 0},
		{name: "createEffect", argument: 1},
		{name: "createRenderEffect", argument: 1},
		{name: "createReaction", argument: 0},
	}
	for _, file := range files {
		path := filepath.Clean(file.Path)
		ownerRegions, err := ownerProvidingRegions(ctx, facts, path, file.Source)
		if err != nil {
			return err
		}
		imperativeRegions, err := allowedWriteRegions(ctx, facts, path, file.Source)
		if err != nil {
			return err
		}
		for _, callback := range callbacks {
			calls, err := solidSourceCalls(ctx, facts, path, file.Source, callback.name)
			if err != nil {
				return err
			}
			for _, call := range calls {
				if callback.argument >= len(call.arguments) {
					continue
				}
				argument := call.arguments[callback.argument]
				region := compilerfacts.Span{Start: argument.start, End: argument.end}
				for _, result := range resolvedCleanupReturns(ctx, facts, path, file.Source, region, functions[path]) {
					expression, status := result.span, result.status
					switch status {
					case cleanupReturnFunction:
						if callback.name == "onSettled" {
							function, inFunction := functionContext(functions[path], region.Start)
							functionID := FunctionID("")
							if inFunction {
								functionID = function.id
							}
							owned := false
							callSpan := call.callee
							for _, owner := range ownerRegions {
								if contains(owner, callSpan) {
									owned = true
									break
								}
							}
							unowned := unownedExecution(executionMaps[path], imperativeRegions, callSpan)
							if !owned || unowned || inFunction {
								program.MissingOwners = append(program.MissingOwners, OwnerRequirement{
									Operation: "settled-cleanup", Function: functionID, Unowned: unowned,
									Location: sourceLocation(typefacts.Location{Path: path, StartByte: region.Start, EndByte: region.End}, file.Source),
								})
							}
						}
					case cleanupReturnInvalid:
						program.InvalidCleanupReturns = append(program.InvalidCleanupReturns, InvalidCleanupReturn{
							Primitive: callback.name,
							Location:  sourceLocation(typefacts.Location{Path: path, StartByte: expression.Start, EndByte: expression.End}, file.Source),
						})
					case cleanupReturnUnknown:
						program.Unresolved = append(program.Unresolved, UnresolvedObligation{
							Message:  fmt.Sprintf("cannot prove that %s callback returns only a cleanup function or undefined", callback.name),
							Location: sourceLocation(typefacts.Location{Path: path, StartByte: expression.Start, EndByte: expression.End}, file.Source),
							ID:       "SC9002", Rule: "cleanup-return-unresolved",
						})
					}
				}
			}
		}
	}
	return nil
}

type cleanupReturnStatus uint8

const (
	cleanupReturnValid cleanupReturnStatus = iota
	cleanupReturnFunction
	cleanupReturnInvalid
	cleanupReturnUnknown
)

type cleanupReturnResult struct {
	span   compilerfacts.Span
	status cleanupReturnStatus
}

func callbackCleanupReturns(source []byte, callback compilerfacts.Span) []cleanupReturnResult {
	if callback.Start < 0 || callback.End > len(source) || callback.Start >= callback.End {
		return []cleanupReturnResult{{callback, cleanupReturnUnknown}}
	}
	text := source[callback.Start:callback.End]
	arrow := bytes.Index(text, []byte("=>"))
	if arrow < 0 {
		return []cleanupReturnResult{{callback, cleanupReturnUnknown}}
	}
	if strings.HasPrefix(strings.TrimSpace(string(text[:arrow])), "async") {
		return []cleanupReturnResult{{callback, cleanupReturnInvalid}}
	}
	start := callback.Start + arrow + 2
	end := callback.End
	for start < end && (source[start] == ' ' || source[start] == '\t' || source[start] == '\r' || source[start] == '\n') {
		start++
	}
	for end > start && (source[end-1] == ' ' || source[end-1] == '\t' || source[end-1] == '\r' || source[end-1] == '\n') {
		end--
	}
	if start < end && source[start] == '{' {
		return blockCleanupReturns(source, start, end)
	}
	span := compilerfacts.Span{Start: start, End: end}
	return []cleanupReturnResult{{span, classifyCleanupExpression(source[start:end])}}
}

func blockCleanupReturns(source []byte, start, end int) []cleanupReturnResult {
	results := make([]cleanupReturnResult, 0)
	brace := 0
	for index := start; index < end; index++ {
		switch source[index] {
		case '{':
			brace++
		case '}':
			brace--
		}
		if brace != 1 || index+6 > end || string(source[index:index+6]) != "return" {
			continue
		}
		if index > start && isIdentifierByte(source[index-1]) {
			continue
		}
		expressionStart := index + 6
		for expressionStart < end && (source[expressionStart] == ' ' || source[expressionStart] == '\t') {
			expressionStart++
		}
		expressionEnd := expressionStart
		paren, bracket, innerBrace := 0, 0, 0
		for expressionEnd < end {
			character := source[expressionEnd]
			if character == ';' && paren == 0 && bracket == 0 && innerBrace == 0 {
				break
			}
			switch character {
			case '(':
				paren++
			case ')':
				paren--
			case '[':
				bracket++
			case ']':
				bracket--
			case '{':
				innerBrace++
			case '}':
				if innerBrace > 0 {
					innerBrace--
				}
			}
			expressionEnd++
		}
		span := compilerfacts.Span{Start: expressionStart, End: expressionEnd}
		results = append(results, cleanupReturnResult{span, classifyCleanupExpression(source[expressionStart:expressionEnd])})
		index = expressionEnd
	}
	return results
}

func resolvedCleanupReturns(ctx context.Context, facts typefacts.Project, path string, source []byte, callback compilerfacts.Span, functions []sourceFunction) []cleanupReturnResult {
	results := callbackCleanupReturns(source, callback)
	start, end := trimByteSpan(source, callback.Start, callback.End)
	if len(results) == 1 && results[0].status == cleanupReturnUnknown && identifierPattern.Match(source[start:end]) {
		if symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: start, EndByte: end}); err == nil {
			id := FunctionID(canonicalSymbol(ctx, facts, symbol))
			for _, function := range functions {
				if function.id == id {
					results = blockCleanupReturns(source, function.bodyStart, function.bodyEnd)
					break
				}
			}
		}
	}
	for index := range results {
		if results[index].status != cleanupReturnUnknown {
			continue
		}
		expressionStart, expressionEnd := trimByteSpan(source, results[index].span.Start, results[index].span.End)
		calleeEnd := expressionStart
		for calleeEnd < expressionEnd && (isIdentifierByte(source[calleeEnd]) || source[calleeEnd] == '.') {
			calleeEnd++
		}
		if calleeEnd > expressionStart {
			cursor := calleeEnd
			for cursor < expressionEnd && (source[cursor] == ' ' || source[cursor] == '\t' || source[cursor] == '\r' || source[cursor] == '\n') {
				cursor++
			}
			if cursor < expressionEnd && source[cursor] == '(' {
				if call, err := facts.ResolvedCall(ctx, typefacts.Location{Path: path, StartByte: expressionStart, EndByte: calleeEnd}); err == nil {
					results[index].status = cleanupStatusFromTypeText(call.ReturnTypeText)
					if results[index].status != cleanupReturnUnknown {
						continue
					}
				}
			}
		}
		if !identifierPattern.Match(source[expressionStart:expressionEnd]) {
			continue
		}
		symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: expressionStart, EndByte: expressionEnd})
		if err != nil {
			continue
		}
		id := FunctionID(canonicalSymbol(ctx, facts, symbol))
		for _, function := range functions {
			if function.id == id {
				results[index].status = cleanupReturnFunction
				break
			}
		}
		if results[index].status == cleanupReturnUnknown {
			declarations, err := facts.Declarations(ctx, symbol)
			if err == nil {
				for _, declaration := range declarations {
					if filepath.Clean(declaration.Location.Path) != filepath.Clean(path) {
						continue
					}
					end := statementEnd(source, declaration.Location.EndByte)
					if end > declaration.Location.EndByte && bytes.Contains(source[declaration.Location.EndByte:end], []byte("=>")) {
						results[index].status = cleanupReturnFunction
						break
					}
				}
			}
		}
	}
	return results
}

func cleanupStatusFromTypeText(value string) cleanupReturnStatus {
	typeText := strings.TrimSpace(value)
	if typeText == "void" || typeText == "undefined" || typeText == "never" {
		return cleanupReturnValid
	}
	if typeText == "VoidFunction" || strings.Contains(typeText, "=>") {
		for _, invalid := range []string{"Promise", "AsyncIterable", "number", "string", "boolean", "null", "{"} {
			if strings.Contains(typeText, invalid) {
				return cleanupReturnUnknown
			}
		}
		return cleanupReturnFunction
	}
	return cleanupReturnUnknown
}

func classifyCleanupExpression(value []byte) cleanupReturnStatus {
	expression := strings.TrimSpace(string(value))
	if expression == "" || expression == "undefined" || strings.HasPrefix(expression, "void ") {
		return cleanupReturnValid
	}
	if strings.HasPrefix(expression, "function") || strings.Contains(expression, "=>") {
		return cleanupReturnFunction
	}
	if literalExpressionPattern.MatchString(expression) {
		return cleanupReturnInvalid
	}
	return cleanupReturnUnknown
}

func isIdentifierByte(value byte) bool {
	return value == '_' || value == '$' || value >= '0' && value <= '9' || value >= 'A' && value <= 'Z' || value >= 'a' && value <= 'z'
}

func addTypedAccessorReads(
	ctx context.Context,
	facts typefacts.Project,
	files []typefacts.SourceFile,
	sources map[string][]byte,
	functions map[string][]sourceFunction,
	executionMaps map[string]compilerfacts.ExecutionMap,
	computationRegions map[string][]allowedWriteRegion,
	program *Program,
) error {
	describer, ok := facts.(typefacts.TypeDescriber)
	if !ok {
		return nil
	}
	for _, file := range files {
		path := filepath.Clean(file.Path)
		applyRegions, err := solidCallArgumentRegions(ctx, facts, path, file.Source, "createEffect", 1)
		if err != nil {
			return err
		}
		type candidate struct {
			start, end, callEnd int
			target              typefacts.SymbolID
		}
		candidates := make([]candidate, 0)
		if discoverer, available := facts.(typefacts.CallDiscoverer); available {
			calls, err := sourceCallFacts(ctx, discoverer, path)
			if err != nil {
				return err
			}
			for _, call := range calls {
				candidates = append(candidates, candidate{start: call.Callee.StartByte, end: call.Callee.EndByte, callEnd: call.Location.EndByte, target: call.Target})
			}
		} else {
			for _, match := range callCandidatePattern.FindAllSubmatchIndex(file.Source, -1) {
				open := match[1] - 1
				close := matchingBrace(file.Source, open, '(', ')')
				if close >= open {
					candidates = append(candidates, candidate{start: match[2], end: match[3], callEnd: close + 1})
				}
			}
		}
		for _, candidate := range candidates {
			start, end := candidate.start, candidate.end
			typeStart := start
			if dot := bytes.LastIndexByte(file.Source[start:end], '.'); dot >= 0 {
				typeStart = start + dot + 1
			}
			descriptor, describeErr := describeTarget(ctx, describer, candidate.target, typefacts.Location{Path: path, StartByte: typeStart, EndByte: end})
			if describeErr != nil || !solidAccessorDescriptor(descriptor, sources) {
				continue
			}
			span := compilerfacts.Span{Start: start, End: end}
			declarationSource := sources[filepath.Clean(descriptor.AliasDeclarations[0].Location.Path)]
			read := ReactiveRead{Kind: ReactiveAccessor, Accessor: string(file.Source[start:end]), Location: sourceLocation(typefacts.Location{Path: path, StartByte: start, EndByte: candidate.callEnd}, file.Source), Declaration: sourceLocation(descriptor.AliasDeclarations[0].Location, declarationSource), Execution: executionRoleWithComputations(executionMaps[path], computationRegions[path], span)}
			inApply := false
			for _, region := range applyRegions {
				if contains(region, span) {
					inApply = true
					read.Context = "createEffect apply callback"
					read.Execution = ExecutionUntrackedRendering
					break
				}
			}
			if duplicateReactiveRead(*program, read) {
				continue
			}
			if inApply {
				program.Reads = append(program.Reads, read)
				continue
			}
			if function, found := functionContext(functions[path], start); found {
				read.Context = function.name
				if function.rendering {
					program.Reads = append(program.Reads, read)
				} else {
					program.Functions[function.programIndex].Reads = append(program.Functions[function.programIndex].Reads, read)
				}
			}
		}
	}
	return nil
}

func describeTarget(ctx context.Context, describer typefacts.TypeDescriber, target typefacts.SymbolID, location typefacts.Location) (typefacts.TypeDescriptor, error) {
	cache := solidCallCacheFrom(ctx)
	if cache != nil && target != "" {
		if descriptor, ok := cache.descriptors[target]; ok {
			return descriptor, cache.describeErrs[target]
		}
	}
	descriptor, err := describer.DescribeTypeAt(ctx, location)
	if cache != nil && target != "" {
		cache.descriptors[target] = descriptor
		cache.describeErrs[target] = err
	}
	return descriptor, err
}

func solidAccessorDescriptor(descriptor typefacts.TypeDescriptor, sources map[string][]byte) bool {
	if descriptor.OriginModule == "solid-js" {
		return true
	}
	for _, declaration := range descriptor.AliasDeclarations {
		if declaration.Name != "Accessor" {
			continue
		}
		path := filepath.ToSlash(filepath.Clean(declaration.Location.Path))
		source := sources[filepath.Clean(declaration.Location.Path)]
		if strings.Contains(path, "/node_modules/solid-js/") || bytes.Contains(source, []byte(`module "solid-js"`)) || bytes.Contains(source, []byte(`module 'solid-js'`)) {
			return true
		}
	}
	return false
}

func addComponentPropReads(
	ctx context.Context,
	facts typefacts.Project,
	files []typefacts.SourceFile,
	functions map[string][]sourceFunction,
	executionMaps map[string]compilerfacts.ExecutionMap,
	computationRegions map[string][]allowedWriteRegion,
	program *Program,
) error {
	bodyDestructure := bodyDestructurePattern
	for _, file := range files {
		path := filepath.Clean(file.Path)
		for _, function := range functions[path] {
			if !function.rendering || len(function.parameterSpans) == 0 {
				continue
			}
			first := function.parameterSpans[0]
			parameterStart, parameterEnd := trimByteSpan(file.Source, first.start, first.end)
			if parameterStart >= parameterEnd {
				continue
			}
			if file.Source[parameterStart] == '{' {
				patternEnd := matchingBrace(file.Source, parameterStart, '{', '}')
				if patternEnd > parameterStart && patternEnd <= parameterEnd {
					fix := simpleComponentPropsParameterFix(ctx, facts, path, file.Source, function, parameterStart, patternEnd, executionMaps[path])
					fixes := []certification.Fix(nil)
					if fix != nil {
						fixes = []certification.Fix{*fix}
					}
					program.StaticViolations = append(program.StaticViolations, StaticViolation{
						ID: "SC1003", Rule: "component-props-destructure",
						Message:         "destructuring component props reads them outside tracking; keep the props object and read its properties inside JSX or a tracked computation",
						AnalysisContext: function.name,
						Location:        sourceLocation(typefacts.Location{Path: path, StartByte: parameterStart, EndByte: patternEnd}, file.Source),
						Fixes:           fixes,
					})
				}
				continue
			}
			if len(function.parameters) == 0 || function.parameters[0] == "" {
				continue
			}
			nameMatch := parameterPattern.FindSubmatchIndex(file.Source[parameterStart:parameterEnd])
			if nameMatch == nil {
				continue
			}
			nameStart := parameterStart + nameMatch[2]
			nameEnd := parameterStart + nameMatch[3]
			declaration := sourceLocation(typefacts.Location{Path: path, StartByte: nameStart, EndByte: nameEnd}, file.Source)
			propSymbols, err := componentPropAliasSymbols(ctx, facts, path, file.Source, function, function.parameters[0])
			if err != nil {
				return err
			}
			for _, propSymbol := range propSymbols {
				references, referenceErr := facts.References(ctx, propSymbol)
				if referenceErr != nil {
					return fmt.Errorf("find component props references for %s in %s: %w", function.name, path, referenceErr)
				}
				for _, reference := range references {
					if filepath.Clean(reference.Path) != path || reference.StartByte <= function.bodyStart || reference.StartByte >= function.bodyEnd {
						continue
					}
					statementStart := reference.StartByte
					for statementStart > function.bodyStart+1 {
						previous := file.Source[statementStart-1]
						if previous == ';' || previous == '\n' {
							break
						}
						statementStart--
					}
					if bodyDestructure.Match(file.Source[statementStart:reference.StartByte]) {
						open := bytes.IndexByte(file.Source[statementStart:reference.StartByte], '{')
						if open >= 0 {
							open += statementStart
							close := matchingBrace(file.Source, open, '{', '}')
							if close > open && close <= reference.StartByte {
								program.StaticViolations = append(program.StaticViolations, StaticViolation{
									ID: "SC1003", Rule: "component-props-destructure",
									Message:         "destructuring component props reads them outside tracking; keep the props object and read its properties inside JSX or a tracked computation",
									AnalysisContext: function.name,
									Location:        sourceLocation(typefacts.Location{Path: path, StartByte: open, EndByte: close}, file.Source),
								})
							}
						}
						continue
					}
					readEnd, property := propertyAccessEnd(file.Source, reference.EndByte)
					if !property || reactiveReadStartsAt(*program, path, reference.StartByte) {
						continue
					}
					span := compilerfacts.Span{Start: reference.StartByte, End: readEnd}
					execution := executionRoleWithComputations(executionMaps[path], computationRegions[path], span)
					// Unknown nested callbacks execute at a time selected by their caller.
					// Without a compiler role, treating their props reads as render-time
					// snapshots would create false positives for timers and user APIs.
					if execution == ExecutionUntrackedRendering && insideNestedFunction(file.Source, function.bodyStart, function.bodyEnd, reference.StartByte) {
						continue
					}
					program.Reads = append(program.Reads, ReactiveRead{
						Kind: ReactiveProps, Accessor: string(file.Source[reference.StartByte:readEnd]),
						Location:    sourceLocation(typefacts.Location{Path: path, StartByte: reference.StartByte, EndByte: readEnd}, file.Source),
						Declaration: declaration,
						Execution:   execution,
						Context:     function.name,
					})
				}
			}
		}
	}
	return nil
}

func simpleComponentPropsParameterFix(
	ctx context.Context,
	facts typefacts.Project,
	path string,
	source []byte,
	function sourceFunction,
	patternStart, patternEnd int,
	executionMap compilerfacts.ExecutionMap,
) *certification.Fix {
	if patternStart < 0 || patternEnd > len(source) || patternEnd-patternStart < 2 {
		return nil
	}
	innerStart, innerEnd := patternStart+1, patternEnd-1
	bindings := splitArguments(source, innerStart, innerEnd)
	if len(bindings) == 0 {
		return nil
	}
	type binding struct {
		name       string
		start, end int
	}
	parsed := make([]binding, 0, len(bindings))
	for _, item := range bindings {
		start, end := trimByteSpan(source, item.start, item.end)
		if start >= end || !identifierOnly(source[start:end]) {
			return nil
		}
		parsed = append(parsed, binding{name: string(source[start:end]), start: start, end: end})
	}

	parameterName := availableIdentifier(source[function.bodyStart:function.bodyEnd], "props")
	edits := []certification.TextEdit{{
		Location: sourceLocation(typefacts.Location{Path: path, StartByte: patternStart, EndByte: patternEnd}, source),
		NewText:  parameterName,
	}}
	bodyReferences := 0
	for _, item := range parsed {
		symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: item.start, EndByte: item.end})
		if err != nil {
			return nil
		}
		references, err := facts.References(ctx, symbol)
		if err != nil {
			return nil
		}
		for _, reference := range references {
			if filepath.Clean(reference.Path) != path {
				return nil
			}
			if reference.StartByte >= patternStart && reference.EndByte <= patternEnd {
				continue
			}
			if reference.StartByte < function.bodyStart || reference.EndByte > function.bodyEnd ||
				reference.StartByte < 0 || reference.EndByte > len(source) ||
				string(source[reference.StartByte:reference.EndByte]) != item.name ||
				!safePropsFixExpression(executionMap, compilerfacts.Span{Start: reference.StartByte, End: reference.EndByte}) {
				return nil
			}
			bodyReferences++
			edits = append(edits, certification.TextEdit{
				Location: sourceLocation(reference, source),
				NewText:  parameterName + "." + item.name,
			})
		}
	}
	if bodyReferences == 0 {
		return nil
	}
	sort.Slice(edits, func(i, j int) bool { return edits[i].Location.StartByte < edits[j].Location.StartByte })
	return &certification.Fix{
		Message:       "Fix: Keep component props reactive",
		Applicability: certification.FixSafe,
		Edits:         edits,
	}
}

func safePropsFixExpression(executionMap compilerfacts.ExecutionMap, span compilerfacts.Span) bool {
	if executionRole(executionMap, span) == ExecutionTrackedJSX {
		return true
	}
	for _, operation := range executionMap.JsxOperations {
		if operation.Kind == "jsx-expression" && contains(operation.Span, span) {
			return true
		}
	}
	return false
}

func identifierOnly(source []byte) bool {
	if len(source) == 0 || !identifierStart(source[0]) {
		return false
	}
	for _, character := range source[1:] {
		if !identifierContinue(character) {
			return false
		}
	}
	return true
}

func identifierStart(character byte) bool {
	return character == '_' || character == '$' || character >= 'A' && character <= 'Z' || character >= 'a' && character <= 'z'
}

func identifierContinue(character byte) bool {
	return identifierStart(character) || character >= '0' && character <= '9'
}

func availableIdentifier(source []byte, base string) string {
	for suffix := 0; ; suffix++ {
		candidate := base
		if suffix != 0 {
			candidate = fmt.Sprintf("%s%d", base, suffix+1)
		}
		found := false
		for index := 0; index+len(candidate) <= len(source); index++ {
			if string(source[index:index+len(candidate)]) != candidate {
				continue
			}
			before := index > 0 && identifierContinue(source[index-1])
			after := index+len(candidate) < len(source) && identifierContinue(source[index+len(candidate)])
			if !before && !after {
				found = true
				break
			}
		}
		if !found {
			return candidate
		}
	}
}

func insideNestedFunction(source []byte, start, end, offset int) bool {
	if start < 0 || end > len(source) || start >= end || offset <= start || offset >= end {
		return false
	}
	for cursor := start; cursor+1 < offset; cursor++ {
		if source[cursor] != '=' || source[cursor+1] != '>' {
			continue
		}
		bodyStart := cursor + 2
		for bodyStart < end && (source[bodyStart] == ' ' || source[bodyStart] == '\t' || source[bodyStart] == '\r' || source[bodyStart] == '\n') {
			bodyStart++
		}
		if bodyStart > offset {
			continue
		}
		if source[bodyStart] == '{' {
			if close := matchingBrace(source, bodyStart, '{', '}'); close > offset {
				return true
			}
			continue
		}
		paren, bracket, brace := 0, 0, 0
		for index := bodyStart; index < end; index++ {
			switch source[index] {
			case '(':
				paren++
			case ')':
				if paren == 0 {
					if offset < index {
						return true
					}
					index = end
					continue
				}
				paren--
			case '[':
				bracket++
			case ']':
				if bracket > 0 {
					bracket--
				}
			case '{':
				brace++
			case '}':
				if brace > 0 {
					brace--
				}
			case ',', ';', '\n':
				if paren == 0 && bracket == 0 && brace == 0 {
					if offset < index {
						return true
					}
					index = end
				}
			}
		}
	}
	for _, match := range functionKeywordPattern.FindAllIndex(source[start:offset], -1) {
		open := bytes.IndexByte(source[start+match[1]:offset], '{')
		if open < 0 {
			continue
		}
		open += start + match[1]
		if close := matchingBrace(source, open, '{', '}'); close > offset {
			return true
		}
	}
	return false
}

func componentPropAliasSymbols(ctx context.Context, facts typefacts.Project, path string, source []byte, function sourceFunction, initial typefacts.SymbolID) ([]typefacts.SymbolID, error) {
	aliasPattern := aliasAssignmentPattern
	assignmentPattern := constAssignmentPattern
	proxyCalls := make([]solidSourceCall, 0)
	for _, helper := range []string{"merge", "omit"} {
		calls, err := solidSourceCalls(ctx, facts, path, source, helper)
		if err != nil {
			return nil, err
		}
		proxyCalls = append(proxyCalls, calls...)
	}
	known := map[typefacts.SymbolID]bool{canonicalSymbol(ctx, facts, initial): true}
	result := []typefacts.SymbolID{initial}
	for changed := true; changed; {
		changed = false
		for _, match := range aliasPattern.FindAllSubmatchIndex(source[function.bodyStart:function.bodyEnd], -1) {
			leftStart, leftEnd := function.bodyStart+match[2], function.bodyStart+match[3]
			rightStart, rightEnd := function.bodyStart+match[4], function.bodyStart+match[5]
			right, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: rightStart, EndByte: rightEnd})
			if err != nil || !known[canonicalSymbol(ctx, facts, right)] {
				continue
			}
			left, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: leftStart, EndByte: leftEnd})
			if err != nil {
				continue
			}
			canonical := canonicalSymbol(ctx, facts, left)
			if known[canonical] {
				continue
			}
			known[canonical] = true
			result = append(result, left)
			changed = true
		}
		for _, call := range proxyCalls {
			if call.callee.Start <= function.bodyStart || call.callee.End >= function.bodyEnd {
				continue
			}
			usesProps := false
			for _, argument := range call.arguments {
				start, end := trimByteSpan(source, argument.start, argument.end)
				if start >= end || !identifierPattern.Match(source[start:end]) {
					continue
				}
				symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: start, EndByte: end})
				if err == nil && known[canonicalSymbol(ctx, facts, symbol)] {
					usesProps = true
					break
				}
			}
			if !usesProps {
				continue
			}
			statementStart := call.callee.Start
			for statementStart > function.bodyStart+1 && source[statementStart-1] != ';' && source[statementStart-1] != '\n' {
				statementStart--
			}
			prefix := source[statementStart:call.callee.Start]
			match := assignmentPattern.FindSubmatchIndex(prefix)
			if match == nil {
				continue
			}
			leftStart, leftEnd := statementStart+match[2], statementStart+match[3]
			left, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: leftStart, EndByte: leftEnd})
			if err != nil {
				continue
			}
			canonical := canonicalSymbol(ctx, facts, left)
			if known[canonical] {
				continue
			}
			known[canonical] = true
			result = append(result, left)
			changed = true
		}
	}
	return result, nil
}

func markConditionalComponentReads(files []typefacts.SourceFile, functions map[string][]sourceFunction, program *Program) {
	sources := make(map[string][]byte, len(files))
	for _, file := range files {
		sources[filepath.Clean(file.Path)] = file.Source
	}
	ifPattern := ifStatementPattern
	for index := range program.Reads {
		read := &program.Reads[index]
		if read.Execution != ExecutionUntrackedRendering {
			continue
		}
		path := filepath.Clean(read.Location.Path)
		source := sources[path]
		function, found := functionContext(functions[path], read.Location.StartByte)
		if !found || !function.rendering {
			continue
		}
		for _, match := range ifPattern.FindAllIndex(source[function.bodyStart:function.bodyEnd], -1) {
			open := function.bodyStart + match[1] - 1
			close := matchingBrace(source, open, '(', ')')
			if close <= open || read.Location.StartByte <= open || read.Location.EndByte >= close {
				continue
			}
			cursor := close
			for cursor < function.bodyEnd && (source[cursor] == ' ' || source[cursor] == '\t' || source[cursor] == '\r' || source[cursor] == '\n') {
				cursor++
			}
			if bytes.HasPrefix(source[cursor:function.bodyEnd], []byte("return")) {
				read.Context = function.name + " conditional return"
				break
			}
			if cursor < function.bodyEnd && source[cursor] == '{' {
				cursor++
				for cursor < function.bodyEnd && (source[cursor] == ' ' || source[cursor] == '\t' || source[cursor] == '\r' || source[cursor] == '\n') {
					cursor++
				}
				if bytes.HasPrefix(source[cursor:function.bodyEnd], []byte("return")) {
					read.Context = function.name + " conditional return"
					break
				}
			}
		}
		if read.Context == function.name+" conditional return" {
			continue
		}
		statementStart := read.Location.StartByte
		for statementStart > function.bodyStart+1 && source[statementStart-1] != ';' && source[statementStart-1] != '\n' && source[statementStart-1] != '{' {
			statementStart--
		}
		statementFinish := statementEnd(source, read.Location.EndByte)
		if statementFinish <= read.Location.EndByte || statementFinish > function.bodyEnd {
			continue
		}
		prefix := source[statementStart:read.Location.StartByte]
		returnOffset := bytes.LastIndex(prefix, []byte("return"))
		if returnOffset < 0 {
			continue
		}
		decisionStart := statementStart + returnOffset + len("return")
		decision := source[decisionStart:statementFinish]
		operator := bytes.IndexByte(decision, '?')
		if logical := bytes.Index(decision, []byte("&&")); operator < 0 || logical >= 0 && logical < operator {
			operator = logical
		}
		if operator >= 0 && read.Location.EndByte <= decisionStart+operator {
			read.Context = function.name + " conditional return"
		}
	}
}

func addReactiveReadsAfterAwait(
	ctx context.Context,
	facts typefacts.Project,
	files []typefacts.SourceFile,
	sources map[string][]byte,
	program *Program,
) error {
	describer, ok := facts.(typefacts.TypeDescriber)
	if !ok {
		return nil
	}
	factories := []string{"createMemo", "createEffect", "createRenderEffect", "createProjection", "createSignal", "createStore", "createOptimistic", "createOptimisticStore"}
	for _, file := range files {
		path := filepath.Clean(file.Path)
		var asyncFunctions []typefacts.AsyncFunctionFact
		if discoverer, available := facts.(typefacts.AsyncFunctionDiscoverer); available {
			var err error
			asyncFunctions, err = projectAsyncFunctionFacts(ctx, facts, discoverer)
			if err != nil {
				return err
			}
		}
		var parsedCalls []typefacts.SourceCall
		if discoverer, available := facts.(typefacts.CallDiscoverer); available {
			var err error
			parsedCalls, err = sourceCallFacts(ctx, discoverer, path)
			if err != nil {
				return err
			}
		}
		for _, factory := range factories {
			calls, err := solidSourceCalls(ctx, facts, path, file.Source, factory)
			if err != nil {
				return err
			}
			for _, call := range calls {
				if len(call.arguments) == 0 {
					continue
				}
				callback := call.arguments[0]
				bodyStart, bodyEnd, lexical := asyncArrowBody(file.Source, callback)
				semantic := asyncFunctionInRange(ctx, asyncFunctions, path, callback.start, callback.end)
				if semantic == nil {
					start, end := trimByteSpan(file.Source, callback.start, callback.end)
					if identifierPattern.Match(file.Source[start:end]) {
						if symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: start, EndByte: end}); err == nil {
							symbol = canonicalSymbol(ctx, facts, symbol)
							semantic = asyncFunctionBySymbol(ctx, asyncFunctions, symbol)
						}
					}
				}
				if semantic != nil {
					if !semantic.CanReturnAsync {
						continue
					}
					bodyStart, bodyEnd = semantic.Expression.StartByte, semantic.Expression.EndByte
				} else if !lexical {
					continue
				}
				analysisPath, analysisSource, analysisCalls := path, file.Source, parsedCalls
				if semantic != nil && semantic.Expression.Path != path {
					analysisPath = semantic.Expression.Path
					analysisSource = sources[analysisPath]
					if discoverer, available := facts.(typefacts.CallDiscoverer); available {
						analysisCalls, err = sourceCallFacts(ctx, discoverer, analysisPath)
						if err != nil {
							return err
						}
					}
				}
				type candidate struct {
					start, end, callEnd int
					target              typefacts.SymbolID
				}
				candidates := make([]candidate, 0)
				if analysisCalls != nil {
					for _, parsed := range analysisCalls {
						if parsed.Callee.StartByte >= bodyStart && parsed.Location.EndByte <= bodyEnd {
							candidates = append(candidates, candidate{start: parsed.Callee.StartByte, end: parsed.Callee.EndByte, callEnd: parsed.Location.EndByte - 1, target: parsed.Target})
						}
					}
				} else {
					for _, match := range callCandidatePattern.FindAllSubmatchIndex(analysisSource[bodyStart:bodyEnd], -1) {
						open := bodyStart + match[1] - 1
						close := matchingBrace(analysisSource, open, '(', ')')
						if close > open && close <= bodyEnd {
							candidates = append(candidates, candidate{start: bodyStart + match[2], end: bodyStart + match[3], callEnd: close})
						}
					}
				}
				for _, candidate := range candidates {
					start, end := candidate.start, candidate.end
					if semantic != nil {
						dominated := false
						for _, call := range semantic.CallsAfterAwait {
							if call.StartByte == start {
								dominated = true
								break
							}
						}
						if !dominated {
							continue
						}
					} else {
						if insideNestedFunction(analysisSource, bodyStart, bodyEnd, start) || !guaranteedAwaitBefore(analysisSource, bodyStart, start) {
							continue
						}
					}
					typeStart := start
					if dot := bytes.LastIndexByte(analysisSource[start:end], '.'); dot >= 0 {
						typeStart = start + dot + 1
					}
					descriptor, describeErr := describeTarget(ctx, describer, candidate.target, typefacts.Location{Path: analysisPath, StartByte: typeStart, EndByte: end})
					provenByType := describeErr == nil && solidAccessorDescriptor(descriptor, sources)
					if !provenByType && !reactiveReadStartsAt(*program, analysisPath, start) && !solidSignalAccessorAt(ctx, facts, analysisPath, analysisSource, start, end) {
						continue
					}
					accessor := strings.Join(strings.Fields(string(analysisSource[start:end])), "")
					program.StaticViolations = append(program.StaticViolations, StaticViolation{
						ID: "SC1002", Rule: "reactive-read-after-await",
						Message:         fmt.Sprintf("reactive accessor %q is read after await, when dependency tracking has ended; read it before await or move it into another tracked computation", accessor),
						AnalysisContext: factory + " async computation",
						Location:        sourceLocation(typefacts.Location{Path: analysisPath, StartByte: start, EndByte: candidate.callEnd}, analysisSource),
					})
				}
			}
		}
	}
	return nil
}

func solidSignalAccessorAt(ctx context.Context, facts typefacts.Project, path string, source []byte, start, end int) bool {
	candidate, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: start, EndByte: end})
	if err != nil {
		return false
	}
	declarations, _ := facts.Declarations(ctx, candidate)
	candidate = canonicalSymbol(ctx, facts, candidate)
	for _, match := range signalBindingPattern.FindAllSubmatchIndex(source, -1) {
		isSolid, resolveErr := isSolidCalleeAt(ctx, facts, path, match[4], match[5], "createSignal")
		if resolveErr != nil || !isSolid {
			continue
		}
		getter, symbolErr := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: match[2], EndByte: match[3]})
		if symbolErr == nil && canonicalSymbol(ctx, facts, getter) == candidate {
			return true
		}
		for _, declaration := range declarations {
			if filepath.Clean(declaration.Location.Path) == filepath.Clean(path) && declaration.Location.StartByte == match[2] {
				return true
			}
		}
	}
	return false
}

func reactiveReadStartsAt(program Program, path string, start int) bool {
	matches := func(read ReactiveRead) bool {
		return filepath.Clean(read.Location.Path) == filepath.Clean(path) && read.Location.StartByte == start && read.Kind == ReactiveAccessor
	}
	for _, read := range program.Reads {
		if matches(read) {
			return true
		}
	}
	for _, function := range program.Functions {
		for _, read := range function.Reads {
			if matches(read) {
				return true
			}
		}
		for _, read := range function.ReturnedReads {
			if matches(read) {
				return true
			}
		}
	}
	return false
}

func asyncArrowBody(source []byte, callback byteSpan) (int, int, bool) {
	if callback.start < 0 || callback.end > len(source) || callback.start >= callback.end {
		return 0, 0, false
	}
	text := source[callback.start:callback.end]
	trimmed := strings.TrimSpace(string(text))
	if !strings.HasPrefix(trimmed, "async ") && !strings.HasPrefix(trimmed, "async(") {
		return 0, 0, false
	}
	arrow := bytes.Index(text, []byte("=>"))
	if arrow < 0 {
		return 0, 0, false
	}
	open := callback.start + arrow + 2
	for open < callback.end && (source[open] == ' ' || source[open] == '\t' || source[open] == '\r' || source[open] == '\n') {
		open++
	}
	if open >= callback.end || source[open] != '{' {
		return 0, 0, false
	}
	close := matchingBrace(source, open, '{', '}')
	if close <= open || close > callback.end {
		return 0, 0, false
	}
	return open + 1, close - 1, true
}

func guaranteedAwaitBefore(source []byte, bodyStart, readStart int) bool {
	if bodyStart < 0 || readStart > len(source) || bodyStart >= readStart {
		return false
	}
	prefix := source[bodyStart:readStart]
	statementStart := 0
	paren, bracket, brace := 0, 0, 0
	quote := byte(0)
	escaped, lineComment, blockComment := false, false, false
	for index := 0; index < len(prefix); index++ {
		character := prefix[index]
		if lineComment {
			if character == '\n' {
				lineComment = false
			}
			continue
		}
		if blockComment {
			if character == '*' && index+1 < len(prefix) && prefix[index+1] == '/' {
				blockComment = false
				index++
			}
			continue
		}
		if quote != 0 {
			if escaped {
				escaped = false
			} else if character == '\\' {
				escaped = true
			} else if character == quote {
				quote = 0
			}
			continue
		}
		if character == '/' && index+1 < len(prefix) {
			if prefix[index+1] == '/' {
				lineComment = true
				index++
				continue
			}
			if prefix[index+1] == '*' {
				blockComment = true
				index++
				continue
			}
		}
		if character == '\'' || character == '"' || character == '`' {
			quote = character
			continue
		}
		switch character {
		case '(':
			paren++
		case ')':
			paren--
		case '[':
			bracket++
		case ']':
			bracket--
		case '{':
			brace++
		case '}':
			brace--
		case ';':
			if paren == 0 && bracket == 0 && brace == 0 {
				statement := strings.TrimSpace(string(prefix[statementStart:index]))
				statementStart = index + 1
				if guaranteedAwaitStatement(statement) {
					return true
				}
			}
		}
	}
	return false
}

func guaranteedAwaitStatement(statement string) bool {
	if !awaitKeywordPattern.MatchString(statement) {
		return false
	}
	if controlStatementPattern.MatchString(statement) {
		return false
	}
	return !strings.Contains(statement, "&&") && !strings.Contains(statement, "||") && !strings.Contains(statement, "?")
}

func duplicateReactiveRead(program Program, candidate ReactiveRead) bool {
	matches := func(read ReactiveRead) bool {
		return filepath.Clean(read.Location.Path) == filepath.Clean(candidate.Location.Path) && read.Location.StartByte == candidate.Location.StartByte && read.Location.EndByte == candidate.Location.EndByte
	}
	for _, read := range program.Reads {
		if matches(read) {
			return true
		}
	}
	for _, function := range program.Functions {
		for _, read := range function.Reads {
			if matches(read) {
				return true
			}
		}
	}
	return false
}

func addAsyncReads(
	ctx context.Context,
	facts typefacts.Project,
	files []typefacts.SourceFile,
	sources map[string][]byte,
	functions map[string][]sourceFunction,
	executionMaps map[string]compilerfacts.ExecutionMap,
	program *Program,
) error {
	leafRegions := make(map[string]map[string][]compilerfacts.Span)
	loadingByPath := make(map[string][]compilerfacts.Span)
	computationByPath := make(map[string][]allowedWriteRegion)
	for _, file := range files {
		path := filepath.Clean(file.Path)
		leafRegions[path] = make(map[string][]compilerfacts.Span)
		loadingByPath[path] = solidJSXOperationSpans(ctx, facts, path, file.Source, executionMaps[path], "Loading")
		computationRegions, computationErr := readExecutionRegions(ctx, facts, path, file.Source)
		if computationErr != nil {
			return computationErr
		}
		computationByPath[path] = computationRegions
		for _, owner := range []string{"onSettled", "createTrackedEffect"} {
			regions, err := solidCallArgumentRegions(ctx, facts, path, file.Source, owner, 0)
			if err != nil {
				return err
			}
			leafRegions[path][owner] = regions
		}
	}
	for _, file := range files {
		path := filepath.Clean(file.Path)
		type asyncBinding struct {
			name      typefacts.Location
			primitive string
			kind      ReactiveValueKind
			arguments []typefacts.Location
		}
		bindings := make([]asyncBinding, 0)
		if discoverer, ok := facts.(typefacts.BindingDiscoverer); ok {
			discovered, err := sourceBindingFacts(ctx, discoverer, path)
			if err != nil {
				return err
			}
			for _, binding := range discovered {
				primitive := solidPrimitiveName(ctx, facts, binding.Initializer.Target)
				kind, array := ReactiveAccessor, false
				switch primitive {
				case "createSignal", "createOptimistic":
					array = true
				case "createStore", "createOptimisticStore":
					kind, array = ReactiveStorePath, true
				case "createMemo":
				case "createProjection":
					kind = ReactiveStorePath
				default:
					continue
				}
				if binding.Array != array || len(binding.Names) == 0 || binding.Names[0].Path == "" {
					continue
				}
				bindings = append(bindings, asyncBinding{name: binding.Names[0], primitive: primitive, kind: kind, arguments: binding.Initializer.Arguments})
			}
		} else {
			fallbacks := []struct {
				pattern   *regexp.Regexp
				primitive string
				kind      ReactiveValueKind
			}{
				{asyncDirectPattern, "createMemo", ReactiveAccessor},
				{signalBindingPattern, "createSignal", ReactiveAccessor},
				{signalBindingPattern, "createOptimistic", ReactiveAccessor},
				{storeBindingPattern, "createStore", ReactiveStorePath},
				{storeBindingPattern, "createOptimisticStore", ReactiveStorePath},
				{asyncDirectPattern, "createProjection", ReactiveStorePath},
			}
			for _, fallback := range fallbacks {
				for _, match := range fallback.pattern.FindAllSubmatchIndex(file.Source, -1) {
					call, err := facts.ResolvedCall(ctx, typefacts.Location{Path: path, StartByte: match[4], EndByte: match[5]})
					if err != nil || !isSolidPrimitive(ctx, facts, call.Target, fallback.primitive) {
						continue
					}
					open := match[1] - 1
					close := matchingBrace(file.Source, open, '(', ')')
					if close > open {
						bindings = append(bindings, asyncBinding{
							name: typefacts.Location{Path: path, StartByte: match[2], EndByte: match[3]}, primitive: fallback.primitive, kind: fallback.kind,
							arguments: []typefacts.Location{{Path: path, StartByte: open + 1, EndByte: close}},
						})
					}
				}
			}
		}
		for _, binding := range bindings {
			if len(binding.arguments) == 0 || !computationIsAsync(ctx, facts, path, file.Source, binding.arguments[0].StartByte, binding.arguments[0].EndByte) {
				continue
			}
			symbol, err := facts.SymbolAt(ctx, binding.name)
			if err != nil {
				return fmt.Errorf("resolve async %s result in %s: %w", binding.primitive, path, err)
			}
			declarations, err := facts.Declarations(ctx, symbol)
			if err != nil || len(declarations) == 0 {
				return fmt.Errorf("resolve async %s declaration in %s", binding.primitive, path)
			}
			declaration := sourceLocation(declarations[0].Location, sources[filepath.Clean(declarations[0].Location.Path)])
			references, err := facts.References(ctx, symbol)
			if err != nil {
				return fmt.Errorf("find async %s references in %s: %w", binding.primitive, path, err)
			}
			for _, reference := range references {
				reference.Path = filepath.Clean(reference.Path)
				source, ok := sources[reference.Path]
				if !ok {
					continue
				}
				end, isRead := accessorCallEnd(source, reference.EndByte)
				if binding.kind == ReactiveStorePath {
					end, isRead = propertyAccessEnd(source, reference.EndByte)
				}
				if !isRead {
					continue
				}
				span := compilerfacts.Span{Start: reference.StartByte, End: end}
				read := AsyncRead{
					Kind:        binding.kind,
					Accessor:    string(source[reference.StartByte:end]),
					Location:    sourceLocation(typefacts.Location{Path: reference.Path, StartByte: reference.StartByte, EndByte: end}, source),
					Declaration: declaration, Execution: executionRoleWithComputations(executionMaps[reference.Path], computationByPath[reference.Path], span),
				}
				if function, ok := functionContext(functions[reference.Path], reference.StartByte); ok {
					read.Function = function.id
				}
				for owner, regions := range leafRegions[reference.Path] {
					for _, region := range regions {
						if contains(region, span) {
							read.LeafOwner = owner
						}
					}
				}
				for _, operation := range loadingByPath[reference.Path] {
					if contains(operation, span) {
						read.UnderLoading = true
					}
				}
				program.AsyncReads = append(program.AsyncReads, read)
			}
		}
	}
	applyLoadingComponentDominance(ctx, facts, files, functions, executionMaps, program)
	return nil
}

type loadingFunctionInfo struct{ exported bool }

func applyLoadingComponentDominance(ctx context.Context, facts typefacts.Project, files []typefacts.SourceFile, functions map[string][]sourceFunction, executionMaps map[string]compilerfacts.ExecutionMap, program *Program) {
	infos := make(map[FunctionID]loadingFunctionInfo)
	loadingProviders := make(map[FunctionID]bool)
	for _, declared := range functions {
		for _, function := range declared {
			infos[function.id] = loadingFunctionInfo{exported: function.exported}
		}
	}
	// A component is a Loading provider only when every direct props.children
	// read in its body is dominated by a real Solid <Loading>. This lets a
	// transparent wrapper carry the boundary proof without trusting its name.
	for _, file := range files {
		path := filepath.Clean(file.Path)
		loading := solidJSXOperationSpans(ctx, facts, path, file.Source, executionMaps[path], "Loading")
		for _, function := range functions[path] {
			if !function.rendering || len(function.parameters) == 0 || function.parameters[0] == "" {
				continue
			}
			propSymbols, err := componentPropAliasSymbols(ctx, facts, path, file.Source, function, function.parameters[0])
			if err != nil {
				continue
			}
			children, allUnderLoading := 0, true
			for _, propSymbol := range propSymbols {
				references, err := facts.References(ctx, propSymbol)
				if err != nil {
					allUnderLoading = false
					break
				}
				for _, reference := range references {
					if filepath.Clean(reference.Path) != path || reference.StartByte <= function.bodyStart || reference.StartByte >= function.bodyEnd {
						continue
					}
					end, property := propertyAccessEnd(file.Source, reference.EndByte)
					if !property || string(file.Source[reference.EndByte:end]) != ".children" {
						continue
					}
					children++
					span := compilerfacts.Span{Start: reference.StartByte, End: end}
					under := false
					for _, boundary := range loading {
						if contains(boundary, span) {
							under = true
							break
						}
					}
					allUnderLoading = allUnderLoading && under
				}
			}
			loadingProviders[function.id] = children > 0 && allUnderLoading
		}
	}
	sites := make(map[FunctionID][]bool)
	boundariesByPath := make(map[string][]compilerfacts.Span)
	for _, file := range files {
		path := filepath.Clean(file.Path)
		boundaries := append([]compilerfacts.Span(nil), solidJSXOperationSpans(ctx, facts, path, file.Source, executionMaps[path], "Loading")...)
		for _, operation := range executionMaps[path].JsxOperations {
			if operation.Kind != "component-invocation" || operation.Span.Start+1 >= len(file.Source) {
				continue
			}
			if id, ok := jsxComponentFunctionID(ctx, facts, path, file.Source, operation.Span, infos); ok && loadingProviders[id] {
				boundaries = append(boundaries, operation.Span)
			}
		}
		boundariesByPath[path] = boundaries
		for _, operation := range executionMaps[path].JsxOperations {
			if operation.Kind != "component-invocation" || jsxSolidPrimitive(ctx, facts, path, file.Source, operation.Span, "Loading") != "" || operation.Span.Start+1 >= len(file.Source) {
				continue
			}
			id, known := jsxComponentFunctionID(ctx, facts, path, file.Source, operation.Span, infos)
			if !known {
				continue
			}
			under := false
			for _, boundary := range boundaries {
				if contains(boundary, operation.Span) {
					under = true
					break
				}
			}
			sites[id] = append(sites[id], under)
		}
	}
	for index := range program.AsyncReads {
		read := &program.AsyncReads[index]
		for _, boundary := range boundariesByPath[filepath.Clean(read.Location.Path)] {
			if contains(boundary, compilerfacts.Span{Start: read.Location.StartByte, End: read.Location.EndByte}) {
				read.UnderLoading = true
				break
			}
		}
		if read.UnderLoading || read.Function == "" || infos[read.Function].exported {
			continue
		}
		functionSites := sites[read.Function]
		if len(functionSites) == 0 {
			continue
		}
		all := true
		for _, under := range functionSites {
			all = all && under
		}
		read.UnderLoading = all
	}
}

func jsxComponentFunctionID(ctx context.Context, facts typefacts.Project, path string, source []byte, span compilerfacts.Span, known map[FunctionID]loadingFunctionInfo) (FunctionID, bool) {
	start, end := span.Start+1, span.Start+1
	for end < len(source) && identifierContinue(source[end]) {
		end++
	}
	if end == start {
		return "", false
	}
	symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: start, EndByte: end})
	if err != nil {
		return "", false
	}
	id := FunctionID(canonicalSymbol(ctx, facts, symbol))
	_, ok := known[id]
	return id, ok
}

func computationIsAsync(ctx context.Context, facts typefacts.Project, path string, source []byte, start, end int) bool {
	arguments := splitArguments(source, start, end)
	if len(arguments) == 0 {
		return false
	}
	argument := arguments[0]
	if discoverer, ok := facts.(typefacts.AsyncFunctionDiscoverer); ok {
		asyncFunctions, err := projectAsyncFunctionFacts(ctx, facts, discoverer)
		if err == nil {
			if function := asyncFunctionInRange(ctx, asyncFunctions, path, argument.start, argument.end); function != nil {
				return function.CanReturnAsync
			}
			trimmedStart, trimmedEnd := trimByteSpan(source, argument.start, argument.end)
			if identifierPattern.Match(source[trimmedStart:trimmedEnd]) {
				if symbol, symbolErr := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: trimmedStart, EndByte: trimmedEnd}); symbolErr == nil {
					symbol = canonicalSymbol(ctx, facts, symbol)
					if function := asyncFunctionBySymbol(ctx, asyncFunctions, symbol); function != nil {
						return function.CanReturnAsync
					}
				}
			}
		}
	}
	trimmed := strings.TrimSpace(string(source[argument.start:argument.end]))
	if strings.HasPrefix(trimmed, "async ") || strings.HasPrefix(trimmed, "async(") {
		return true
	}
	callPattern := identifierCallPattern
	for _, match := range callPattern.FindAllIndex(source[argument.start:argument.end], -1) {
		callStart := argument.start + match[0]
		nameEnd := callStart
		for nameEnd < len(source) && ((source[nameEnd] >= 'A' && source[nameEnd] <= 'Z') || (source[nameEnd] >= 'a' && source[nameEnd] <= 'z') || (source[nameEnd] >= '0' && source[nameEnd] <= '9') || source[nameEnd] == '_' || source[nameEnd] == '$') {
			nameEnd++
		}
		call, err := facts.ResolvedCall(ctx, typefacts.Location{Path: path, StartByte: callStart, EndByte: nameEnd})
		if err != nil {
			continue
		}
		if strings.Contains(call.ReturnTypeText, "Promise<") || strings.Contains(call.ReturnTypeText, "AsyncIterable<") || strings.Contains(call.ReturnTypeText, "PromiseLike<") {
			return true
		}
	}
	return false
}

func addMissingOwners(
	ctx context.Context,
	facts typefacts.Project,
	files []typefacts.SourceFile,
	functions map[string][]sourceFunction,
	executionMaps map[string]compilerfacts.ExecutionMap,
	program *Program,
) error {
	for _, file := range files {
		path := filepath.Clean(file.Path)
		rootRegions, err := ownerProvidingRegions(ctx, facts, path, file.Source)
		if err != nil {
			return err
		}
		rootOwns := func(offset int) bool {
			point := compilerfacts.Span{Start: offset, End: offset}
			for _, region := range rootRegions {
				if contains(region, point) {
					return true
				}
			}
			return false
		}
		imperativeRegions, err := allowedWriteRegions(ctx, facts, path, file.Source)
		if err != nil {
			return err
		}
		for _, operation := range []struct{ primitive, kind string }{
			{primitive: "createEffect", kind: "effect"},
			{primitive: "createTrackedEffect", kind: "effect"},
			{primitive: "onCleanup", kind: "cleanup"},
		} {
			calls, err := solidCallSpans(ctx, facts, path, file.Source, operation.primitive)
			if err != nil {
				return err
			}
			for _, call := range calls {
				if rootOwns(call.Start) {
					continue
				}
				function, inFunction := functionContext(functions[path], call.Start)
				functionID := FunctionID("")
				if inFunction {
					functionID = function.id
				}
				program.MissingOwners = append(program.MissingOwners, OwnerRequirement{
					Operation: operation.kind,
					Location:  sourceLocation(typefacts.Location{Path: path, StartByte: call.Start, EndByte: call.End}, file.Source),
					Function:  functionID,
					Unowned:   unownedExecution(executionMaps[path], imperativeRegions, call),
				})
			}
		}
		for _, operation := range executionMaps[path].JsxOperations {
			if operation.Kind != "component-invocation" || operation.Span.Start >= len(file.Source) {
				continue
			}
			name := jsxSolidPrimitive(ctx, facts, path, file.Source, operation.Span, "Loading", "Errored")
			if name == "" {
				continue
			}
			if rootOwns(operation.Span.Start) {
				continue
			}
			function, inFunction := functionContext(functions[path], operation.Span.Start)
			functionID := FunctionID("")
			if inFunction {
				functionID = function.id
			}
			program.MissingOwners = append(program.MissingOwners, OwnerRequirement{
				Operation: "boundary",
				Location:  sourceLocation(typefacts.Location{Path: path, StartByte: operation.Span.Start, EndByte: jsxTagEnd(file.Source, operation.Span)}, file.Source),
				Function:  functionID,
				Unowned:   unownedExecution(executionMaps[path], imperativeRegions, operation.Span),
			})
		}
	}
	return nil
}

func unownedExecution(executionMap compilerfacts.ExecutionMap, regions []allowedWriteRegion, span compilerfacts.Span) bool {
	for _, region := range regions {
		if contains(region.span, span) {
			return true
		}
	}
	for _, callback := range executionMap.CallbackRoles {
		if (callback.Role == compilerfacts.CallbackEventHandler || callback.Role == compilerfacts.CallbackDirectiveApply) && contains(callback.Span, span) {
			return true
		}
	}
	return false
}

func ownerProvidingRegions(ctx context.Context, facts typefacts.Project, path string, source []byte) ([]compilerfacts.Span, error) {
	roles := []struct {
		name     string
		argument int
	}{
		{"createRoot", 0}, {"runWithOwner", 1}, {"createMemo", 0},
		{"createEffect", 0}, {"createRenderEffect", 0}, {"createProjection", 0},
		{"createSignal", 0}, {"createStore", 0},
	}
	regions := make([]compilerfacts.Span, 0)
	for _, role := range roles {
		spans, err := solidCallArgumentRegions(ctx, facts, path, source, role.name, role.argument)
		if err != nil {
			return nil, err
		}
		for _, span := range spans {
			if functionLikeArgument(source, span) {
				regions = append(regions, span)
			}
		}
	}
	return regions, nil
}

func functionLikeArgument(source []byte, span compilerfacts.Span) bool {
	if span.Start < 0 || span.End > len(source) || span.Start >= span.End {
		return false
	}
	value := strings.TrimSpace(string(source[span.Start:span.End]))
	return identifierPattern.MatchString(value) || strings.HasPrefix(value, "function") ||
		strings.HasPrefix(value, "async function") || func() bool {
		arrow := topLevelArrow(source, span.Start)
		return arrow >= span.Start && arrow < span.End
	}()
}

func addLeafOwnerOperations(ctx context.Context, facts typefacts.Project, files []typefacts.SourceFile, program *Program) error {
	leafOwners := []string{"onSettled", "createTrackedEffect"}
	for _, file := range files {
		path := filepath.Clean(file.Path)
		for _, owner := range leafOwners {
			ownerRegions, err := solidCallArgumentRegions(ctx, facts, path, file.Source, owner, 0)
			if err != nil {
				return err
			}
			for _, ownerRegion := range ownerRegions {
				for _, primitive := range []struct {
					name        string
					derivedOnly bool
				}{
					{name: "onCleanup"}, {name: "flush"},
					{name: "createMemo"}, {name: "createEffect"}, {name: "createRenderEffect"}, {name: "createTrackedEffect"},
					{name: "createProjection"}, {name: "createRoot"}, {name: "createOwner"}, {name: "mapArray"}, {name: "children"},
					{name: "createSignal", derivedOnly: true}, {name: "createStore", derivedOnly: true},
					{name: "createOptimistic", derivedOnly: true}, {name: "createOptimisticStore", derivedOnly: true},
				} {
					calls, err := solidSourceCalls(ctx, facts, path, file.Source, primitive.name)
					if err != nil {
						return err
					}
					for _, call := range calls {
						span := call.callee
						if !contains(ownerRegion, span) || primitive.derivedOnly && (len(call.arguments) == 0 || !functionLikeArgument(file.Source, compilerfacts.Span{Start: call.arguments[0].start, End: call.arguments[0].end})) {
							continue
						}
						operation := LeafOwnerOperation{
							Primitive: primitive.name, Owner: owner,
							Location: sourceLocation(typefacts.Location{Path: path, StartByte: span.Start, EndByte: span.End}, file.Source),
						}
						if primitive.name == "onCleanup" {
							operation.Fix = terminalCleanupFix(path, file.Source, ownerRegion, span)
						}
						program.LeafOperations = append(program.LeafOperations, operation)
					}
				}
			}
		}
	}
	return nil
}

func addPrimitiveCreations(ctx context.Context, facts typefacts.Project, files []typefacts.SourceFile, functions map[string][]sourceFunction, executionMaps map[string]compilerfacts.ExecutionMap, program *Program) error {
	primitives := []string{"createSignal", "createStore", "createMemo", "createEffect", "createRenderEffect", "createTrackedEffect", "createProjection", "createOptimistic", "createOptimisticStore"}
	for _, file := range files {
		path := filepath.Clean(file.Path)
		for _, primitive := range primitives {
			calls, err := solidCallSpans(ctx, facts, path, file.Source, primitive)
			if err != nil {
				return err
			}
			for _, call := range calls {
				creation := PrimitiveCreation{
					Primitive: primitive,
					Location:  sourceLocation(typefacts.Location{Path: path, StartByte: call.Start, EndByte: call.End}, file.Source),
				}
				for _, callback := range executionMaps[path].CallbackRoles {
					if callback.Role == compilerfacts.CallbackDirectiveApply && contains(callback.Span, call) {
						program.DirectiveCreations = append(program.DirectiveCreations, creation)
					}
				}
				function, ok := functionContext(functions[path], call.Start)
				if !ok {
					continue
				}
				program.Functions[function.programIndex].PrimitiveCreations = append(
					program.Functions[function.programIndex].PrimitiveCreations,
					func() PrimitiveCreation {
						creation.InReturnedClosure = containsOffset(function.returnedClosures, call.Start)
						return creation
					}(),
				)
			}
		}
	}
	return nil
}

type solidSourceCall struct {
	callee    compilerfacts.Span
	arguments []byteSpan
}

type solidCallCacheKey struct {
	path string
	name string
}

type solidCallCache struct {
	calls         map[solidCallCacheKey][]solidSourceCall
	sourceCalls   map[string]map[string][]solidSourceCall
	sourceErrs    map[string]error
	sourceFacts   map[string][]typefacts.SourceCall
	factErrs      map[string]error
	bindings      map[string][]typefacts.SourceBinding
	bindingErrs   map[string]error
	ownedWrites   map[string][]allowedWriteRegion
	ownedErrs     map[string]error
	descriptors   map[typefacts.SymbolID]typefacts.TypeDescriptor
	describeErrs  map[typefacts.SymbolID]error
	asyncFacts    map[string][]typefacts.AsyncFunctionFact
	asyncErrs     map[string]error
	projectAsync  []typefacts.AsyncFunctionFact
	asyncByPath   map[string][]int
	asyncBySymbol map[typefacts.SymbolID]int
	projectErr    error
	projectSet    bool
}

type solidCallCacheContextKey struct{}

func withSolidCallCache(ctx context.Context) context.Context {
	return context.WithValue(ctx, solidCallCacheContextKey{}, &solidCallCache{
		calls:         make(map[solidCallCacheKey][]solidSourceCall),
		sourceCalls:   make(map[string]map[string][]solidSourceCall),
		sourceErrs:    make(map[string]error),
		sourceFacts:   make(map[string][]typefacts.SourceCall),
		factErrs:      make(map[string]error),
		bindings:      make(map[string][]typefacts.SourceBinding),
		bindingErrs:   make(map[string]error),
		ownedWrites:   make(map[string][]allowedWriteRegion),
		ownedErrs:     make(map[string]error),
		descriptors:   make(map[typefacts.SymbolID]typefacts.TypeDescriptor),
		describeErrs:  make(map[typefacts.SymbolID]error),
		asyncFacts:    make(map[string][]typefacts.AsyncFunctionFact),
		asyncErrs:     make(map[string]error),
		asyncByPath:   make(map[string][]int),
		asyncBySymbol: make(map[typefacts.SymbolID]int),
	})
}

func solidCallCacheFrom(ctx context.Context) *solidCallCache {
	cache, _ := ctx.Value(solidCallCacheContextKey{}).(*solidCallCache)
	return cache
}

func solidSourceCalls(ctx context.Context, facts typefacts.Project, path string, source []byte, name string) ([]solidSourceCall, error) {
	if discoverer, ok := facts.(typefacts.CallDiscoverer); ok {
		cache := solidCallCacheFrom(ctx)
		if cache != nil {
			if calls, found := cache.sourceCalls[path]; found {
				return calls[name], cache.sourceErrs[path]
			}
		}
		callsByName, err := indexedSolidSourceCalls(ctx, facts, discoverer, path)
		if cache != nil {
			cache.sourceCalls[path] = callsByName
			cache.sourceErrs[path] = err
		}
		return callsByName[name], err
	}
	key := solidCallCacheKey{path: path, name: name}
	cache := solidCallCacheFrom(ctx)
	if cache != nil {
		if calls, ok := cache.calls[key]; ok {
			return calls, nil
		}
	}
	calls := make([]solidSourceCall, 0)
	for _, match := range callCandidatePattern.FindAllSubmatchIndex(source, -1) {
		calleeStart, calleeEnd := match[2], match[3]
		isSolid, err := isSolidCalleeAt(ctx, facts, path, calleeStart, calleeEnd, name)
		if err != nil {
			return nil, err
		}
		if !isSolid {
			continue
		}
		open := match[1] - 1
		close := matchingBrace(source, open, '(', ')')
		if close <= open {
			continue
		}
		calls = append(calls, solidSourceCall{
			callee:    compilerfacts.Span{Start: calleeStart, End: calleeEnd},
			arguments: splitArguments(source, open+1, close-1),
		})
	}
	if cache != nil {
		cache.calls[key] = calls
	}
	return calls, nil
}

func indexedSolidSourceCalls(ctx context.Context, facts typefacts.Project, discoverer typefacts.CallDiscoverer, path string) (map[string][]solidSourceCall, error) {
	indexed := make(map[string][]solidSourceCall)
	calls, err := sourceCallFacts(ctx, discoverer, path)
	if err != nil {
		return indexed, err
	}
	for _, call := range calls {
		name := solidPrimitiveName(ctx, facts, call.Target)
		if name == "" {
			continue
		}
		arguments := make([]byteSpan, len(call.Arguments))
		for index, argument := range call.Arguments {
			arguments[index] = byteSpan{start: argument.StartByte, end: argument.EndByte}
		}
		indexed[name] = append(indexed[name], solidSourceCall{
			callee: compilerfacts.Span{Start: call.Callee.StartByte, End: call.Callee.EndByte}, arguments: arguments,
		})
	}
	return indexed, nil
}

func sourceCallFacts(ctx context.Context, discoverer typefacts.CallDiscoverer, path string) ([]typefacts.SourceCall, error) {
	cache := solidCallCacheFrom(ctx)
	if cache != nil {
		if calls, ok := cache.sourceFacts[path]; ok {
			return calls, cache.factErrs[path]
		}
	}
	calls, err := discoverer.SourceCalls(ctx, path)
	if cache != nil {
		cache.sourceFacts[path] = calls
		cache.factErrs[path] = err
	}
	return calls, err
}

func sourceBindingFacts(ctx context.Context, discoverer typefacts.BindingDiscoverer, path string) ([]typefacts.SourceBinding, error) {
	cache := solidCallCacheFrom(ctx)
	if cache != nil {
		if bindings, ok := cache.bindings[path]; ok {
			return bindings, cache.bindingErrs[path]
		}
	}
	bindings, err := discoverer.SourceBindings(ctx, path)
	if cache != nil {
		cache.bindings[path] = bindings
		cache.bindingErrs[path] = err
	}
	return bindings, err
}

func sourceAsyncFunctionFacts(ctx context.Context, discoverer typefacts.AsyncFunctionDiscoverer, path string) ([]typefacts.AsyncFunctionFact, error) {
	cache := solidCallCacheFrom(ctx)
	if cache != nil {
		if facts, ok := cache.asyncFacts[path]; ok {
			return facts, cache.asyncErrs[path]
		}
	}
	facts, err := discoverer.SourceAsyncFunctions(ctx, path)
	if cache != nil {
		cache.asyncFacts[path] = facts
		cache.asyncErrs[path] = err
	}
	return facts, err
}

func projectAsyncFunctionFacts(ctx context.Context, facts typefacts.Project, discoverer typefacts.AsyncFunctionDiscoverer) ([]typefacts.AsyncFunctionFact, error) {
	cache := solidCallCacheFrom(ctx)
	if cache != nil && cache.projectSet {
		return cache.projectAsync, cache.projectErr
	}
	files, err := facts.SourceFiles(ctx)
	all := make([]typefacts.AsyncFunctionFact, 0)
	if err == nil {
		for _, file := range files {
			var discovered []typefacts.AsyncFunctionFact
			discovered, err = sourceAsyncFunctionFacts(ctx, discoverer, filepath.Clean(file.Path))
			if err != nil {
				break
			}
			for index := range discovered {
				discovered[index].Expression.Path = filepath.Clean(discovered[index].Expression.Path)
				for callIndex := range discovered[index].CallsAfterAwait {
					discovered[index].CallsAfterAwait[callIndex].Path = filepath.Clean(discovered[index].CallsAfterAwait[callIndex].Path)
				}
			}
			all = append(all, discovered...)
		}
		resolveAsyncAliases(all)
		if cache != nil {
			clear(cache.asyncByPath)
			clear(cache.asyncBySymbol)
			for index := range all {
				fact := &all[index]
				cache.asyncByPath[fact.Expression.Path] = append(cache.asyncByPath[fact.Expression.Path], index)
				if fact.Symbol != "" {
					cache.asyncBySymbol[fact.Symbol] = index
				}
			}
		}
	}
	if cache != nil {
		cache.projectAsync, cache.projectErr, cache.projectSet = all, err, true
	}
	return all, err
}

func asyncFunctionInRange(ctx context.Context, facts []typefacts.AsyncFunctionFact, path string, start, end int) *typefacts.AsyncFunctionFact {
	indexes := make([]int, 0)
	if cache := solidCallCacheFrom(ctx); cache != nil {
		indexes = cache.asyncByPath[path]
	} else {
		for index := range facts {
			indexes = append(indexes, index)
		}
	}
	for _, index := range indexes {
		candidate := &facts[index]
		if candidate.Expression.Path == path && start <= candidate.Expression.StartByte && candidate.Expression.EndByte <= end {
			return candidate
		}
	}
	return nil
}

func asyncFunctionBySymbol(ctx context.Context, facts []typefacts.AsyncFunctionFact, symbol typefacts.SymbolID) *typefacts.AsyncFunctionFact {
	if cache := solidCallCacheFrom(ctx); cache != nil {
		if index, ok := cache.asyncBySymbol[symbol]; ok {
			return &facts[index]
		}
		return nil
	}
	for index := range facts {
		if facts[index].Symbol == symbol {
			return &facts[index]
		}
	}
	return nil
}

func resolveAsyncAliases(facts []typefacts.AsyncFunctionFact) {
	bySymbol := make(map[typefacts.SymbolID]int, len(facts))
	for index := range facts {
		if facts[index].Symbol != "" {
			bySymbol[facts[index].Symbol] = index
		}
	}
	for index := range facts {
		seen := make(map[typefacts.SymbolID]bool)
		target := facts[index].Target
		for target != "" && !seen[target] {
			seen[target] = true
			targetIndex, ok := bySymbol[target]
			if !ok {
				break
			}
			summary := facts[targetIndex]
			facts[index].Expression = summary.Expression
			facts[index].CanReturnAsync = summary.CanReturnAsync
			facts[index].CallsAfterAwait = summary.CallsAfterAwait
			if summary.Target == "" {
				break
			}
			target = summary.Target
		}
	}
}

func addStaticAPIDiagnostics(ctx context.Context, facts typefacts.Project, files []typefacts.SourceFile, program *Program) error {
	for _, file := range files {
		path := filepath.Clean(file.Path)
		effects, err := solidSourceCalls(ctx, facts, path, file.Source, "createEffect")
		if err != nil {
			return err
		}
		for _, call := range effects {
			missing := len(call.arguments) < 2
			if !missing {
				missing = strings.TrimSpace(string(file.Source[call.arguments[1].start:call.arguments[1].end])) == "undefined"
			}
			if missing {
				program.StaticViolations = append(program.StaticViolations, StaticViolation{
					ID: "SC7001", Rule: "missing-effect-function",
					Message:  "createEffect requires both a compute function and an effect function",
					Location: sourceLocation(typefacts.Location{Path: path, StartByte: call.callee.Start, EndByte: call.callee.End}, file.Source),
				})
			}
		}

		for _, primitive := range []struct {
			name         string
			optionsIndex int
		}{
			{"createMemo", 1}, {"createSignal", 1}, {"createStore", 2}, {"createProjection", 2},
			{"createOptimistic", 1}, {"createOptimisticStore", 2},
			{"createEffect", 2}, {"createRenderEffect", 2}, {"createTrackedEffect", 1},
		} {
			calls, err := solidSourceCalls(ctx, facts, path, file.Source, primitive.name)
			if err != nil {
				return err
			}
			for _, call := range calls {
				if len(call.arguments) <= primitive.optionsIndex || len(call.arguments) == 0 {
					continue
				}
				options := file.Source[call.arguments[primitive.optionsIndex].start:call.arguments[primitive.optionsIndex].end]
				if !syncTruePattern.Match(options) {
					continue
				}
				first, last := call.arguments[0].start, call.arguments[len(call.arguments)-1].end
				if !computationIsAsync(ctx, facts, path, file.Source, first, last) {
					continue
				}
				program.StaticViolations = append(program.StaticViolations, StaticViolation{
					ID: "SC7002", Rule: "sync-node-received-async",
					Message:  primitive.name + " uses sync: true but its computation can return a Promise or AsyncIterable",
					Location: sourceLocation(typefacts.Location{Path: path, StartByte: call.callee.Start, EndByte: call.callee.End}, file.Source),
				})
			}
		}
	}
	return nil
}

func terminalCleanupFix(path string, source []byte, ownerRegion, call compilerfacts.Span) *certification.Fix {
	open := call.End
	for open < len(source) && (source[open] == ' ' || source[open] == '\t') {
		open++
	}
	if open >= len(source) || source[open] != '(' {
		return nil
	}
	close := matchingBrace(source, open, '(', ')')
	if close <= open {
		return nil
	}
	bodyClose := ownerRegion.End - 1
	for bodyClose >= ownerRegion.Start && (source[bodyClose] == ' ' || source[bodyClose] == '\t' || source[bodyClose] == '\r' || source[bodyClose] == '\n') {
		bodyClose--
	}
	if bodyClose < ownerRegion.Start || source[bodyClose] != '}' {
		return nil
	}
	tail := close + 1
	for tail < bodyClose && (source[tail] == ' ' || source[tail] == '\t' || source[tail] == '\r' || source[tail] == '\n' || source[tail] == ';') {
		tail++
	}
	if tail != bodyClose {
		return nil
	}
	arguments := splitArguments(source, open+1, close-1)
	if len(arguments) != 1 {
		return nil
	}
	argument := strings.TrimSpace(string(source[arguments[0].start:arguments[0].end]))
	if argument == "" {
		return nil
	}
	location := sourceLocation(typefacts.Location{Path: path, StartByte: call.Start, EndByte: close}, source)
	return &certification.Fix{
		Message:       "return the cleanup function from the leaf-owner callback",
		Applicability: certification.FixSafe,
		Edits:         []certification.TextEdit{{Location: location, NewText: "return " + argument}},
	}
}

func solidCallArgumentRegions(ctx context.Context, facts typefacts.Project, path string, source []byte, name string, argument int) ([]compilerfacts.Span, error) {
	regions := make([]compilerfacts.Span, 0)
	calls, err := solidSourceCalls(ctx, facts, path, source, name)
	if err != nil {
		return nil, fmt.Errorf("resolve %s callback in %s: %w", name, path, err)
	}
	for _, call := range calls {
		if argument >= len(call.arguments) {
			continue
		}
		regions = append(regions, compilerfacts.Span{
			Start: call.arguments[argument].start,
			End:   call.arguments[argument].end,
		})
	}
	return regions, nil
}

func solidCallSpans(ctx context.Context, facts typefacts.Project, path string, source []byte, name string) ([]compilerfacts.Span, error) {
	spans := make([]compilerfacts.Span, 0)
	calls, err := solidSourceCalls(ctx, facts, path, source, name)
	if err != nil {
		return nil, fmt.Errorf("resolve %s call in %s: %w", name, path, err)
	}
	for _, call := range calls {
		spans = append(spans, call.callee)
	}
	return spans, nil
}

func isSolidCalleeAt(ctx context.Context, facts typefacts.Project, path string, start, end int, name string) (bool, error) {
	locations := []typefacts.Location{{Path: path, StartByte: start, EndByte: end}}
	if propertyStart := end - len(name); propertyStart > start {
		locations = append(locations, typefacts.Location{Path: path, StartByte: propertyStart, EndByte: end})
	}
	for _, location := range locations {
		symbol, err := facts.SymbolAt(ctx, location)
		if err != nil {
			if errors.Is(err, typefacts.ErrNotFound) {
				continue
			}
			return false, err
		}
		resolved, err := facts.ResolveAlias(ctx, symbol)
		if err == nil {
			symbol = resolved
		} else if !errors.Is(err, typefacts.ErrNotFound) {
			return false, err
		}
		if isSolidPrimitive(ctx, facts, symbol, name) {
			return true, nil
		}
	}
	return false, nil
}

func addSignalWrites(
	ctx context.Context,
	facts typefacts.Project,
	files []typefacts.SourceFile,
	sources map[string][]byte,
	functions map[string][]sourceFunction,
	executionMaps map[string]compilerfacts.ExecutionMap,
	program *Program,
) error {
	allowedByPath := make(map[string][]allowedWriteRegion, len(files))
	ownedByPath := make(map[string][]allowedWriteRegion, len(files))
	for _, file := range files {
		path := filepath.Clean(file.Path)
		allowed, err := allowedWriteRegions(ctx, facts, path, file.Source)
		if err != nil {
			return err
		}
		owned, err := ownedWriteRegions(ctx, facts, path, file.Source)
		if err != nil {
			return err
		}
		allowedByPath[path], ownedByPath[path] = allowed, owned
	}
	for _, file := range files {
		path := filepath.Clean(file.Path)
		type setterBinding struct {
			location   typefacts.Location
			ownedWrite bool
		}
		setters := make([]setterBinding, 0)
		if discoverer, ok := facts.(typefacts.BindingDiscoverer); ok {
			bindings, err := sourceBindingFacts(ctx, discoverer, path)
			if err != nil {
				return err
			}
			for _, binding := range bindings {
				primitive := solidPrimitiveName(ctx, facts, binding.Initializer.Target)
				if !binding.Array || len(binding.Names) < 2 || binding.Names[1].Path == "" ||
					(primitive != "createSignal" && primitive != "createStore" && primitive != "createOptimistic" && primitive != "createOptimisticStore") {
					continue
				}
				ownedWrite := (primitive == "createSignal" || primitive == "createOptimistic") && ownedWriteTruePattern.Match(file.Source[binding.Initializer.Location.StartByte:binding.Initializer.Location.EndByte])
				setters = append(setters, setterBinding{location: binding.Names[1], ownedWrite: ownedWrite})
			}
		} else {
			bindings := []struct {
				pattern    *regexp.Regexp
				primitive  string
				ownedWrite bool
			}{
				{signalSetterPattern, "createSignal", true}, {storeSetterPattern, "createStore", false},
				{signalSetterPattern, "createOptimistic", true}, {storeSetterPattern, "createOptimisticStore", false},
			}
			for _, binding := range bindings {
				for _, match := range binding.pattern.FindAllSubmatchIndex(file.Source, -1) {
					calleeStart, calleeEnd := match[4], match[5]
					call, err := facts.ResolvedCall(ctx, typefacts.Location{Path: path, StartByte: calleeStart, EndByte: calleeEnd})
					if err != nil {
						if errors.Is(err, typefacts.ErrNotFound) {
							continue
						}
						return fmt.Errorf("resolve %s setter candidate in %s: %w", binding.primitive, path, err)
					}
					if !isSolidPrimitive(ctx, facts, call.Target, binding.primitive) {
						continue
					}
					callEnd := matchingBrace(file.Source, match[1]-1, '(', ')')
					ownedWrite := binding.ownedWrite && callEnd > match[1] && ownedWriteTruePattern.Match(file.Source[match[1]:callEnd])
					setters = append(setters, setterBinding{location: typefacts.Location{Path: path, StartByte: match[2], EndByte: match[3]}, ownedWrite: ownedWrite})
				}
			}
		}
		for _, setter := range setters {
			ownedWrite := setter.ownedWrite
			setterSymbol, err := facts.SymbolAt(ctx, setter.location)
			if err != nil {
				return fmt.Errorf("resolve signal setter in %s: %w", path, err)
			}
			declarations, err := facts.Declarations(ctx, setterSymbol)
			if err != nil || len(declarations) == 0 {
				return fmt.Errorf("resolve signal setter declaration in %s: %w", path, err)
			}
			declarationSource := sources[filepath.Clean(declarations[0].Location.Path)]
			declaration := sourceLocation(declarations[0].Location, declarationSource)
			references, err := facts.References(ctx, setterSymbol)
			if err != nil {
				return fmt.Errorf("find references to signal setter in %s: %w", path, err)
			}
			for _, reference := range references {
				reference.Path = filepath.Clean(reference.Path)
				source, ok := sources[reference.Path]
				if !ok {
					continue
				}
				writeEnd, called := accessorCallEnd(source, reference.EndByte)
				if !called {
					continue
				}
				function, inFunction := functionContext(functions[reference.Path], reference.StartByte)
				span := compilerfacts.Span{Start: reference.StartByte, End: writeEnd}
				execution := executionRole(executionMaps[reference.Path], span)
				allowedBy := ""
				contextName := "owned computation"
				ownedRegion := false
				for _, region := range ownedByPath[reference.Path] {
					if contains(region.span, span) {
						ownedRegion, contextName = true, region.reason
						break
					}
				}
				if !inFunction && !ownedRegion {
					continue
				}
				if inFunction {
					contextName = function.name
				}
				if ownedWrite {
					allowedBy = "owned-write-option"
				}
				for _, callback := range executionMaps[reference.Path].CallbackRoles {
					if callback.Role == compilerfacts.CallbackEventHandler && contains(callback.Span, span) {
						allowedBy = "event-handler"
						break
					}
					if callback.Role == compilerfacts.CallbackDirectiveApply && contains(callback.Span, span) {
						allowedBy = "directive-apply"
						break
					}
				}
				for _, region := range allowedByPath[reference.Path] {
					if contains(region.span, span) {
						allowedBy = region.reason
						break
					}
				}
				write := ReactiveWrite{
					Setter:      string(source[reference.StartByte:reference.EndByte]),
					Location:    sourceLocation(typefacts.Location{Path: reference.Path, StartByte: reference.StartByte, EndByte: writeEnd}, source),
					Declaration: declaration, Execution: execution, Context: contextName, AllowedBy: allowedBy,
				}
				if inFunction {
					write.InReturnedClosure = containsOffset(function.returnedClosures, reference.StartByte)
				}
				if !inFunction || function.rendering {
					program.Writes = append(program.Writes, write)
				} else {
					program.Functions[function.programIndex].Writes = append(program.Functions[function.programIndex].Writes, write)
				}
			}
		}
	}
	sort.Slice(program.Writes, func(i, j int) bool {
		left, right := program.Writes[i].Location, program.Writes[j].Location
		if left.Path != right.Path {
			return left.Path < right.Path
		}
		return left.StartByte < right.StartByte
	})
	return nil
}

func addActionInvocations(
	ctx context.Context,
	facts typefacts.Project,
	files []typefacts.SourceFile,
	sources map[string][]byte,
	functions map[string][]sourceFunction,
	executionMaps map[string]compilerfacts.ExecutionMap,
	program *Program,
) error {
	allowedByPath := make(map[string][]allowedWriteRegion, len(files))
	ownedByPath := make(map[string][]allowedWriteRegion, len(files))
	for _, file := range files {
		path := filepath.Clean(file.Path)
		allowed, err := allowedWriteRegions(ctx, facts, path, file.Source)
		if err != nil {
			return err
		}
		owned, err := ownedWriteRegions(ctx, facts, path, file.Source)
		if err != nil {
			return err
		}
		roots, err := ownerProvidingRegions(ctx, facts, path, file.Source)
		if err != nil {
			return err
		}
		for _, root := range roots {
			owned = append(owned, allowedWriteRegion{span: root, reason: "owned callback"})
		}
		allowedByPath[path], ownedByPath[path] = allowed, owned
	}
	for _, file := range files {
		path := filepath.Clean(file.Path)
		actions := make([]typefacts.Location, 0)
		if discoverer, ok := facts.(typefacts.BindingDiscoverer); ok {
			bindings, err := sourceBindingFacts(ctx, discoverer, path)
			if err != nil {
				return err
			}
			for _, binding := range bindings {
				if binding.Array || len(binding.Names) == 0 || binding.Names[0].Path == "" || solidPrimitiveName(ctx, facts, binding.Initializer.Target) != "action" {
					continue
				}
				actions = append(actions, binding.Names[0])
			}
		} else {
			for _, match := range asyncDirectPattern.FindAllSubmatchIndex(file.Source, -1) {
				call, err := facts.ResolvedCall(ctx, typefacts.Location{Path: path, StartByte: match[4], EndByte: match[5]})
				if err == nil && isSolidPrimitive(ctx, facts, call.Target, "action") {
					actions = append(actions, typefacts.Location{Path: path, StartByte: match[2], EndByte: match[3]})
				}
			}
		}
		for _, action := range actions {
			symbol, err := facts.SymbolAt(ctx, action)
			if err != nil {
				return fmt.Errorf("resolve action result in %s: %w", path, err)
			}
			references, err := facts.References(ctx, symbol)
			if err != nil {
				return fmt.Errorf("find action references in %s: %w", path, err)
			}
			for _, reference := range references {
				reference.Path = filepath.Clean(reference.Path)
				source, ok := sources[reference.Path]
				if !ok {
					continue
				}
				end, called := accessorCallEnd(source, reference.EndByte)
				if !called {
					continue
				}
				span := compilerfacts.Span{Start: reference.StartByte, End: end}
				function, inFunction := functionContext(functions[reference.Path], reference.StartByte)
				owned := false
				contextName := "owned scope"
				for _, region := range ownedByPath[reference.Path] {
					if contains(region.span, span) {
						owned, contextName = true, region.reason
						break
					}
				}
				if !inFunction && !owned {
					continue
				}
				if inFunction {
					contextName = function.name
				}
				allowedBy := ""
				for _, callback := range executionMaps[reference.Path].CallbackRoles {
					if (callback.Role == compilerfacts.CallbackEventHandler || callback.Role == compilerfacts.CallbackDirectiveApply) && contains(callback.Span, span) {
						allowedBy = string(callback.Role)
						break
					}
				}
				for _, region := range allowedByPath[reference.Path] {
					if contains(region.span, span) {
						allowedBy = region.reason
						break
					}
				}
				invocation := ActionInvocation{
					Action:   string(source[reference.StartByte:reference.EndByte]),
					Location: sourceLocation(typefacts.Location{Path: reference.Path, StartByte: reference.StartByte, EndByte: end}, source),
					Context:  contextName, AllowedBy: allowedBy,
				}
				if !inFunction || function.rendering {
					program.ActionCalls = append(program.ActionCalls, invocation)
				} else {
					program.Functions[function.programIndex].ActionCalls = append(program.Functions[function.programIndex].ActionCalls, invocation)
				}
			}
		}
	}
	return nil
}

type reactiveBindingFact struct {
	kind        ReactiveValueKind
	declaration certification.SourceLocation
	ownedWrite  bool
}

func collectReactiveBindingFacts(ctx context.Context, facts typefacts.Project, files []typefacts.SourceFile) (map[typefacts.SymbolID]reactiveBindingFact, error) {
	if discoverer, ok := facts.(typefacts.BindingDiscoverer); ok {
		return collectDiscoveredReactiveBindingFacts(ctx, facts, discoverer, files)
	}
	result := make(map[typefacts.SymbolID]reactiveBindingFact)
	bindings := []struct {
		pattern   *regexp.Regexp
		primitive string
		kind      ReactiveValueKind
	}{
		{signalBindingPattern, "createSignal", ReactiveAccessor},
		{signalBindingPattern, "createOptimistic", ReactiveAccessor},
		{asyncDirectPattern, "createMemo", ReactiveAccessor},
		{storeBindingPattern, "createStore", ReactiveStorePath},
		{storeBindingPattern, "createOptimisticStore", ReactiveStorePath},
		{asyncDirectPattern, "createProjection", ReactiveStorePath},
	}
	for _, file := range files {
		path := filepath.Clean(file.Path)
		for _, binding := range bindings {
			for _, match := range binding.pattern.FindAllSubmatchIndex(file.Source, -1) {
				call, err := facts.ResolvedCall(ctx, typefacts.Location{Path: path, StartByte: match[4], EndByte: match[5]})
				if err != nil || !isSolidPrimitive(ctx, facts, call.Target, binding.primitive) {
					continue
				}
				symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: match[2], EndByte: match[3]})
				if err != nil {
					return nil, err
				}
				callEnd := matchingBrace(file.Source, match[1]-1, '(', ')')
				ownedWrite := false
				if (binding.primitive == "createSignal" || binding.primitive == "createOptimistic") && callEnd > match[1] {
					ownedWrite = ownedWriteTruePattern.Match(file.Source[match[1]:callEnd])
				}
				result[canonicalSymbol(ctx, facts, symbol)] = reactiveBindingFact{
					kind: binding.kind, ownedWrite: ownedWrite,
					declaration: sourceLocation(typefacts.Location{Path: path, StartByte: match[2], EndByte: match[3]}, file.Source),
				}
			}
		}
	}
	return result, nil
}

func collectDiscoveredReactiveBindingFacts(ctx context.Context, facts typefacts.Project, discoverer typefacts.BindingDiscoverer, files []typefacts.SourceFile) (map[typefacts.SymbolID]reactiveBindingFact, error) {
	result := make(map[typefacts.SymbolID]reactiveBindingFact)
	for _, file := range files {
		path := filepath.Clean(file.Path)
		bindings, err := sourceBindingFacts(ctx, discoverer, path)
		if err != nil {
			return nil, err
		}
		for _, binding := range bindings {
			primitive := solidPrimitiveName(ctx, facts, binding.Initializer.Target)
			kind := ReactiveValueKind("")
			switch {
			case (primitive == "createSignal" || primitive == "createOptimistic") && binding.Array:
				kind = ReactiveAccessor
			case primitive == "createMemo" && !binding.Array:
				kind = ReactiveAccessor
			case (primitive == "createStore" || primitive == "createOptimisticStore") && binding.Array:
				kind = ReactiveStorePath
			case primitive == "createProjection" && !binding.Array:
				kind = ReactiveStorePath
			default:
				continue
			}
			if len(binding.Names) == 0 || binding.Names[0].Path == "" {
				continue
			}
			location := binding.Names[0]
			symbol, err := facts.SymbolAt(ctx, location)
			if err != nil {
				return nil, err
			}
			ownedWrite := (primitive == "createSignal" || primitive == "createOptimistic") && ownedWriteTruePattern.Match(file.Source[binding.Initializer.Location.StartByte:binding.Initializer.Location.EndByte])
			result[canonicalSymbol(ctx, facts, symbol)] = reactiveBindingFact{
				kind: kind, ownedWrite: ownedWrite, declaration: sourceLocation(location, file.Source),
			}
		}
	}
	return result, nil
}

func addRefreshAndAffectsDiagnostics(
	ctx context.Context,
	facts typefacts.Project,
	files []typefacts.SourceFile,
	sources map[string][]byte,
	functions map[string][]sourceFunction,
	executionMaps map[string]compilerfacts.ExecutionMap,
	program *Program,
) error {
	bindings, err := collectReactiveBindingFacts(ctx, facts, files)
	if err != nil {
		return err
	}
	allowedByPath := make(map[string][]allowedWriteRegion, len(files))
	ownedByPath := make(map[string][]allowedWriteRegion, len(files))
	for _, file := range files {
		path := filepath.Clean(file.Path)
		allowedByPath[path], err = allowedWriteRegions(ctx, facts, path, file.Source)
		if err != nil {
			return err
		}
		ownedByPath[path], err = ownedWriteRegions(ctx, facts, path, file.Source)
		if err != nil {
			return err
		}
		roots, rootErr := ownerProvidingRegions(ctx, facts, path, file.Source)
		if rootErr != nil {
			return rootErr
		}
		for _, root := range roots {
			ownedByPath[path] = append(ownedByPath[path], allowedWriteRegion{span: root, reason: "owned callback"})
		}
	}
	for _, file := range files {
		path := filepath.Clean(file.Path)
		for _, api := range []string{"refresh", "affects"} {
			calls, err := solidSourceCalls(ctx, facts, path, file.Source, api)
			if err != nil {
				return err
			}
			for _, call := range calls {
				if len(call.arguments) == 0 || api == "refresh" && len(call.arguments) != 1 || api == "affects" && len(call.arguments) > 2 {
					program.StaticViolations = append(program.StaticViolations, StaticViolation{
						ID: "SC7003", Rule: "invalid-" + api + "-target",
						Message:  api + "() received an invalid number of target arguments",
						Location: sourceLocation(typefacts.Location{Path: path, StartByte: call.callee.Start, EndByte: call.callee.End}, file.Source),
					})
					continue
				}
				target := call.arguments[0]
				start, end := trimByteSpan(file.Source, target.start, target.end)
				var binding reactiveBindingFact
				known := false
				if identifierPattern.Match(file.Source[start:end]) {
					symbol, symbolErr := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: start, EndByte: end})
					if symbolErr == nil {
						binding, known = bindings[canonicalSymbol(ctx, facts, symbol)]
					}
					if !known {
						program.Unresolved = append(program.Unresolved, UnresolvedObligation{
							ID: "SC9003", Rule: api + "-target-unresolved",
							Message:  "cannot prove that " + string(file.Source[start:end]) + " is a branded Solid source accepted by " + api,
							Location: sourceLocation(typefacts.Location{Path: path, StartByte: start, EndByte: end}, file.Source),
						})
						continue
					}
				} else {
					program.StaticViolations = append(program.StaticViolations, StaticViolation{
						ID: "SC7003", Rule: "invalid-" + api + "-target",
						Message:  api + "() expects the original Solid source accessor or store, not a wrapper, read value, or literal",
						Location: sourceLocation(typefacts.Location{Path: path, StartByte: start, EndByte: end}, file.Source),
					})
					continue
				}
				if api == "affects" {
					if binding.kind == ReactiveAccessor && len(call.arguments) == 2 {
						program.StaticViolations = append(program.StaticViolations, StaticViolation{
							ID: "SC7004", Rule: "invalid-affects-target",
							Message:  "affects() keys are only valid on store targets",
							Location: sourceLocation(typefacts.Location{Path: path, StartByte: call.callee.Start, EndByte: call.callee.End}, file.Source),
						})
					}
					continue
				}
				if binding.ownedWrite {
					continue
				}
				span := compilerfacts.Span{Start: call.callee.Start, End: call.arguments[len(call.arguments)-1].end}
				function, inFunction := functionContext(functions[path], call.callee.Start)
				owned, contextName := false, "owned scope"
				for _, region := range ownedByPath[path] {
					if contains(region.span, span) {
						owned, contextName = true, region.reason
						break
					}
				}
				if !inFunction && !owned {
					continue
				}
				if inFunction {
					contextName = function.name
				}
				allowedBy := ""
				for _, callback := range executionMaps[path].CallbackRoles {
					if (callback.Role == compilerfacts.CallbackEventHandler || callback.Role == compilerfacts.CallbackDirectiveApply) && contains(callback.Span, span) {
						allowedBy = string(callback.Role)
					}
				}
				for _, region := range allowedByPath[path] {
					if contains(region.span, span) {
						allowedBy = region.reason
					}
				}
				write := ReactiveWrite{
					Setter: "refresh", Location: sourceLocation(typefacts.Location{Path: path, StartByte: call.callee.Start, EndByte: span.End}, file.Source),
					Declaration: binding.declaration, Context: contextName, AllowedBy: allowedBy,
				}
				if !inFunction || function.rendering {
					program.Writes = append(program.Writes, write)
				} else {
					program.Functions[function.programIndex].Writes = append(program.Functions[function.programIndex].Writes, write)
				}
			}
		}
	}
	_ = sources
	return nil
}

func trimByteSpan(source []byte, start, end int) (int, int) {
	for start < end && (source[start] == ' ' || source[start] == '\t' || source[start] == '\r' || source[start] == '\n') {
		start++
	}
	for end > start && (source[end-1] == ' ' || source[end-1] == '\t' || source[end-1] == '\r' || source[end-1] == '\n') {
		end--
	}
	return start, end
}

func ownedWriteRegions(ctx context.Context, facts typefacts.Project, path string, source []byte) ([]allowedWriteRegion, error) {
	cache := solidCallCacheFrom(ctx)
	if cache != nil {
		if regions, ok := cache.ownedWrites[path]; ok {
			return regions, cache.ownedErrs[path]
		}
	}
	regions, err := discoverOwnedWriteRegions(ctx, facts, path, source)
	if cache != nil {
		cache.ownedWrites[path] = regions
		cache.ownedErrs[path] = err
	}
	return regions, err
}

func discoverOwnedWriteRegions(ctx context.Context, facts typefacts.Project, path string, source []byte) ([]allowedWriteRegion, error) {
	roles := []struct {
		name     string
		argument int
	}{
		{"createMemo", 0}, {"createEffect", 0}, {"createRenderEffect", 0}, {"createProjection", 0},
		{"createSignal", 0}, {"createStore", 0}, {"createOptimistic", 0}, {"createOptimisticStore", 0},
	}
	regions := make([]allowedWriteRegion, 0)
	for _, role := range roles {
		spans, err := solidCallArgumentRegions(ctx, facts, path, source, role.name, role.argument)
		if err != nil {
			return nil, err
		}
		for _, span := range spans {
			if !functionLikeArgument(source, span) {
				continue
			}
			regions = append(regions, allowedWriteRegion{span: span, reason: role.name + " compute"})
		}
	}
	return regions, nil
}

func trackedReadRegions(ctx context.Context, facts typefacts.Project, path string, source []byte) ([]allowedWriteRegion, error) {
	regions, err := ownedWriteRegions(ctx, facts, path, source)
	if err != nil {
		return nil, err
	}
	regions = append([]allowedWriteRegion(nil), regions...)
	trackedEffects, err := solidCallArgumentRegions(ctx, facts, path, source, "createTrackedEffect", 0)
	if err != nil {
		return nil, err
	}
	for _, span := range trackedEffects {
		if functionLikeArgument(source, span) {
			regions = append(regions, allowedWriteRegion{span: span, reason: "createTrackedEffect compute"})
		}
	}
	return regions, nil
}

func readExecutionRegions(ctx context.Context, facts typefacts.Project, path string, source []byte) ([]allowedWriteRegion, error) {
	regions, err := trackedReadRegions(ctx, facts, path, source)
	if err != nil {
		return nil, err
	}
	for _, primitive := range []string{"action", "untrack", "onSettled"} {
		spans, err := solidCallArgumentRegions(ctx, facts, path, source, primitive, 0)
		if err != nil {
			return nil, err
		}
		for _, span := range spans {
			if functionLikeArgument(source, span) {
				regions = append(regions, allowedWriteRegion{span: span, reason: "non-strict " + primitive})
			}
		}
	}
	return regions, nil
}

type allowedWriteRegion struct {
	span   compilerfacts.Span
	reason string
}

func allowedWriteRegions(ctx context.Context, facts typefacts.Project, path string, source []byte) ([]allowedWriteRegion, error) {
	type primitiveRole struct {
		name     string
		argument int
		reason   string
	}
	roles := []primitiveRole{
		{name: "action", argument: 0, reason: "action"},
		{name: "untrack", argument: 0, reason: "untracked-callback"},
		{name: "onSettled", argument: 0, reason: "on-settled"},
		{name: "createTrackedEffect", argument: 0, reason: "tracked-effect"},
		{name: "createEffect", argument: 1, reason: "effect-apply"},
		{name: "createRenderEffect", argument: 1, reason: "effect-apply"},
	}
	regions := make([]allowedWriteRegion, 0)
	for _, role := range roles {
		spans, err := solidCallArgumentRegions(ctx, facts, path, source, role.name, role.argument)
		if err != nil {
			return nil, err
		}
		for _, argument := range spans {
			regions = append(regions, allowedWriteRegion{
				span: argument, reason: role.reason,
			})
		}
	}
	return regions, nil
}

func declarationName(source []byte, start, end int) (int, int, bool) {
	index := start
	for index < end {
		for index < end && (source[index] == ' ' || source[index] == '\t' || source[index] == '\r' || source[index] == '\n') {
			index++
		}
		if index+1 < end && source[index] == '/' && source[index+1] == '/' {
			index += 2
			for index < end && source[index] != '\n' {
				index++
			}
			continue
		}
		if index+1 < end && source[index] == '/' && source[index+1] == '*' {
			index += 2
			for index+1 < end && !(source[index] == '*' && source[index+1] == '/') {
				index++
			}
			index += 2
			continue
		}
		break
	}
	finish := index
	for finish < end && ((source[finish] >= 'A' && source[finish] <= 'Z') ||
		(source[finish] >= 'a' && source[finish] <= 'z') ||
		(source[finish] >= '0' && source[finish] <= '9') || source[finish] == '_' || source[finish] == '$') {
		finish++
	}
	return index, finish, finish > index
}

func statementEnd(source []byte, start int) int {
	paren, bracket, brace, angle := 0, 0, 0, 0
	for index := start; index < len(source); index++ {
		switch source[index] {
		case '(':
			paren++
		case ')':
			paren--
		case '[':
			bracket++
		case ']':
			bracket--
		case '{':
			brace++
		case '}':
			brace--
		case '<':
			angle++
		case '>':
			if angle > 0 {
				angle--
			}
		case ';':
			if paren == 0 && bracket == 0 && brace == 0 && angle == 0 {
				return index
			}
		}
	}
	return -1
}

func addContractExports(
	ctx context.Context,
	facts typefacts.Project,
	sourceFiles []typefacts.SourceFile,
	packageContracts []contracts.Contract,
	program *Program,
) ([]callableTarget, error) {
	byPackage := make(map[string]contracts.Contract, len(packageContracts))
	for _, contract := range packageContracts {
		byPackage[contract.Package.Name] = contract
	}
	targets := make([]callableTarget, 0)
	seen := make(map[FunctionID]struct{})
	for _, file := range sourceFiles {
		path := filepath.Clean(file.Path)
		for _, match := range namedImportPattern.FindAllSubmatchIndex(file.Source, -1) {
			packageName := string(file.Source[match[4]:match[5]])
			contract, ok := byPackage[packageName]
			if !ok {
				for contractedPackage, candidate := range byPackage {
					if strings.HasPrefix(packageName, contractedPackage+"/") {
						contract, ok = candidate, true
						break
					}
				}
			}
			if !ok {
				continue
			}
			for _, span := range splitArguments(file.Source, match[2], match[3]) {
				specifier := importSpecifierPattern.FindSubmatchIndex(file.Source[span.start:span.end])
				if specifier == nil {
					continue
				}
				exportName := string(file.Source[span.start+specifier[2] : span.start+specifier[3]])
				summary, ok := contract.Exports[exportName]
				if !ok {
					program.Unresolved = append(program.Unresolved, UnresolvedObligation{
						Message: fmt.Sprintf("package contract for %s does not describe imported export %s", packageName, exportName),
						Location: sourceLocation(typefacts.Location{
							Path: path, StartByte: span.start + specifier[2], EndByte: span.start + specifier[3],
						}, file.Source),
					})
					continue
				}
				if len(summary.ReactiveReads) == 0 && summary.Returns == nil && len(summary.Callbacks) == 0 {
					continue
				}
				localStart, localEnd := span.start+specifier[2], span.start+specifier[3]
				localName := exportName
				if specifier[4] >= 0 {
					localStart, localEnd = span.start+specifier[4], span.start+specifier[5]
					localName = string(file.Source[localStart:localEnd])
				}
				symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: localStart, EndByte: localEnd})
				if err != nil {
					if errors.Is(err, typefacts.ErrNotFound) {
						continue
					}
					return nil, fmt.Errorf("resolve contracted import %s from %s: %w", localName, packageName, err)
				}
				symbol = canonicalSymbol(ctx, facts, symbol)
				id := FunctionID(symbol)
				if _, duplicate := seen[id]; duplicate {
					continue
				}
				seen[id] = struct{}{}
				reads := make([]ReactiveRead, 0, len(summary.ReactiveReads))
				location := certification.SourceLocation{
					Path: contract.Path + "#" + exportName, Line: 1, Column: 1,
				}
				for _, contracted := range summary.ReactiveReads {
					kind := ReactiveAccessor
					if contracted.Kind == "store-path" {
						kind = ReactiveStorePath
					}
					reads = append(reads, ReactiveRead{
						Kind: kind, Accessor: packageName + "." + exportName,
						Location: location, Declaration: location,
						Execution: ExecutionInline, Context: exportName,
					})
				}
				returnedReads := make([]ReactiveRead, 0, 1)
				if summary.Returns != nil {
					kind := ReactiveAccessor
					if summary.Returns.Kind == "store-path" {
						kind = ReactiveStorePath
					}
					returnedReads = append(returnedReads, ReactiveRead{
						Kind: kind, Accessor: summary.Returns.Label,
						Location: location, Declaration: location,
						Execution: ExecutionInline, Context: exportName,
					})
				}
				callbackInvocations := make([]CallbackInvocation, 0, len(summary.Callbacks))
				for _, callback := range summary.Callbacks {
					execution := ExecutionInline
					if callback.Execution == "tracked" {
						execution = ExecutionTrackedJSX
					} else if callback.Execution == "deferred" {
						execution = ExecutionDeferredCallback
					}
					callbackInvocations = append(callbackInvocations, CallbackInvocation{
						Parameter: callback.Parameter, Location: location,
						Execution: execution, Context: exportName,
					})
				}
				program.Functions = append(program.Functions, Function{
					ID: id, Name: localName, Reads: reads,
					ReturnedReads: returnedReads, Calls: []FunctionCall{}, CallbackInvocations: callbackInvocations,
				})
				targets = append(targets, callableTarget{id: id, symbol: symbol, name: localName})
			}
		}
	}
	return targets, nil
}

func isSolidCreateSignal(ctx context.Context, facts typefacts.Project, target typefacts.SymbolID) bool {
	return isSolidPrimitive(ctx, facts, target, "createSignal")
}

func isSolidPrimitive(ctx context.Context, facts typefacts.Project, target typefacts.SymbolID, name string) bool {
	return solidPrimitiveName(ctx, facts, target) == name
}

func solidPrimitiveName(ctx context.Context, facts typefacts.Project, target typefacts.SymbolID) string {
	declarations, err := facts.Declarations(ctx, target)
	if err != nil {
		return ""
	}
	for _, declaration := range declarations {
		path := strings.ToLower(filepath.ToSlash(declaration.Location.Path))
		if declaration.Name != "" && strings.Contains(path, "solid-js") {
			return declaration.Name
		}
	}
	return ""
}

func jsxSolidPrimitive(ctx context.Context, facts typefacts.Project, path string, source []byte, span compilerfacts.Span, names ...string) string {
	if span.Start < 0 || span.Start+1 >= len(source) || source[span.Start] != '<' {
		return ""
	}
	start, end := span.Start+1, jsxTagEnd(source, span)
	if end == start {
		return ""
	}
	if dot := bytes.LastIndexByte(source[start:end], '.'); dot >= 0 {
		start += dot + 1
	}
	symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: start, EndByte: end})
	if err != nil {
		return ""
	}
	symbol = canonicalSymbol(ctx, facts, symbol)
	for _, name := range names {
		if isSolidPrimitive(ctx, facts, symbol, name) {
			return name
		}
	}
	return ""
}

func jsxTagEnd(source []byte, span compilerfacts.Span) int {
	if span.Start < 0 || span.Start+1 >= len(source) || source[span.Start] != '<' {
		return span.Start
	}
	end := span.Start + 1
	for end < len(source) && (isIdentifierByte(source[end]) || source[end] == '.') {
		end++
	}
	return end
}

func solidJSXOperationSpans(ctx context.Context, facts typefacts.Project, path string, source []byte, executionMap compilerfacts.ExecutionMap, name string) []compilerfacts.Span {
	spans := make([]compilerfacts.Span, 0)
	for _, operation := range executionMap.JsxOperations {
		if operation.Kind == "component-invocation" && jsxSolidPrimitive(ctx, facts, path, source, operation.Span, name) != "" {
			spans = append(spans, operation.Span)
		}
	}
	return spans
}

func propertyAccessEnd(source []byte, identifierEnd int) (int, bool) {
	index := identifierEnd
	found := false
	for index < len(source) && source[index] == '.' {
		propertyStart := index + 1
		propertyEnd := propertyStart
		for propertyEnd < len(source) && ((source[propertyEnd] >= 'A' && source[propertyEnd] <= 'Z') ||
			(source[propertyEnd] >= 'a' && source[propertyEnd] <= 'z') ||
			(source[propertyEnd] >= '0' && source[propertyEnd] <= '9') || source[propertyEnd] == '_' || source[propertyEnd] == '$') {
			propertyEnd++
		}
		if propertyEnd == propertyStart {
			break
		}
		found = true
		index = propertyEnd
	}
	return index, found
}

// addIncompleteExecutionMaps enforces the compiler-facts completeness
// invariant: every jsx-expression operation must be covered by a tracked
// region, an untracked region, a callback role, or a component-property
// operation. An uncovered hole means fact recording has no branch for the
// construct, so reads inside it must surface as unresolved obligations
// instead of silently defaulting to ExecutionUntrackedRendering.
func addIncompleteExecutionMaps(sourceFiles []typefacts.SourceFile, executionMaps map[string]compilerfacts.ExecutionMap, program *Program) {
	for _, file := range sourceFiles {
		path := filepath.Clean(file.Path)
		for _, span := range compilerfacts.UncoveredJSXExpressions(executionMaps[path]) {
			program.Unresolved = append(program.Unresolved, UnresolvedObligation{
				ID: "SC9004", Rule: "execution-map-incomplete",
				Message:  "compiler facts do not classify this JSX expression as tracked, untracked, or a callback",
				Location: sourceLocation(typefacts.Location{Path: path, StartByte: span.Start, EndByte: span.End}, file.Source),
			})
		}
	}
}

func executionRole(executionMap compilerfacts.ExecutionMap, read compilerfacts.Span) ExecutionRole {
	for _, region := range executionMap.TrackedRegions {
		if contains(region.Span, read) {
			return ExecutionTrackedJSX
		}
	}
	// Dynamic custom-component props are emitted as lazy getters by the Solid
	// compiler. The compiler records these as component-property operations
	// rather than ordinary trackedRegions, but reads inside them still execute
	// reactively when the child component consumes the prop.
	for _, operation := range executionMap.JsxOperations {
		if operation.Kind == "component-property" && contains(operation.Span, read) {
			return ExecutionTrackedJSX
		}
	}
	for _, callback := range executionMap.CallbackRoles {
		if contains(callback.Span, read) {
			switch callback.Role {
			case compilerfacts.CallbackRender:
				return ExecutionUntrackedRendering
			case compilerfacts.CallbackEventHandler:
				return ExecutionEventCallback
			case compilerfacts.CallbackDirectiveApply:
				return ExecutionDirectiveApply
			}
			return ExecutionDeferredCallback
		}
	}
	// An explicit untracked region is the compiler stating the expression
	// renders once without tracking. The bare fallback below covers spans the
	// compiler never saw (module level, plain statements); JSX expression
	// holes that reach it without any covering fact are reported as
	// unresolved by addIncompleteExecutionMaps.
	for _, region := range executionMap.UntrackedRegions {
		if contains(region.Span, read) {
			return ExecutionUntrackedRendering
		}
	}
	return ExecutionUntrackedRendering
}

func executionRoleWithComputations(executionMap compilerfacts.ExecutionMap, computations []allowedWriteRegion, span compilerfacts.Span) ExecutionRole {
	role := executionRole(executionMap, span)
	if role == ExecutionTrackedJSX || role == ExecutionEventCallback || role == ExecutionDirectiveApply {
		return role
	}
	for _, computation := range computations {
		if contains(computation.span, span) {
			if strings.HasPrefix(computation.reason, "non-strict ") {
				return ExecutionDeferredCallback
			}
			return ExecutionTrackedComputation
		}
	}
	return role
}

func contains(outer, inner compilerfacts.Span) bool {
	return outer.Start <= inner.Start && inner.End <= outer.End
}

func containedByAny(regions []compilerfacts.Span, inner compilerfacts.Span) bool {
	for _, region := range regions {
		if contains(region, inner) {
			return true
		}
	}
	return false
}

func accessorCallEnd(source []byte, identifierEnd int) (int, bool) {
	index := identifierEnd
	for index < len(source) && (source[index] == ' ' || source[index] == '\t' || source[index] == '\r' || source[index] == '\n') {
		index++
	}
	if index >= len(source) || source[index] != '(' {
		return 0, false
	}
	end := matchingBrace(source, index, '(', ')')
	return end, end > index
}

func isReturnedIdentifier(source []byte, start, end int) bool {
	return hasReturnPrefix(source, start) && hasReturnSuffix(source, end)
}

func hasReturnPrefix(source []byte, start int) bool {
	left := start
	for left > 0 && (source[left-1] == ' ' || source[left-1] == '\t') {
		left--
	}
	const keyword = "return"
	if left < len(keyword) || string(source[left-len(keyword):left]) != keyword {
		return false
	}
	if left > len(keyword) {
		previous := source[left-len(keyword)-1]
		if (previous >= 'A' && previous <= 'Z') || (previous >= 'a' && previous <= 'z') ||
			(previous >= '0' && previous <= '9') || previous == '_' || previous == '$' {
			return false
		}
	}
	return true
}

func hasReturnSuffix(source []byte, end int) bool {
	right := end
	for right < len(source) && (source[right] == ' ' || source[right] == '\t') {
		right++
	}
	return right == len(source) || source[right] == ';' || source[right] == '\r' || source[right] == '\n'
}

type sourceFunction struct {
	id               FunctionID
	symbol           typefacts.SymbolID
	name             string
	bodyStart        int
	bodyEnd          int
	rendering        bool
	exported         bool
	async            bool
	programIndex     int
	parameters       []typefacts.SymbolID
	parameterSpans   []byteSpan
	returnedClosures []byteSpan
}

func declaredFunctions(ctx context.Context, facts typefacts.Project, path string, source []byte, programBase int) ([]sourceFunction, error) {
	if discoverer, ok := facts.(typefacts.FunctionDiscoverer); ok {
		discovered, err := discoverer.SourceFunctions(ctx, path)
		if err != nil {
			return nil, err
		}
		functions := make([]sourceFunction, 0, len(discovered))
		for _, arrow := range []bool{false, true} {
			for _, function := range discovered {
				if function.Arrow != arrow || function.Name.StartByte < 0 || function.Name.EndByte > len(source) || function.Body.StartByte < 0 || function.Body.EndByte > len(source) {
					continue
				}
				name := string(source[function.Name.StartByte:function.Name.EndByte])
				symbol, err := facts.SymbolAt(ctx, function.Name)
				if err != nil {
					if errors.Is(err, typefacts.ErrNotFound) {
						continue
					}
					return nil, fmt.Errorf("resolve function %s in %s: %w", name, path, err)
				}
				symbol = canonicalSymbol(ctx, facts, symbol)
				parameterIDs := make([]typefacts.SymbolID, 0, len(function.Parameters))
				parameterSpans := make([]byteSpan, 0, len(function.Parameters))
				for _, parameter := range function.Parameters {
					parameterSpans = append(parameterSpans, byteSpan{start: parameter.StartByte, end: parameter.EndByte})
					match := parameterPattern.FindSubmatchIndex(source[parameter.StartByte:parameter.EndByte])
					if match == nil {
						parameterIDs = append(parameterIDs, "")
						continue
					}
					parameterSymbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: parameter.StartByte + match[2], EndByte: parameter.StartByte + match[3]})
					if err != nil {
						return nil, fmt.Errorf("resolve parameters for function %s in %s: %w", name, path, err)
					}
					parameterIDs = append(parameterIDs, parameterSymbol)
				}
				functions = append(functions, sourceFunction{
					id: FunctionID(symbol), symbol: symbol, name: name,
					bodyStart: function.Body.StartByte, bodyEnd: function.Body.EndByte,
					rendering: name[0] >= 'A' && name[0] <= 'Z' && jsxPattern.Match(source[function.Body.StartByte:function.Body.EndByte]),
					exported:  function.Exported, async: function.Async, programIndex: programBase + len(functions),
					parameters: parameterIDs, parameterSpans: parameterSpans,
					returnedClosures: returnedClosureSpans(source, function.Body.StartByte, function.Body.EndByte),
				})
			}
		}
		return functions, nil
	}
	functions := make([]sourceFunction, 0)
	for _, match := range functionPattern.FindAllSubmatchIndex(source, -1) {
		cursor := match[1]
		if cursor < len(source) && source[cursor] == '<' {
			cursor = matchingBrace(source, cursor, '<', '>')
			if cursor == 0 {
				continue
			}
		}
		for cursor < len(source) && (source[cursor] == ' ' || source[cursor] == '\t' || source[cursor] == '\r' || source[cursor] == '\n') {
			cursor++
		}
		if cursor >= len(source) || source[cursor] != '(' {
			continue
		}
		parametersStart := cursor + 1
		parametersEnd := matchingBrace(source, cursor, '(', ')')
		if parametersEnd <= parametersStart {
			continue
		}
		bodyStart := functionBodyStart(source, parametersEnd)
		if bodyStart < 0 {
			continue
		}
		bodyEnd := matchingBrace(source, bodyStart, '{', '}')
		if bodyEnd <= bodyStart {
			continue
		}
		name := string(source[match[2]:match[3]])
		symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: match[2], EndByte: match[3]})
		if err != nil {
			if errors.Is(err, typefacts.ErrNotFound) {
				continue
			}
			return nil, fmt.Errorf("resolve function %s in %s: %w", name, path, err)
		}
		symbol = canonicalSymbol(ctx, facts, symbol)
		parameterSpans := splitArguments(source, parametersStart, parametersEnd-1)
		parameters, err := functionParameters(ctx, facts, path, source, parametersStart, parametersEnd-1)
		if err != nil {
			return nil, fmt.Errorf("resolve parameters for function %s in %s: %w", name, path, err)
		}
		functions = append(functions, sourceFunction{
			id:               FunctionID(symbol),
			symbol:           symbol,
			name:             name,
			bodyStart:        bodyStart,
			bodyEnd:          bodyEnd,
			rendering:        name[0] >= 'A' && name[0] <= 'Z' && jsxPattern.Match(source[bodyStart:bodyEnd]),
			exported:         bytes.Contains(source[match[0]:match[2]], []byte("export")),
			async:            bytes.Contains(source[match[0]:match[2]], []byte("async")),
			programIndex:     programBase + len(functions),
			parameters:       parameters,
			parameterSpans:   parameterSpans,
			returnedClosures: returnedClosureSpans(source, bodyStart, bodyEnd),
		})
	}
	for _, match := range arrowFunctionPattern.FindAllSubmatchIndex(source, -1) {
		cursor := match[1]
		for cursor < len(source) && (source[cursor] == ' ' || source[cursor] == '\t' || source[cursor] == '\r' || source[cursor] == '\n') {
			cursor++
		}
		async := bytes.HasPrefix(source[cursor:], []byte("async"))
		if async {
			cursor += len("async")
			for cursor < len(source) && (source[cursor] == ' ' || source[cursor] == '\t' || source[cursor] == '\r' || source[cursor] == '\n') {
				cursor++
			}
		}
		if cursor < len(source) && source[cursor] == '<' {
			cursor = matchingBrace(source, cursor, '<', '>')
			for cursor < len(source) && (source[cursor] == ' ' || source[cursor] == '\t' || source[cursor] == '\r' || source[cursor] == '\n') {
				cursor++
			}
		}
		if cursor >= len(source) || source[cursor] != '(' {
			continue
		}
		parametersStart := cursor + 1
		parametersEnd := matchingBrace(source, cursor, '(', ')')
		if parametersEnd <= parametersStart {
			continue
		}
		arrow := topLevelArrow(source, parametersEnd)
		if arrow < 0 {
			continue
		}
		bodyStart := arrow + 2
		for bodyStart < len(source) && (source[bodyStart] == ' ' || source[bodyStart] == '\t' || source[bodyStart] == '\r' || source[bodyStart] == '\n') {
			bodyStart++
		}
		if bodyStart >= len(source) || source[bodyStart] != '{' {
			continue
		}
		bodyEnd := matchingBrace(source, bodyStart, '{', '}')
		if bodyEnd <= bodyStart {
			continue
		}
		name := string(source[match[2]:match[3]])
		symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: match[2], EndByte: match[3]})
		if err != nil {
			if errors.Is(err, typefacts.ErrNotFound) {
				continue
			}
			return nil, fmt.Errorf("resolve arrow function %s in %s: %w", name, path, err)
		}
		symbol = canonicalSymbol(ctx, facts, symbol)
		parameterSpans := splitArguments(source, parametersStart, parametersEnd-1)
		parameters, err := functionParameters(ctx, facts, path, source, parametersStart, parametersEnd-1)
		if err != nil {
			return nil, fmt.Errorf("resolve parameters for arrow function %s in %s: %w", name, path, err)
		}
		functions = append(functions, sourceFunction{
			id: FunctionID(symbol), symbol: symbol, name: name,
			bodyStart: bodyStart, bodyEnd: bodyEnd,
			rendering:    name[0] >= 'A' && name[0] <= 'Z' && jsxPattern.Match(source[bodyStart:bodyEnd]),
			exported:     bytes.Contains(source[match[0]:match[2]], []byte("export")),
			async:        async,
			programIndex: programBase + len(functions), parameters: parameters, parameterSpans: parameterSpans,
			returnedClosures: returnedClosureSpans(source, bodyStart, bodyEnd),
		})
	}
	return functions, nil
}

func functionBodyStart(source []byte, start int) int {
	paren, bracket, angle := 0, 0, 0
	for index := start; index < len(source); index++ {
		switch source[index] {
		case '(':
			paren++
		case ')':
			if paren > 0 {
				paren--
			}
		case '[':
			bracket++
		case ']':
			if bracket > 0 {
				bracket--
			}
		case '<':
			angle++
		case '>':
			if angle > 0 {
				angle--
			}
		case '{':
			if paren == 0 && bracket == 0 && angle == 0 {
				return index
			}
		case ';':
			if paren == 0 && bracket == 0 && angle == 0 {
				return -1
			}
		}
	}
	return -1
}

func topLevelArrow(source []byte, start int) int {
	paren, bracket, brace, angle := 0, 0, 0, 0
	quote := byte(0)
	escaped := false
	for index := start; index+1 < len(source); index++ {
		character := source[index]
		if quote != 0 {
			if escaped {
				escaped = false
			} else if character == '\\' {
				escaped = true
			} else if character == quote {
				quote = 0
			}
			continue
		}
		if character == '\'' || character == '"' || character == '`' {
			quote = character
			continue
		}
		switch character {
		case '=':
			if source[index+1] == '>' && paren == 0 && bracket == 0 && brace == 0 && angle == 0 {
				return index
			}
		case '(':
			paren++
		case ')':
			if paren > 0 {
				paren--
			}
		case '[':
			bracket++
		case ']':
			if bracket > 0 {
				bracket--
			}
		case '{':
			brace++
		case '}':
			if brace > 0 {
				brace--
			}
		case '<':
			angle++
		case '>':
			if angle > 0 {
				angle--
			}
		case ';':
			if paren == 0 && bracket == 0 && brace == 0 && angle == 0 {
				return -1
			}
		}
	}
	return -1
}

func returnedClosureSpans(source []byte, bodyStart, bodyEnd int) []byteSpan {
	spans := make([]byteSpan, 0)
	for _, match := range returnedArrowPattern.FindAllIndex(source[bodyStart:bodyEnd], -1) {
		start := bodyStart + match[0]
		valueStart := bodyStart + match[1]
		for valueStart < bodyEnd && (source[valueStart] == ' ' || source[valueStart] == '\t' || source[valueStart] == '\r' || source[valueStart] == '\n') {
			valueStart++
		}
		end := valueStart
		if valueStart < bodyEnd && source[valueStart] == '{' {
			end = matchingBrace(source, valueStart, '{', '}')
		} else {
			for end < bodyEnd && source[end] != ';' && source[end] != '\n' {
				end++
			}
		}
		spans = append(spans, byteSpan{start: start, end: end})
	}
	return spans
}

func containsOffset(spans []byteSpan, offset int) bool {
	for _, span := range spans {
		if span.start <= offset && offset < span.end {
			return true
		}
	}
	return false
}

func functionParameters(ctx context.Context, facts typefacts.Project, path string, source []byte, start, end int) ([]typefacts.SymbolID, error) {
	spans := splitArguments(source, start, end)
	parameters := make([]typefacts.SymbolID, 0, len(spans))
	for _, span := range spans {
		match := parameterPattern.FindSubmatchIndex(source[span.start:span.end])
		if match == nil {
			parameters = append(parameters, "")
			continue
		}
		nameStart := span.start + match[2]
		nameEnd := span.start + match[3]
		symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: nameStart, EndByte: nameEnd})
		if err != nil {
			return nil, err
		}
		parameters = append(parameters, symbol)
	}
	return parameters, nil
}

type byteSpan struct{ start, end int }

func splitArguments(source []byte, start, end int) []byteSpan {
	spans := make([]byteSpan, 0)
	itemStart := start
	paren, bracket, brace, angle := 0, 0, 0, 0
	quote := byte(0)
	escaped, lineComment, blockComment := false, false, false
	for index := start; index < end; index++ {
		character := source[index]
		if lineComment {
			if character == '\n' {
				lineComment = false
			}
			continue
		}
		if blockComment {
			if character == '*' && index+1 < end && source[index+1] == '/' {
				blockComment = false
				index++
			}
			continue
		}
		if quote != 0 {
			if escaped {
				escaped = false
			} else if character == '\\' {
				escaped = true
			} else if character == quote {
				quote = 0
			}
			continue
		}
		if character == '/' && index+1 < end {
			if source[index+1] == '/' {
				lineComment = true
				index++
				continue
			}
			if source[index+1] == '*' {
				blockComment = true
				index++
				continue
			}
		}
		if character == '\'' || character == '"' || character == '`' {
			quote = character
			continue
		}
		switch character {
		case '(':
			paren++
		case ')':
			paren--
		case '[':
			bracket++
		case ']':
			bracket--
		case '{':
			brace++
		case '}':
			brace--
		case '<':
			angle++
		case '>':
			if angle > 0 {
				angle--
			}
		case ',':
			if paren == 0 && bracket == 0 && brace == 0 && angle == 0 {
				trimmedStart, trimmedEnd := trimByteSpan(source, itemStart, index)
				if trimmedStart < trimmedEnd {
					spans = append(spans, byteSpan{start: itemStart, end: index})
				}
				itemStart = index + 1
			}
		}
	}
	trimmedStart, trimmedEnd := trimByteSpan(source, itemStart, end)
	if trimmedStart < trimmedEnd {
		spans = append(spans, byteSpan{start: itemStart, end: end})
	}
	return spans
}

func functionContext(functions []sourceFunction, offset int) (sourceFunction, bool) {
	for _, function := range functions {
		if function.bodyStart < offset && offset < function.bodyEnd {
			return function, true
		}
	}
	return sourceFunction{}, false
}

type callableTarget struct {
	id     FunctionID
	symbol typefacts.SymbolID
	name   string
}

func declaredCallableTargets(functions map[string][]sourceFunction) []callableTarget {
	targets := make([]callableTarget, 0)
	for _, declared := range functions {
		for _, function := range declared {
			targets = append(targets, callableTarget{id: function.id, symbol: function.symbol, name: function.name})
		}
	}
	return targets
}

func addFactoryInstances(
	ctx context.Context,
	facts typefacts.Project,
	sourceFiles []typefacts.SourceFile,
	sources map[string][]byte,
	functions map[string][]sourceFunction,
	executionMaps map[string]compilerfacts.ExecutionMap,
	program *Program,
) ([]callableTarget, error) {
	computationRegions := make(map[string][]allowedWriteRegion, len(sourceFiles))
	for _, file := range sourceFiles {
		path := filepath.Clean(file.Path)
		regions, err := readExecutionRegions(ctx, facts, path, file.Source)
		if err != nil {
			return nil, err
		}
		computationRegions[path] = regions
	}
	returned := make(map[FunctionID][]ReactiveRead, len(program.Functions))
	for _, function := range program.Functions {
		if len(function.ReturnedReads) != 0 {
			returned[function.ID] = function.ReturnedReads
		}
	}
	targets := make([]callableTarget, 0)
	for _, file := range sourceFiles {
		path := filepath.Clean(file.Path)
		type factoryBinding struct {
			name   typefacts.Location
			target typefacts.SymbolID
		}
		bindings := make([]factoryBinding, 0)
		if discoverer, ok := facts.(typefacts.BindingDiscoverer); ok {
			discovered, err := sourceBindingFacts(ctx, discoverer, path)
			if err != nil {
				return nil, err
			}
			for _, binding := range discovered {
				if !binding.Array && len(binding.Names) != 0 && binding.Names[0].Path != "" {
					bindings = append(bindings, factoryBinding{name: binding.Names[0], target: binding.Initializer.Target})
				}
			}
		} else {
			for _, match := range factoryBindingPattern.FindAllSubmatchIndex(file.Source, -1) {
				call, err := facts.ResolvedCall(ctx, typefacts.Location{Path: path, StartByte: match[4], EndByte: match[5]})
				if err == nil {
					bindings = append(bindings, factoryBinding{name: typefacts.Location{Path: path, StartByte: match[2], EndByte: match[3]}, target: call.Target})
				}
			}
		}
		for _, binding := range bindings {
			reads := returned[FunctionID(binding.target)]
			if len(reads) == 0 {
				continue
			}
			name := string(file.Source[binding.name.StartByte:binding.name.EndByte])
			symbol, err := facts.SymbolAt(ctx, binding.name)
			if err != nil {
				return nil, fmt.Errorf("resolve factory result %s in %s: %w", name, path, err)
			}
			symbol = canonicalSymbol(ctx, facts, symbol)
			id := FunctionID(symbol)
			if reads[0].Kind == ReactiveStorePath {
				references, err := facts.References(ctx, symbol)
				if err != nil {
					return nil, fmt.Errorf("find returned store references to %s in %s: %w", name, path, err)
				}
				for _, reference := range references {
					reference.Path = filepath.Clean(reference.Path)
					source, ok := sources[reference.Path]
					if !ok {
						continue
					}
					readEnd, ok := propertyAccessEnd(source, reference.EndByte)
					if !ok {
						continue
					}
					function, ok := functionContext(functions[reference.Path], reference.StartByte)
					if !ok {
						continue
					}
					execution := executionRoleWithComputations(executionMaps[reference.Path], computationRegions[reference.Path], compilerfacts.Span{
						Start: reference.StartByte, End: readEnd,
					})
					if !function.rendering && execution == ExecutionUntrackedRendering {
						execution = ExecutionInline
					}
					read := reads[0]
					read.Accessor = string(source[reference.StartByte:readEnd])
					read.Location = sourceLocation(typefacts.Location{
						Path: reference.Path, StartByte: reference.StartByte, EndByte: readEnd,
					}, source)
					read.Execution = execution
					read.Context = function.name
					if function.rendering {
						program.Reads = append(program.Reads, read)
					} else {
						program.Functions[function.programIndex].Reads = append(program.Functions[function.programIndex].Reads, read)
					}
				}
				continue
			}
			program.Functions = append(program.Functions, Function{
				ID: id, Name: name, Reads: append([]ReactiveRead{}, reads...),
				ReturnedReads: []ReactiveRead{}, Calls: []FunctionCall{}, CallbackInvocations: []CallbackInvocation{},
			})
			targets = append(targets, callableTarget{id: id, symbol: symbol, name: name})
		}
	}
	return targets, nil
}

func propagateReturnedFactoryCalls(
	ctx context.Context,
	facts typefacts.Project,
	sources map[string][]byte,
	functions map[string][]sourceFunction,
	targets []callableTarget,
	program *Program,
) error {
	programIndex := make(map[FunctionID]int, len(program.Functions))
	for index, function := range program.Functions {
		programIndex[function.ID] = index
	}
	for changed := true; changed; {
		changed = false
		for _, target := range targets {
			targetIndex, ok := programIndex[target.id]
			if !ok || len(program.Functions[targetIndex].ReturnedReads) == 0 && !hasReturnedPrimitive(program.Functions[targetIndex]) {
				continue
			}
			references, err := facts.References(ctx, target.symbol)
			if err != nil {
				return fmt.Errorf("find returned factory references to %s: %w", target.name, err)
			}
			for _, reference := range references {
				reference.Path = filepath.Clean(reference.Path)
				source, ok := sources[reference.Path]
				if !ok {
					continue
				}
				callEnd, ok := accessorCallEnd(source, reference.EndByte)
				if !ok || !hasReturnPrefix(source, reference.StartByte) || !hasReturnSuffix(source, callEnd) {
					continue
				}
				caller, ok := functionContext(functions[reference.Path], reference.StartByte)
				if !ok || caller.programIndex == targetIndex {
					continue
				}
				for _, read := range program.Functions[targetIndex].ReturnedReads {
					if appendUniqueReturnedRead(&program.Functions[caller.programIndex], read) {
						changed = true
					}
				}
				for _, creation := range program.Functions[targetIndex].PrimitiveCreations {
					if !creation.InReturnedClosure {
						continue
					}
					if appendUniqueReturnedPrimitive(&program.Functions[caller.programIndex], creation) {
						changed = true
					}
				}
			}
		}
	}
	return nil
}

func hasReturnedPrimitive(function Function) bool {
	for _, creation := range function.PrimitiveCreations {
		if creation.InReturnedClosure {
			return true
		}
	}
	return false
}

func appendUniqueReturnedPrimitive(function *Function, candidate PrimitiveCreation) bool {
	for _, existing := range function.PrimitiveCreations {
		if existing.InReturnedClosure && existing.Primitive == candidate.Primitive &&
			existing.Location.Path == candidate.Location.Path && existing.Location.StartByte == candidate.Location.StartByte {
			return false
		}
	}
	candidate.InReturnedClosure = true
	function.PrimitiveCreations = append(function.PrimitiveCreations, candidate)
	return true
}

func appendUniqueReturnedRead(function *Function, candidate ReactiveRead) bool {
	for _, existing := range function.ReturnedReads {
		if existing.Kind == candidate.Kind && existing.Accessor == candidate.Accessor &&
			existing.Declaration.Path == candidate.Declaration.Path &&
			existing.Declaration.StartByte == candidate.Declaration.StartByte {
			return false
		}
	}
	function.ReturnedReads = append(function.ReturnedReads, candidate)
	return true
}

func addFunctionCalls(
	ctx context.Context,
	facts typefacts.Project,
	sources map[string][]byte,
	functions map[string][]sourceFunction,
	executionMaps map[string]compilerfacts.ExecutionMap,
	targets []callableTarget,
	program *Program,
) error {
	ownerRegions := make(map[string][]compilerfacts.Span, len(sources))
	imperativeRegions := make(map[string][]allowedWriteRegion, len(sources))
	computationRegions := make(map[string][]allowedWriteRegion, len(sources))
	for path, source := range sources {
		regions, err := ownerProvidingRegions(ctx, facts, path, source)
		if err != nil {
			return err
		}
		ownerRegions[path] = regions
		imperative, err := allowedWriteRegions(ctx, facts, path, source)
		if err != nil {
			return err
		}
		imperativeRegions[path] = imperative
		computationRegions[path], err = readExecutionRegions(ctx, facts, path, source)
		if err != nil {
			return err
		}
	}
	functionIDs := make(map[FunctionID]struct{}, len(program.Functions))
	for _, function := range program.Functions {
		functionIDs[function.ID] = struct{}{}
	}
	if err := addOwnedCallbackEntries(ctx, facts, sources, functionIDs, program); err != nil {
		return err
	}
	for path, executionMap := range executionMaps {
		source := sources[path]
		for _, callback := range executionMap.CallbackRoles {
			if callback.Role != compilerfacts.CallbackEventHandler && callback.Role != compilerfacts.CallbackDirectiveApply {
				continue
			}
			start, end := trimByteSpan(source, callback.Span.Start, callback.Span.End)
			if start < 0 || end > len(source) || !identifierPattern.Match(source[start:end]) {
				continue
			}
			symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: start, EndByte: end})
			if err != nil {
				continue
			}
			id := FunctionID(canonicalSymbol(ctx, facts, symbol))
			if _, known := functionIDs[id]; !known {
				continue
			}
			caller, inFunction := functionContext(functions[path], start)
			call := FunctionCall{
				Target: id, TargetName: string(source[start:end]), Unowned: true,
				Execution: executionRole(executionMap, callback.Span),
				Location:  sourceLocation(typefacts.Location{Path: path, StartByte: start, EndByte: end}, source),
			}
			if callback.Role == compilerfacts.CallbackEventHandler {
				call.Execution = ExecutionEventCallback
			} else {
				call.Execution = ExecutionDirectiveApply
			}
			if inFunction {
				call.Context = caller.name
				program.Functions[caller.programIndex].Calls = append(program.Functions[caller.programIndex].Calls, call)
			} else {
				call.Context = "module callback"
				program.ModuleCalls = append(program.ModuleCalls, call)
			}
		}
	}
	for path, declared := range functions {
		for _, function := range declared {
			for parameter, symbol := range function.parameters {
				if symbol == "" {
					continue
				}
				references, err := facts.References(ctx, symbol)
				if err != nil {
					return fmt.Errorf("find references to parameter %d of %s: %w", parameter, function.name, err)
				}
				for _, reference := range references {
					reference.Path = filepath.Clean(reference.Path)
					if reference.Path != path || reference.StartByte <= function.bodyStart || reference.StartByte >= function.bodyEnd {
						continue
					}
					source := sources[reference.Path]
					callEnd, ok := accessorCallEnd(source, reference.EndByte)
					if !ok {
						continue
					}
					span := compilerfacts.Span{Start: reference.StartByte, End: callEnd}
					execution := executionRoleWithComputations(executionMaps[reference.Path], computationRegions[reference.Path], span)
					if !function.rendering && execution == ExecutionUntrackedRendering {
						execution = ExecutionInline
					}
					program.Functions[function.programIndex].CallbackInvocations = append(
						program.Functions[function.programIndex].CallbackInvocations,
						CallbackInvocation{
							Parameter: parameter,
							Location: sourceLocation(typefacts.Location{
								Path: reference.Path, StartByte: reference.StartByte, EndByte: callEnd,
							}, source),
							Execution: execution,
							Context:   function.name,
						},
					)
				}
			}
		}
	}
	for _, target := range targets {
		references, err := facts.References(ctx, target.symbol)
		if err != nil {
			return fmt.Errorf("find references to function %s: %w", target.name, err)
		}
		for _, reference := range references {
			reference.Path = filepath.Clean(reference.Path)
			source, ok := sources[reference.Path]
			if !ok {
				continue
			}
			callEnd, ok := accessorCallEnd(source, reference.EndByte)
			if !ok {
				continue
			}
			span := compilerfacts.Span{Start: reference.StartByte, End: callEnd}
			execution := executionRoleWithComputations(executionMaps[reference.Path], computationRegions[reference.Path], span)
			caller, inFunction := functionContext(functions[reference.Path], reference.StartByte)
			if (!inFunction || !caller.rendering) && execution == ExecutionUntrackedRendering {
				execution = ExecutionInline
			}
			functionCall := FunctionCall{
				Target:     target.id,
				TargetName: target.name,
				Arguments:  callArgumentFunctions(ctx, facts, reference.Path, source, reference.EndByte, callEnd, functionIDs),
				Location: sourceLocation(typefacts.Location{
					Path: reference.Path, StartByte: reference.StartByte, EndByte: callEnd,
				}, source),
				Execution: execution,
				Unowned:   unownedExecution(executionMaps[reference.Path], imperativeRegions[reference.Path], span),
			}
			point := compilerfacts.Span{Start: reference.StartByte, End: reference.StartByte}
			for _, region := range ownerRegions[reference.Path] {
				if contains(region, point) {
					functionCall.Owned = true
					break
				}
			}
			if inFunction {
				functionCall.Context = caller.name
				program.Functions[caller.programIndex].Calls = append(program.Functions[caller.programIndex].Calls, functionCall)
			} else {
				functionCall.Context = "module scope"
				program.ModuleCalls = append(program.ModuleCalls, functionCall)
			}
		}
	}
	return nil
}

func addOwnedCallbackEntries(ctx context.Context, facts typefacts.Project, sources map[string][]byte, functionIDs map[FunctionID]struct{}, program *Program) error {
	roles := []struct {
		name     string
		argument int
	}{
		{"createRoot", 0}, {"runWithOwner", 1}, {"createMemo", 0},
		{"createEffect", 0}, {"createRenderEffect", 0}, {"createProjection", 0},
		{"createSignal", 0}, {"createStore", 0},
	}
	for path, source := range sources {
		for _, role := range roles {
			regions, err := solidCallArgumentRegions(ctx, facts, path, source, role.name, role.argument)
			if err != nil {
				return err
			}
			for _, region := range regions {
				start, end := region.Start, region.End
				for start < end && (source[start] == ' ' || source[start] == '\t' || source[start] == '\r' || source[start] == '\n') {
					start++
				}
				for end > start && (source[end-1] == ' ' || source[end-1] == '\t' || source[end-1] == '\r' || source[end-1] == '\n') {
					end--
				}
				if !identifierPattern.Match(source[start:end]) {
					continue
				}
				symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: start, EndByte: end})
				if err != nil {
					continue
				}
				id := FunctionID(canonicalSymbol(ctx, facts, symbol))
				if _, known := functionIDs[id]; !known {
					continue
				}
				program.ModuleCalls = append(program.ModuleCalls, FunctionCall{
					Target: id, TargetName: string(source[start:end]), Owned: true,
					Location: sourceLocation(typefacts.Location{Path: path, StartByte: start, EndByte: end}, source),
					Context:  role.name + " callback",
				})
			}
		}
	}
	return nil
}

func callArgumentFunctions(
	ctx context.Context,
	facts typefacts.Project,
	path string,
	source []byte,
	identifierEnd, callEnd int,
	functionIDs map[FunctionID]struct{},
) []FunctionID {
	open := identifierEnd
	for open < callEnd && (source[open] == ' ' || source[open] == '\t' || source[open] == '\r' || source[open] == '\n') {
		open++
	}
	arguments := splitArguments(source, open+1, callEnd-1)
	result := make([]FunctionID, len(arguments))
	for index, argument := range arguments {
		start, end := argument.start, argument.end
		for start < end && (source[start] == ' ' || source[start] == '\t' || source[start] == '\r' || source[start] == '\n') {
			start++
		}
		for end > start && (source[end-1] == ' ' || source[end-1] == '\t' || source[end-1] == '\r' || source[end-1] == '\n') {
			end--
		}
		if !identifierPattern.Match(source[start:end]) {
			continue
		}
		symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: start, EndByte: end})
		if err != nil {
			continue
		}
		id := FunctionID(canonicalSymbol(ctx, facts, symbol))
		if _, ok := functionIDs[id]; ok {
			result[index] = id
		}
	}
	return result
}

func canonicalSymbol(ctx context.Context, facts typefacts.Project, symbol typefacts.SymbolID) typefacts.SymbolID {
	if resolved, err := facts.ResolveAlias(ctx, symbol); err == nil {
		return resolved
	}
	return symbol
}

func matchingBrace(source []byte, start int, open, close byte) int {
	depth := 0
	quote := byte(0)
	escaped := false
	lineComment := false
	blockComment := false
	for index := start; index < len(source); index++ {
		character := source[index]
		if lineComment {
			if character == '\n' {
				lineComment = false
			}
			continue
		}
		if blockComment {
			if character == '*' && index+1 < len(source) && source[index+1] == '/' {
				blockComment = false
				index++
			}
			continue
		}
		if quote != 0 {
			if escaped {
				escaped = false
			} else if character == '\\' {
				escaped = true
			} else if character == quote {
				quote = 0
			}
			continue
		}
		if character == '/' && index+1 < len(source) {
			if source[index+1] == '/' {
				lineComment = true
				index++
				continue
			}
			if source[index+1] == '*' {
				blockComment = true
				index++
				continue
			}
		}
		if character == '\'' || character == '"' || character == '`' {
			quote = character
			continue
		}
		switch character {
		case open:
			depth++
		case close:
			if close == '>' && index > start && source[index-1] == '=' {
				continue
			}
			depth--
			if depth == 0 {
				return index + 1
			}
		}
	}
	return 0
}

func sourceLocation(location typefacts.Location, source []byte) certification.SourceLocation {
	line, column := 1, 1
	if location.StartByte >= 0 && location.StartByte <= len(source) {
		prefix := source[:location.StartByte]
		lastNewline := -1
		for index, character := range prefix {
			if character == '\n' {
				line++
				lastNewline = index
			}
		}
		lineBytes := prefix[lastNewline+1:]
		units := 0
		for len(lineBytes) > 0 {
			r, size := utf8.DecodeRune(lineBytes)
			units += len(utf16.Encode([]rune{r}))
			lineBytes = lineBytes[size:]
		}
		column = units + 1
	}
	return certification.SourceLocation{
		Path: location.Path, StartByte: location.StartByte, EndByte: location.EndByte,
		Line: line, Column: column,
	}
}
