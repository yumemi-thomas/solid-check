package compilerfacts_test

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
)

func TestClientReusesOneSidecarForMultipleAnalyses(t *testing.T) {
	client, err := compilerfacts.Start(context.Background(), compilerfacts.ProcessConfig{
		Executable: os.Args[0],
		Args:       []string{"-test.run=TestCompilerFactsHelperProcess", "--", "valid"},
		Env:        []string{"GO_WANT_COMPILER_FACTS_HELPER=1"},
	})
	if err != nil {
		t.Fatalf("Start() error = %v", err)
	}
	t.Cleanup(func() { _ = client.Close() })

	for _, source := range []string{
		"const first = <div>{one()}</div>;",
		"const second = <div>{two()}</div>;",
	} {
		request := compilerfacts.NewRequest("/workspace/App.tsx", []byte(source), compilerfacts.CompilerOptions{
			ModuleName: "dom",
			Generate:   "dom",
		})
		facts, err := client.Analyze(context.Background(), request)
		if err != nil {
			t.Fatalf("Analyze() error = %v", err)
		}
		if facts.SourceHash != request.SourceHash {
			t.Fatalf("SourceHash = %q, want %q", facts.SourceHash, request.SourceHash)
		}
	}
}

func TestClientRejectsInvalidSidecarFacts(t *testing.T) {
	client, err := compilerfacts.Start(context.Background(), compilerfacts.ProcessConfig{
		Executable: os.Args[0],
		Args:       []string{"-test.run=TestCompilerFactsHelperProcess", "--", "stale"},
		Env:        []string{"GO_WANT_COMPILER_FACTS_HELPER=1"},
	})
	if err != nil {
		t.Fatalf("Start() error = %v", err)
	}
	t.Cleanup(func() { _ = client.Close() })

	request := compilerfacts.NewRequest("/workspace/App.tsx", []byte("const view = <div />;"), compilerfacts.CompilerOptions{})
	_, err = client.Analyze(context.Background(), request)
	if err == nil || !strings.Contains(err.Error(), "source hash") {
		t.Fatalf("Analyze() error = %v, want source hash validation failure", err)
	}
}

func TestCompilerFactsHelperProcess(t *testing.T) {
	if os.Getenv("GO_WANT_COMPILER_FACTS_HELPER") != "1" {
		return
	}
	mode := os.Args[len(os.Args)-1]
	scanner := bufio.NewScanner(os.Stdin)
	encoder := json.NewEncoder(os.Stdout)
	for scanner.Scan() {
		var request compilerfacts.AnalysisRequest
		if err := json.Unmarshal(scanner.Bytes(), &request); err != nil {
			os.Exit(2)
		}
		hash := request.SourceHash
		if mode == "stale" {
			hash = "sha256:stale"
		}
		response := map[string]any{
			"ok": true,
			"executionMap": compilerfacts.ExecutionMap{
				CompilerFactsProtocol: compilerfacts.ProtocolVersion,
				SourceHash:            hash,
			},
		}
		if err := encoder.Encode(response); err != nil {
			_, _ = fmt.Fprintln(os.Stderr, err)
			os.Exit(3)
		}
	}
	os.Exit(0)
}
