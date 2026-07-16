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
	signalBindingPattern   = regexp.MustCompile(`(?m)(?:export\s+)?const\s*\[\s*([A-Za-z_$][A-Za-z0-9_$]*)[^]]*\]\s*=\s*(createSignal)\s*(?:<[^>\n]+>)?\s*\(`)
	storeBindingPattern    = regexp.MustCompile(`(?m)(?:export\s+)?const\s*\[\s*([A-Za-z_$][A-Za-z0-9_$]*)[^]]*\]\s*=\s*(createStore)\s*(?:<[^>\n]+>)?\s*\(`)
	functionPattern        = regexp.MustCompile(`(?m)(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*`)
	arrowFunctionPattern   = regexp.MustCompile(`(?m)(?:export\s+)?const\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*`)
	exportConstPattern     = regexp.MustCompile(`(?m)export\s+const\s+`)
	exportClassPattern     = regexp.MustCompile(`(?m)export\s+class\s+([A-Za-z_$][A-Za-z0-9_$]*)`)
	exportListPattern      = regexp.MustCompile(`(?m)export\s*\{([^}]*)\}`)
	parameterPattern       = regexp.MustCompile(`^\s*([A-Za-z_$][A-Za-z0-9_$]*)`)
	identifierPattern      = regexp.MustCompile(`^[A-Za-z_$][A-Za-z0-9_$]*$`)
	returnedArrowPattern   = regexp.MustCompile(`return\s*\(\s*\)\s*=>`)
	factoryBindingPattern  = regexp.MustCompile(`(?m)(?:export\s+)?const\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\(`)
	namedImportPattern     = regexp.MustCompile(`(?m)import\s*\{([^}]*)\}\s*from\s*["']([^"']+)["']`)
	importSpecifierPattern = regexp.MustCompile(`^\s*([A-Za-z_$][A-Za-z0-9_$]*)(?:\s+as\s+([A-Za-z_$][A-Za-z0-9_$]*))?\s*$`)
	jsxPattern             = regexp.MustCompile(`<[A-Za-z]`)
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
	sources := make(map[string][]byte, len(sourceFiles))
	functions := make(map[string][]sourceFunction, len(sourceFiles))
	program := Program{Reads: []ReactiveRead{}, Functions: []Function{}, ExportAliases: map[string]FunctionID{}}
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
				Reads: []ReactiveRead{}, ReturnedReads: []ReactiveRead{}, Calls: []FunctionCall{}, CallbackInvocations: []CallbackInvocation{},
			})
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
				execution := executionRole(executionMaps[reference.Path], readSpan)
				if !function.rendering && execution == ExecutionUntrackedRendering {
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
				if containsOffset(function.returnedClosures, reference.StartByte) {
					program.Functions[function.programIndex].ReturnedReads = append(program.Functions[function.programIndex].ReturnedReads, read)
				} else if function.rendering {
					program.Reads = append(program.Reads, read)
				} else {
					program.Functions[function.programIndex].Reads = append(program.Functions[function.programIndex].Reads, read)
				}
			}
		}
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
				execution := executionRole(executionMaps[reference.Path], readSpan)
				if !function.rendering && execution == ExecutionUntrackedRendering {
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
				if function.rendering {
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
	declarations, err := facts.Declarations(ctx, target)
	if err != nil {
		return false
	}
	for _, declaration := range declarations {
		path := strings.ToLower(filepath.ToSlash(declaration.Location.Path))
		if declaration.Name == name && strings.Contains(path, "solid-js") {
			return true
		}
	}
	return false
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

func executionRole(executionMap compilerfacts.ExecutionMap, read compilerfacts.Span) ExecutionRole {
	for _, region := range executionMap.TrackedRegions {
		if contains(region.Span, read) {
			return ExecutionTrackedJSX
		}
	}
	for _, callback := range executionMap.CallbackRoles {
		if contains(callback.Span, read) {
			return ExecutionDeferredCallback
		}
	}
	return ExecutionUntrackedRendering
}

func contains(outer, inner compilerfacts.Span) bool {
	return outer.Start <= inner.Start && inner.End <= outer.End
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
	returnedClosures []byteSpan
}

func declaredFunctions(ctx context.Context, facts typefacts.Project, path string, source []byte, programBase int) ([]sourceFunction, error) {
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
			programIndex: programBase + len(functions), parameters: parameters,
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
		start := bodyStart + match[1]
		end := start
		for end < bodyEnd && source[end] != ';' && source[end] != '\n' {
			end++
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
	for index := start; index < end; index++ {
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
		case ',':
			if paren == 0 && bracket == 0 && brace == 0 && angle == 0 {
				spans = append(spans, byteSpan{start: itemStart, end: index})
				itemStart = index + 1
			}
		}
	}
	if itemStart < end {
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
	returned := make(map[FunctionID][]ReactiveRead, len(program.Functions))
	for _, function := range program.Functions {
		if len(function.ReturnedReads) != 0 {
			returned[function.ID] = function.ReturnedReads
		}
	}
	targets := make([]callableTarget, 0)
	for _, file := range sourceFiles {
		path := filepath.Clean(file.Path)
		for _, match := range factoryBindingPattern.FindAllSubmatchIndex(file.Source, -1) {
			call, err := facts.ResolvedCall(ctx, typefacts.Location{Path: path, StartByte: match[4], EndByte: match[5]})
			if err != nil {
				continue
			}
			reads := returned[FunctionID(call.Target)]
			if len(reads) == 0 {
				continue
			}
			name := string(file.Source[match[2]:match[3]])
			symbol, err := facts.SymbolAt(ctx, typefacts.Location{Path: path, StartByte: match[2], EndByte: match[3]})
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
					execution := executionRole(executionMaps[reference.Path], compilerfacts.Span{
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
			if !ok || len(program.Functions[targetIndex].ReturnedReads) == 0 {
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
			}
		}
	}
	return nil
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
	functionIDs := make(map[FunctionID]struct{}, len(program.Functions))
	for _, function := range program.Functions {
		functionIDs[function.ID] = struct{}{}
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
					execution := executionRole(executionMaps[reference.Path], span)
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
			caller, ok := functionContext(functions[reference.Path], reference.StartByte)
			if !ok {
				continue
			}
			span := compilerfacts.Span{Start: reference.StartByte, End: callEnd}
			execution := executionRole(executionMaps[reference.Path], span)
			if !caller.rendering && execution == ExecutionUntrackedRendering {
				execution = ExecutionInline
			}
			program.Functions[caller.programIndex].Calls = append(program.Functions[caller.programIndex].Calls, FunctionCall{
				Target:     target.id,
				TargetName: target.name,
				Arguments:  callArgumentFunctions(ctx, facts, reference.Path, source, reference.EndByte, callEnd, functionIDs),
				Location: sourceLocation(typefacts.Location{
					Path: reference.Path, StartByte: reference.StartByte, EndByte: callEnd,
				}, source),
				Execution: execution,
				Context:   caller.name,
			})
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
