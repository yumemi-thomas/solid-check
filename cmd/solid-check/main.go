package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
	"github.com/yumemi-thomas/solid-check/pkg/contracts"
)

func main() {
	os.Exit(run(context.Background(), os.Args[1:], os.Stdout, os.Stderr))
}

type stringListFlag []string

func (values *stringListFlag) String() string {
	return strings.Join(*values, ",")
}

func (values *stringListFlag) Set(value string) error {
	*values = append(*values, value)
	return nil
}

func run(ctx context.Context, args []string, stdout, stderr io.Writer) int {
	if len(args) != 0 && args[0] == "oxlint" {
		return runOxlint(ctx, args[1:], stdout, stderr)
	}
	flags := flag.NewFlagSet("solid-check", flag.ContinueOnError)
	flags.SetOutput(stderr)
	project := flags.String("project", "tsconfig.json", "path to a TypeScript project")
	format := flags.String("format", "text", "output format: text or json")
	certify := flags.Bool("certify", false, "fail unless the project is certified")
	emitContract := flags.String("emit-contract", "", "write a solid-reactivity.json package contract")
	packageName := flags.String("package-name", "", "package name used by --emit-contract")
	packageVersion := flags.String("package-version", "", "package version used by --emit-contract")
	declarationArtifact := flags.String("declaration-artifact", "", "declaration file to hash into an emitted contract")
	implementationArtifact := flags.String("implementation-artifact", "", "implementation file to hash into an emitted contract")
	var contractPaths stringListFlag
	flags.Var(&contractPaths, "contract", "load a solid-reactivity.json package contract (repeatable)")
	var validateContractPaths stringListFlag
	flags.Var(&validateContractPaths, "validate-contract", "validate a package contract and its artifacts (repeatable)")
	if err := flags.Parse(args); err != nil {
		return 2
	}
	if *format != "text" && *format != "json" {
		fmt.Fprintf(stderr, "solid-check: unsupported format %q\n", *format)
		return 2
	}
	if len(validateContractPaths) != 0 {
		for _, path := range validateContractPaths {
			if _, err := contracts.LoadFile(path); err != nil {
				fmt.Fprintf(stderr, "solid-check: %v\n", err)
				return 2
			}
		}
		return 0
	}

	session, err := nativeEngine().OpenProject(ctx, engine.ProjectConfig{
		ConfigPath:    *project,
		ContractPaths: append([]string(nil), contractPaths...),
	})
	if err != nil {
		fmt.Fprintf(stderr, "solid-check: %v\n", err)
		return 2
	}
	defer session.Close()
	if *emitContract != "" {
		if *packageName == "" {
			fmt.Fprintln(stderr, "solid-check: --package-name is required with --emit-contract")
			return 2
		}
		emitter, ok := session.(engine.PackageContractEmitter)
		if !ok {
			fmt.Fprintln(stderr, "solid-check: project session cannot emit package contracts")
			return 2
		}
		artifacts := contracts.Artifacts{}
		if *declarationArtifact != "" {
			artifacts.Declaration, err = contracts.ArtifactForFile(*emitContract, *declarationArtifact)
			if err != nil {
				fmt.Fprintf(stderr, "solid-check: %v\n", err)
				return 2
			}
		}
		if *implementationArtifact != "" {
			artifacts.Implementation, err = contracts.ArtifactForFile(*emitContract, *implementationArtifact)
			if err != nil {
				fmt.Fprintf(stderr, "solid-check: %v\n", err)
				return 2
			}
		}
		contract, err := emitter.EmitPackageContract(ctx, engine.PackageContractOptions{
			Package:               contracts.PackageIdentity{Name: *packageName, Version: *packageVersion},
			CompilerFactsProtocol: compilerfacts.ProtocolVersion,
			Artifacts:             artifacts,
		})
		if err != nil {
			fmt.Fprintf(stderr, "solid-check: %v\n", err)
			return 2
		}
		if err := contracts.WriteFile(*emitContract, contract); err != nil {
			fmt.Fprintf(stderr, "solid-check: %v\n", err)
			return 2
		}
		return 0
	}

	snapshot, err := session.Snapshot(ctx, nil)
	if err != nil {
		fmt.Fprintf(stderr, "solid-check: %v\n", err)
		return 2
	}
	if *format == "json" {
		encoder := json.NewEncoder(stdout)
		encoder.SetIndent("", "  ")
		if err := encoder.Encode(snapshot); err != nil {
			fmt.Fprintf(stderr, "solid-check: %v\n", err)
			return 2
		}
	} else {
		fmt.Fprintf(stdout, "%s: %s\n", *project, snapshot.Status)
		for _, finding := range snapshot.Findings {
			fmt.Fprintf(stdout, "%s [%s] %s\n", finding.ID, finding.Kind, finding.Message)
		}
	}

	if *certify && snapshot.Status != certification.StatusCertified {
		return 1
	}
	return 0
}

func runOxlint(ctx context.Context, args []string, stdout, stderr io.Writer) int {
	project, contractPaths, oxlintArgs, err := parseOxlintArgs(args)
	if err != nil {
		fmt.Fprintf(stderr, "solid-check oxlint: %v\n", err)
		return 2
	}

	session, err := nativeEngine().OpenProject(ctx, engine.ProjectConfig{
		ConfigPath:    project,
		ContractPaths: append([]string(nil), contractPaths...),
	})
	if err != nil {
		fmt.Fprintf(stderr, "solid-check oxlint: %v\n", err)
		return 2
	}
	defer session.Close()
	snapshot, err := session.Snapshot(ctx, nil)
	if err != nil {
		fmt.Fprintf(stderr, "solid-check oxlint: %v\n", err)
		return 2
	}

	file, err := os.CreateTemp("", "solid-check-oxlint-*.json")
	if err != nil {
		fmt.Fprintf(stderr, "solid-check oxlint: %v\n", err)
		return 2
	}
	path := file.Name()
	defer os.Remove(path)
	encoder := json.NewEncoder(file)
	if err := encoder.Encode(snapshot); err != nil {
		_ = file.Close()
		fmt.Fprintf(stderr, "solid-check oxlint: %v\n", err)
		return 2
	}
	if err := file.Close(); err != nil {
		fmt.Fprintf(stderr, "solid-check oxlint: %v\n", err)
		return 2
	}

	executable := os.Getenv("OXLINT_BIN")
	if executable == "" {
		executable = "oxlint"
	}
	if !hasOxlintFormat(oxlintArgs) {
		oxlintArgs = append([]string{"--format=default"}, oxlintArgs...)
	}
	command := exec.CommandContext(ctx, executable, oxlintArgs...)
	command.Env = append(os.Environ(), "SOLID_CHECK_SNAPSHOT_PATH="+path)
	command.Stdout = stdout
	command.Stderr = stderr
	if err := command.Run(); err != nil {
		if exitError, ok := err.(*exec.ExitError); ok {
			return exitError.ExitCode()
		}
		fmt.Fprintf(stderr, "solid-check oxlint: start %s: %v\n", executable, err)
		return 2
	}
	return 0
}

func parseOxlintArgs(args []string) (string, []string, []string, error) {
	project := "tsconfig.json"
	contracts := []string(nil)
	passthrough := make([]string, 0, len(args))
	for index := 0; index < len(args); index++ {
		arg := args[index]
		if arg == "--" {
			passthrough = append(passthrough, args[index+1:]...)
			break
		}
		if arg == "--project" || arg == "-project" || arg == "--contract" || arg == "-contract" {
			if index+1 >= len(args) {
				return "", nil, nil, fmt.Errorf("%s requires a value", arg)
			}
			index++
			if strings.HasSuffix(arg, "project") {
				project = args[index]
			} else {
				contracts = append(contracts, args[index])
			}
			continue
		}
		if value, found := strings.CutPrefix(arg, "--project="); found {
			project = value
			continue
		}
		if value, found := strings.CutPrefix(arg, "--contract="); found {
			contracts = append(contracts, value)
			continue
		}
		passthrough = append(passthrough, arg)
	}
	return project, contracts, passthrough, nil
}

func hasOxlintFormat(args []string) bool {
	for _, arg := range args {
		if arg == "--format" || arg == "-f" || strings.HasPrefix(arg, "--format=") || strings.HasPrefix(arg, "-f=") {
			return true
		}
	}
	return false
}

func nativeEngine() engine.NativeEngine {
	result := engine.NativeEngine{OpenTypeFacts: tsgo.OpenProject}
	if executable := os.Getenv("SOLID_COMPILER_FACTS_BIN"); executable != "" {
		result.OpenCompilerFacts = func(ctx context.Context) (compilerfacts.Analyzer, error) {
			return compilerfacts.Start(ctx, compilerfacts.ProcessConfig{Executable: executable})
		}
	}
	return result
}
