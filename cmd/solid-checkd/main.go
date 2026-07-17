package main

import (
	"context"
	"flag"
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/lsp"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
)

func main() { os.Exit(run(context.Background(), os.Args[1:], os.Stdin, os.Stdout, os.Stderr)) }

type stringListFlag []string

func (values *stringListFlag) String() string         { return strings.Join(*values, ",") }
func (values *stringListFlag) Set(value string) error { *values = append(*values, value); return nil }

func run(ctx context.Context, args []string, stdin io.Reader, stdout, stderr io.Writer) int {
	flags := flag.NewFlagSet("solid-checkd", flag.ContinueOnError)
	flags.SetOutput(stderr)
	project := flags.String("project", "tsconfig.json", "path to a TypeScript project")
	var contracts stringListFlag
	flags.Var(&contracts, "contract", "load a solid-reactivity.json package contract (repeatable)")
	if err := flags.Parse(args); err != nil {
		return 2
	}

	backend := engine.NativeEngine{OpenTypeFacts: tsgo.OpenProject}
	if executable := os.Getenv("SOLID_COMPILER_FACTS_BIN"); executable != "" {
		backend.OpenCompilerFacts = func(ctx context.Context) (compilerfacts.Analyzer, error) {
			return compilerfacts.Start(ctx, compilerfacts.ProcessConfig{Executable: executable})
		}
	}
	server, err := lsp.New(ctx, backend, lsp.Options{Project: engine.ProjectConfig{ConfigPath: *project, ContractPaths: append([]string(nil), contracts...)}})
	if err != nil {
		fmt.Fprintf(stderr, "solid-checkd: %v\n", err)
		return 2
	}
	defer server.Close()
	if err := server.Serve(ctx, stdin, stdout); err != nil {
		fmt.Fprintf(stderr, "solid-checkd: %v\n", err)
		return 2
	}
	return 0
}
