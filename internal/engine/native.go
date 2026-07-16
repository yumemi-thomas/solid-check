package engine

import (
	"context"
	"errors"
	"os"
	"path/filepath"
	"regexp"
	"slices"
	"strings"
	"sync"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/packagecontracts"
	"github.com/yumemi-thomas/solid-check/internal/reactiveir"
	"github.com/yumemi-thomas/solid-check/internal/solver"
	"github.com/yumemi-thomas/solid-check/internal/typefacts"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
	"github.com/yumemi-thomas/solid-check/pkg/contracts"
)

// NativeEngine opens sessions backed by the real native Type Facts module.
// Compiler execution facts and solver rules remain separate dependencies.
type NativeEngine struct {
	OpenTypeFacts     typefacts.OpenProjectFunc
	OpenCompilerFacts compilerfacts.OpenFunc
}

var moduleImportPattern = regexp.MustCompile(`(?m)(?:from\s*|import\s*)["']([^"']+)["']`)

func (e NativeEngine) OpenProject(ctx context.Context, config ProjectConfig) (ProjectSession, error) {
	if e.OpenTypeFacts == nil {
		return nil, errors.New("open Type Facts adapter is required")
	}
	if config.ConfigPath == "" {
		config.ConfigPath = "tsconfig.json"
	}
	facts, err := e.OpenTypeFacts(ctx, config.ConfigPath)
	if err != nil {
		return nil, err
	}
	var compiler compilerfacts.Analyzer
	if e.OpenCompilerFacts != nil {
		compiler, err = e.OpenCompilerFacts(ctx)
		if err != nil {
			_ = facts.Close()
			return nil, err
		}
	}
	executionMaps := make(map[string]compilerfacts.ExecutionMap)
	sources, err := facts.SourceFiles(ctx)
	if err != nil {
		if compiler != nil {
			_ = compiler.Close()
		}
		_ = facts.Close()
		return nil, err
	}
	sources, err = sourcesWithinProject(sources, config.ConfigPath)
	if err != nil {
		if compiler != nil {
			_ = compiler.Close()
		}
		_ = facts.Close()
		return nil, err
	}
	projectSources := cloneSourceFiles(sources)
	packageContracts := make([]contracts.Contract, 0, len(config.ContractPaths)+2)
	bundled, err := contracts.Bundled()
	if err != nil {
		if compiler != nil {
			_ = compiler.Close()
		}
		_ = facts.Close()
		return nil, err
	}
	for _, contract := range bundled {
		if importsPackage(sources, contract.Package.Name) {
			packageContracts = append(packageContracts, contract)
		}
	}
	discovered, err := discoverPackageContracts(sources, config.ConfigPath)
	if err != nil {
		if compiler != nil {
			_ = compiler.Close()
		}
		_ = facts.Close()
		return nil, err
	}
	for _, contract := range discovered {
		packageContracts = replacePackageContract(packageContracts, contract)
	}
	for _, path := range config.ContractPaths {
		contract, loadErr := contracts.LoadFile(path)
		if loadErr != nil {
			if compiler != nil {
				_ = compiler.Close()
			}
			_ = facts.Close()
			return nil, loadErr
		}
		packageContracts = replacePackageContract(packageContracts, contract)
	}
	if compiler != nil {
		for _, source := range sources {
			if !isJSXPath(source.Path) {
				continue
			}
			request := newCompilerRequest(source.Path, source.Source)
			executionMap, analysisErr := compiler.Analyze(ctx, request)
			if analysisErr != nil {
				_ = compiler.Close()
				_ = facts.Close()
				return nil, analysisErr
			}
			if validationErr := compilerfacts.Validate(request, executionMap); validationErr != nil {
				_ = compiler.Close()
				_ = facts.Close()
				return nil, validationErr
			}
			executionMaps[filepath.Clean(source.Path)] = executionMap
		}
	}
	return &nativeSession{
		facts:         facts,
		compiler:      compiler,
		executionMaps: executionMaps,
		sources:       projectSources,
		contracts:     packageContracts,
	}, nil
}

func discoverPackageContracts(sources []typefacts.SourceFile, configPath string) ([]contracts.Contract, error) {
	absoluteConfig, err := filepath.Abs(configPath)
	if err != nil {
		return nil, err
	}
	modules := make(map[string]struct{})
	for _, source := range sources {
		for _, match := range moduleImportPattern.FindAllSubmatch(source.Source, -1) {
			module := string(match[1])
			if module == "" || strings.HasPrefix(module, ".") || strings.HasPrefix(module, "/") || strings.HasPrefix(module, "node:") {
				continue
			}
			modules[packageRoot(module)] = struct{}{}
		}
	}
	result := make([]contracts.Contract, 0)
	for module := range modules {
		for directory := filepath.Dir(absoluteConfig); ; directory = filepath.Dir(directory) {
			candidate := filepath.Join(directory, "node_modules", filepath.FromSlash(module), "solid-reactivity.json")
			if _, err := os.Stat(candidate); err == nil {
				contract, err := contracts.LoadFile(candidate)
				if err != nil {
					return nil, err
				}
				result = append(result, contract)
				break
			} else if !errors.Is(err, os.ErrNotExist) {
				return nil, err
			}
			parent := filepath.Dir(directory)
			if parent == directory {
				break
			}
		}
	}
	return result, nil
}

func packageRoot(module string) string {
	parts := strings.Split(module, "/")
	if strings.HasPrefix(module, "@") && len(parts) >= 2 {
		return parts[0] + "/" + parts[1]
	}
	return parts[0]
}

func sourcesWithinProject(sources []typefacts.SourceFile, configPath string) ([]typefacts.SourceFile, error) {
	absoluteConfig, err := filepath.Abs(configPath)
	if err != nil {
		return nil, err
	}
	root := filepath.Dir(absoluteConfig)
	filtered := make([]typefacts.SourceFile, 0, len(sources))
	for _, source := range sources {
		relative, err := filepath.Rel(root, source.Path)
		if err != nil {
			return nil, err
		}
		if relative == ".." || strings.HasPrefix(relative, ".."+string(filepath.Separator)) {
			continue
		}
		filtered = append(filtered, source)
	}
	return filtered, nil
}

func importsPackage(sources []typefacts.SourceFile, packageName string) bool {
	for _, source := range sources {
		for _, match := range moduleImportPattern.FindAllSubmatch(source.Source, -1) {
			if packageRoot(string(match[1])) == packageName {
				return true
			}
		}
	}
	return false
}

func replacePackageContract(existing []contracts.Contract, replacement contracts.Contract) []contracts.Contract {
	for index := range existing {
		if existing[index].Package.Name == replacement.Package.Name {
			existing[index] = replacement
			return existing
		}
	}
	return append(existing, replacement)
}

type nativeSession struct {
	mu            sync.RWMutex
	facts         typefacts.Project
	compiler      compilerfacts.Analyzer
	executionMaps map[string]compilerfacts.ExecutionMap
	sources       []typefacts.SourceFile
	contracts     []contracts.Contract
	version       uint64
	closed        bool
}

func (s *nativeSession) Update(ctx context.Context, changes []FileChange) (AnalysisDelta, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.closed {
		return AnalysisDelta{}, ErrSessionClosed
	}
	pendingMaps := make(map[string]compilerfacts.ExecutionMap)
	pendingSources := make(map[string][]byte)
	deletedJSX := make([]string, 0)
	deletedSources := make([]string, 0)
	for _, change := range changes {
		path, err := filepath.Abs(change.Path)
		if err != nil {
			return AnalysisDelta{}, err
		}
		path = filepath.Clean(path)
		if change.Deleted {
			deletedSources = append(deletedSources, path)
		} else {
			pendingSources[path] = append([]byte(nil), change.Source...)
		}
	}
	if s.compiler != nil {
		for _, change := range changes {
			if !isJSXPath(change.Path) {
				continue
			}
			path, err := filepath.Abs(change.Path)
			if err != nil {
				return AnalysisDelta{}, err
			}
			path = filepath.Clean(path)
			if change.Deleted {
				deletedJSX = append(deletedJSX, path)
				continue
			}
			request := newCompilerRequest(path, change.Source)
			executionMap, err := s.compiler.Analyze(ctx, request)
			if err != nil {
				return AnalysisDelta{}, err
			}
			if err := compilerfacts.Validate(request, executionMap); err != nil {
				return AnalysisDelta{}, err
			}
			pendingMaps[path] = executionMap
		}
	}
	factChanges := make([]typefacts.FileChange, len(changes))
	for i, change := range changes {
		factChanges[i] = typefacts.FileChange{
			Path:    change.Path,
			Version: change.Version,
			Source:  append([]byte(nil), change.Source...),
			Deleted: change.Deleted,
		}
	}
	affected, err := s.facts.Update(ctx, factChanges)
	if err != nil {
		return AnalysisDelta{}, err
	}
	for path, executionMap := range pendingMaps {
		s.executionMaps[path] = executionMap
	}
	for _, path := range deletedJSX {
		delete(s.executionMaps, path)
	}
	s.sources = updateSources(s.sources, pendingSources, deletedSources)
	if len(affected.Files) != 0 {
		s.version++
	}
	return AnalysisDelta{
		Version:       s.version,
		AffectedPaths: append([]string{}, affected.Files...),
	}, nil
}

func (s *nativeSession) Snapshot(ctx context.Context, _ *AnalysisScope) (certification.Snapshot, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	if s.closed {
		return certification.Snapshot{}, ErrSessionClosed
	}
	if s.compiler == nil || len(s.executionMaps) == 0 {
		return certification.NewSnapshot([]certification.Finding{{
			ID:       "SC0002",
			Rule:     "execution-map-unavailable",
			Kind:     certification.FindingUncertifiable,
			Severity: certification.SeverityError,
			Message:  "the Solid compiler execution-map backend is not connected",
			Evidence: []certification.EvidenceStep{{
				Message: "TypeScript project facts are available, but JSX execution regions are unresolved",
			}},
		}}, nil, certification.Metrics{
			ProofObligations:      1,
			UnresolvedObligations: 1,
		})
	}
	program, err := reactiveir.BuildWithContracts(ctx, s.facts, s.sources, s.executionMaps, s.contracts)
	if err != nil {
		return certification.Snapshot{}, err
	}
	result := solver.SolveStrictReads(program)
	packages := make([]certification.PackageSummary, 0, len(s.contracts))
	for _, contract := range s.contracts {
		packages = append(packages, certification.PackageSummary{
			Name: contract.Package.Name, Version: contract.Package.Version,
			ContractHash: contract.ContractHash, Evidence: contract.Evidence.Kind,
			ExportsAnalyzed: len(contract.Exports),
		})
	}
	return certification.NewSnapshot(result.Findings, packages, certification.Metrics{
		FilesAnalyzed:         len(s.executionMaps),
		FunctionsAnalyzed:     len(program.Functions),
		ProofObligations:      result.ProofObligations,
		UnresolvedObligations: result.UnresolvedObligations,
	})
}

func (s *nativeSession) EmitPackageContract(ctx context.Context, options PackageContractOptions) (contracts.Contract, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	if s.closed {
		return contracts.Contract{}, ErrSessionClosed
	}
	program, err := reactiveir.BuildWithContracts(ctx, s.facts, s.sources, s.executionMaps, s.contracts)
	if err != nil {
		return contracts.Contract{}, err
	}
	return packagecontracts.Emit(program, packagecontracts.EmitOptions{
		Package: options.Package, CompilerFactsProtocol: options.CompilerFactsProtocol,
		Artifacts: options.Artifacts,
	})
}

func (s *nativeSession) Close() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.closed {
		return ErrSessionClosed
	}
	s.closed = true
	var compilerErr error
	if s.compiler != nil {
		compilerErr = s.compiler.Close()
	}
	return errors.Join(s.facts.Close(), compilerErr)
}

func isJSXPath(path string) bool {
	extension := strings.ToLower(filepath.Ext(path))
	return extension == ".jsx" || extension == ".tsx"
}

func newCompilerRequest(path string, source []byte) compilerfacts.AnalysisRequest {
	return compilerfacts.NewRequest(path, source, compilerfacts.CompilerOptions{
		ModuleName: "dom",
		Generate:   "dom",
	})
}

func cloneSourceFiles(files []typefacts.SourceFile) []typefacts.SourceFile {
	cloned := make([]typefacts.SourceFile, len(files))
	for index, file := range files {
		cloned[index] = typefacts.SourceFile{
			Path:   filepath.Clean(file.Path),
			Source: append([]byte(nil), file.Source...),
		}
	}
	return cloned
}

func updateSources(files []typefacts.SourceFile, changed map[string][]byte, deleted []string) []typefacts.SourceFile {
	byPath := make(map[string][]byte, len(files)+len(changed))
	for _, file := range files {
		byPath[filepath.Clean(file.Path)] = append([]byte(nil), file.Source...)
	}
	for path, source := range changed {
		byPath[path] = append([]byte(nil), source...)
	}
	for _, path := range deleted {
		delete(byPath, path)
	}
	updated := make([]typefacts.SourceFile, 0, len(byPath))
	for path, source := range byPath {
		updated = append(updated, typefacts.SourceFile{Path: path, Source: source})
	}
	slices.SortFunc(updated, func(left, right typefacts.SourceFile) int {
		return strings.Compare(left.Path, right.Path)
	})
	return updated
}
