package main

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
	"github.com/yumemi-thomas/solid-check/pkg/contracts"
)

func TestCLIReachesExecutionMapObligationForValidProject(t *testing.T) {
	project := filepath.Join("..", "..", "internal", "typefacts", "testdata", "aliased-import", "tsconfig.json")
	var stdout, stderr bytes.Buffer
	exitCode := run(context.Background(), []string{
		"--project", project,
		"--format", "json",
	}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("exit code = %d, stderr = %s", exitCode, stderr.String())
	}
	var snapshot certification.Snapshot
	if err := json.Unmarshal(stdout.Bytes(), &snapshot); err != nil {
		t.Fatal(err)
	}
	if snapshot.Status != certification.StatusUncertifiable {
		t.Errorf("status = %q, want uncertifiable", snapshot.Status)
	}
	if len(snapshot.Findings) != 1 || snapshot.Findings[0].Rule != "execution-map-unavailable" {
		t.Errorf("findings = %#v", snapshot.Findings)
	}
}

func TestCLIEmitsPackageContract(t *testing.T) {
	helper := filepath.Join(t.TempDir(), "compiler-facts-helper")
	script := "#!/bin/sh\nexec \"$SOLID_CHECK_TEST_BINARY\" -test.run '^TestCLICompilerFactsHelperProcess$'\n"
	if err := os.WriteFile(helper, []byte(script), 0o700); err != nil {
		t.Fatal(err)
	}
	t.Setenv("SOLID_CHECK_TEST_BINARY", os.Args[0])
	t.Setenv("SOLID_COMPILER_FACTS_BIN", helper)

	outputDirectory := t.TempDir()
	output := filepath.Join(outputDirectory, "solid-reactivity.json")
	declaration := filepath.Join(outputDirectory, "index.d.ts")
	if err := os.WriteFile(declaration, []byte("export declare function readCount(): number;\n"), 0o600); err != nil {
		t.Fatal(err)
	}
	project := filepath.Join("..", "..", "internal", "reactiveir", "testdata", "interprocedural", "tsconfig.json")
	var stdout, stderr bytes.Buffer
	exitCode := run(context.Background(), []string{
		"--project", project,
		"--emit-contract", output,
		"--package-name", "reactive-package",
		"--package-version", "1.0.0",
		"--declaration-artifact", declaration,
	}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("exit code = %d, stderr = %s", exitCode, stderr.String())
	}
	contract, err := contracts.LoadFile(output)
	if err != nil {
		t.Fatal(err)
	}
	if len(contract.Exports["readCount"].ReactiveReads) != 1 {
		t.Fatalf("contract exports = %#v", contract.Exports)
	}
	if contract.Artifacts.Declaration == nil || contract.Artifacts.Declaration.Path != "index.d.ts" {
		t.Fatalf("declaration artifact = %#v", contract.Artifacts.Declaration)
	}

	consumer := filepath.Join("..", "..", "internal", "reactiveir", "testdata", "package-consumer")
	stdout.Reset()
	stderr.Reset()
	exitCode = run(context.Background(), []string{
		"--project", filepath.Join(consumer, "tsconfig.json"),
		"--contract", output,
		"--format", "json",
		"--certify",
	}, &stdout, &stderr)
	if exitCode != 1 {
		t.Fatalf("consumer exit code = %d, want violation; stderr = %s", exitCode, stderr.String())
	}
	var snapshot certification.Snapshot
	if err := json.Unmarshal(stdout.Bytes(), &snapshot); err != nil {
		t.Fatal(err)
	}
	if len(snapshot.Findings) != 1 || len(snapshot.PackageSummaries) != 1 {
		t.Fatalf("consumer snapshot = %#v", snapshot)
	}
}

func TestCLIConsumesPackageContract(t *testing.T) {
	helper := filepath.Join(t.TempDir(), "compiler-facts-helper")
	script := "#!/bin/sh\nexec \"$SOLID_CHECK_TEST_BINARY\" -test.run '^TestCLICompilerFactsHelperProcess$'\n"
	if err := os.WriteFile(helper, []byte(script), 0o700); err != nil {
		t.Fatal(err)
	}
	t.Setenv("SOLID_CHECK_TEST_BINARY", os.Args[0])
	t.Setenv("SOLID_COMPILER_FACTS_BIN", helper)

	fixture := filepath.Join("..", "..", "internal", "reactiveir", "testdata", "package-consumer")
	contractPath := filepath.Join(fixture, "node_modules", "reactive-package", "solid-reactivity.json")
	var stdout, stderr bytes.Buffer
	exitCode := run(context.Background(), []string{
		"--project", filepath.Join(fixture, "tsconfig.json"),
		"--contract", contractPath,
		"--format", "json",
		"--certify",
	}, &stdout, &stderr)
	if exitCode != 1 {
		t.Fatalf("exit code = %d, want violation; stderr = %s", exitCode, stderr.String())
	}
	var snapshot certification.Snapshot
	if err := json.Unmarshal(stdout.Bytes(), &snapshot); err != nil {
		t.Fatal(err)
	}
	if snapshot.Status != certification.StatusViolation || len(snapshot.PackageSummaries) != 1 {
		t.Fatalf("snapshot = %#v", snapshot)
	}
	if len(snapshot.Findings) != 1 || snapshot.Findings[0].Rule != "strict-read-untracked" {
		t.Fatalf("findings = %#v, want only the untracked package read", snapshot.Findings)
	}
}

func TestCLIValidatesPackageContractArtifactsWithoutOpeningProject(t *testing.T) {
	fixture := filepath.Join("..", "..", "internal", "reactiveir", "testdata", "package-consumer", "node_modules", "reactive-package")
	var stdout, stderr bytes.Buffer
	exitCode := run(context.Background(), []string{
		"--validate-contract", filepath.Join(fixture, "solid-reactivity.json"),
	}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("exit code = %d, stderr = %s", exitCode, stderr.String())
	}
}

func TestCLIEmitsAndConsumesReturnedAccessorContract(t *testing.T) {
	helper := filepath.Join(t.TempDir(), "compiler-facts-helper")
	script := "#!/bin/sh\nexec \"$SOLID_CHECK_TEST_BINARY\" -test.run '^TestCLICompilerFactsHelperProcess$'\n"
	if err := os.WriteFile(helper, []byte(script), 0o700); err != nil {
		t.Fatal(err)
	}
	t.Setenv("SOLID_CHECK_TEST_BINARY", os.Args[0])
	t.Setenv("SOLID_COMPILER_FACTS_BIN", helper)

	output := filepath.Join(t.TempDir(), "solid-reactivity.json")
	producer := filepath.Join("..", "..", "internal", "reactiveir", "testdata", "package-return-producer")
	var stdout, stderr bytes.Buffer
	exitCode := run(context.Background(), []string{
		"--project", filepath.Join(producer, "tsconfig.json"),
		"--emit-contract", output,
		"--package-name", "reactive-package",
	}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("producer exit code = %d, stderr = %s", exitCode, stderr.String())
	}
	contract, err := contracts.LoadFile(output)
	if err != nil {
		t.Fatal(err)
	}
	if contract.Exports["createCount"].Returns == nil {
		t.Fatalf("contract = %#v, want returned accessor", contract)
	}
	if contract.Exports["createAliasedCount"].Returns == nil {
		t.Fatalf("contract = %#v, want re-exported accessor factory", contract)
	}
	if contract.Exports["createArrowCount"].Returns == nil {
		t.Fatalf("contract = %#v, want exported arrow returned accessor", contract)
	}
	if contract.Exports["createMemoCount"].Returns == nil {
		t.Fatalf("contract = %#v, want accessor returned through createMemo", contract)
	}
	if returned := contract.Exports["createState"].Returns; returned == nil || returned.Kind != "store-path" {
		t.Fatalf("contract = %#v, want returned store", contract)
	}
	if _, ok := contract.Exports["identityFactory"]; !ok {
		t.Fatalf("contract = %#v, want multiline-return-type export", contract)
	}
	if summary := contract.Exports["packageVersion"]; summary.Kind != "value" {
		t.Fatalf("packageVersion = %#v, want inert value export", summary)
	}
	if _, ok := contract.Exports["nestedGeneric"]; !ok {
		t.Fatalf("contract = %#v, want nested-generic export", contract)
	}
	if _, ok := contract.Exports["callbackGeneric"]; !ok {
		t.Fatalf("contract = %#v, want callback-constrained generic export", contract)
	}
	if _, ok := contract.Exports["loadValue"]; !ok {
		t.Fatalf("contract = %#v, want async function export", contract)
	}
	encoded, err := json.Marshal(contract.Exports["loadValue"])
	if err != nil {
		t.Fatal(err)
	}
	var asyncSummary map[string]any
	if err := json.Unmarshal(encoded, &asyncSummary); err != nil {
		t.Fatal(err)
	}
	if asyncSummary["asyncBehavior"] != "promise" {
		t.Fatalf("loadValue = %#v, want promise async behavior", asyncSummary)
	}
	if contract.Exports["secondConstant"].Kind != "value" {
		t.Fatalf("contract = %#v, want secondary const export", contract)
	}

	consumer := filepath.Join("..", "..", "internal", "reactiveir", "testdata", "package-return-consumer")
	stdout.Reset()
	stderr.Reset()
	exitCode = run(context.Background(), []string{
		"--project", filepath.Join(consumer, "tsconfig.json"),
		"--contract", output,
		"--format", "json",
		"--certify",
	}, &stdout, &stderr)
	if exitCode != 1 {
		t.Fatalf("consumer exit code = %d, want violation; stderr = %s", exitCode, stderr.String())
	}
	var snapshot certification.Snapshot
	if err := json.Unmarshal(stdout.Bytes(), &snapshot); err != nil {
		t.Fatal(err)
	}
	if len(snapshot.Findings) != 1 {
		t.Fatalf("consumer findings = %#v, want one returned-accessor violation", snapshot.Findings)
	}
}

func TestCLICertifiesCorrectedTracerAndRejectsViolation(t *testing.T) {
	helper := filepath.Join(t.TempDir(), "compiler-facts-helper")
	script := "#!/bin/sh\nexec \"$SOLID_CHECK_TEST_BINARY\" -test.run '^TestCLICompilerFactsHelperProcess$'\n"
	if err := os.WriteFile(helper, []byte(script), 0o700); err != nil {
		t.Fatal(err)
	}
	t.Setenv("SOLID_CHECK_TEST_BINARY", os.Args[0])
	t.Setenv("SOLID_COMPILER_FACTS_BIN", helper)

	tests := []struct {
		name       string
		project    string
		wantExit   int
		wantStatus certification.Status
		wantRule   string
	}{
		{
			name:       "cross-file violation",
			project:    filepath.Join("..", "..", "internal", "reactiveir", "testdata", "tracer", "tsconfig.json"),
			wantExit:   1,
			wantStatus: certification.StatusViolation,
			wantRule:   "strict-read-untracked",
		},
		{
			name:       "corrected project",
			project:    filepath.Join("..", "..", "internal", "reactiveir", "testdata", "tracer-corrected", "tsconfig.json"),
			wantExit:   0,
			wantStatus: certification.StatusCertified,
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			var stdout, stderr bytes.Buffer
			exitCode := run(context.Background(), []string{
				"--project", test.project,
				"--format", "json",
				"--certify",
			}, &stdout, &stderr)
			if exitCode != test.wantExit {
				t.Fatalf("exit code = %d, want %d; stderr = %s", exitCode, test.wantExit, stderr.String())
			}
			var snapshot certification.Snapshot
			if err := json.Unmarshal(stdout.Bytes(), &snapshot); err != nil {
				t.Fatal(err)
			}
			if snapshot.Status != test.wantStatus {
				t.Fatalf("status = %q, want %q", snapshot.Status, test.wantStatus)
			}
			if test.wantRule == "" {
				if len(snapshot.Findings) != 0 {
					t.Fatalf("findings = %#v, want none", snapshot.Findings)
				}
				return
			}
			if !hasRule(snapshot.Findings, test.wantRule) {
				t.Fatalf("findings = %#v, want rule %q", snapshot.Findings, test.wantRule)
			}
		})
	}
}

func TestCLICompilerFactsHelperProcess(t *testing.T) {
	if os.Getenv("SOLID_CHECK_TEST_BINARY") == "" {
		return
	}
	scanner := bufio.NewScanner(os.Stdin)
	encoder := json.NewEncoder(os.Stdout)
	for scanner.Scan() {
		var request compilerfacts.AnalysisRequest
		if err := json.Unmarshal(scanner.Bytes(), &request); err != nil {
			os.Exit(2)
		}
		tracked := make([]compilerfacts.ExecutionRegion, 0)
		for _, call := range []string{"count()", "readCount()"} {
			remaining := request.Source
			base := 0
			for {
				index := strings.Index(remaining, call)
				if index < 0 {
					break
				}
				start := base + index
				if start > 0 && request.Source[start-1] == '{' {
					tracked = append(tracked, compilerfacts.ExecutionRegion{
						Span:   compilerfacts.Span{Start: start, End: start + len(call)},
						Reason: compilerfacts.RegionJSXChild,
					})
				}
				base = start + len(call)
				remaining = request.Source[base:]
			}
		}
		callbacks := make([]compilerfacts.CallbackRole, 0)
		callbackText := "() => count()"
		if start := strings.LastIndex(request.Source, callbackText); start >= 0 {
			callbacks = append(callbacks, compilerfacts.CallbackRole{
				Span: compilerfacts.Span{Start: start, End: start + len(callbackText)},
				Role: compilerfacts.CallbackEventHandler,
			})
		}
		response := map[string]any{
			"ok": true,
			"executionMap": compilerfacts.ExecutionMap{
				CompilerFactsProtocol: compilerfacts.ProtocolVersion,
				SourceHash:            request.SourceHash,
				TrackedRegions:        tracked,
				CallbackRoles:         callbacks,
			},
		}
		if err := encoder.Encode(response); err != nil {
			_, _ = fmt.Fprintln(os.Stderr, err)
			os.Exit(3)
		}
	}
	os.Exit(0)
}

func hasRule(findings []certification.Finding, rule string) bool {
	for _, finding := range findings {
		if finding.Rule == rule {
			return true
		}
	}
	return false
}
