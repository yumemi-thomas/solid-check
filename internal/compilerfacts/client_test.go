package compilerfacts_test

import (
	"bufio"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"strings"
	"testing"
	"time"

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

	request := compilerfacts.NewRequest("/workspace/App.tsx", []byte("const view = <div />;"), compilerfacts.CompilerOptions{ModuleName: "dom", Generate: "dom"})
	_, err = client.Analyze(context.Background(), request)
	if err == nil || !strings.Contains(err.Error(), "source hash") {
		t.Fatalf("Analyze() error = %v, want source hash validation failure", err)
	}
}

func TestClientRejectsInvalidRequestBeforeSendingIt(t *testing.T) {
	client := startHelperClient(t, "valid")
	request := compilerfacts.NewRequest("", []byte("const view = <div />;"), compilerfacts.CompilerOptions{})
	if _, err := client.Analyze(context.Background(), request); err == nil || !strings.Contains(err.Error(), "request") {
		t.Fatalf("Analyze() error = %v, want request validation error", err)
	}
}

func TestClientCancellationTerminatesUnresponsiveSidecar(t *testing.T) {
	client := startHelperClient(t, "hang")
	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()
	request := compilerfacts.NewRequest("/workspace/App.tsx", []byte("const view = <div />;"), compilerfacts.CompilerOptions{ModuleName: "dom", Generate: "dom"})
	started := time.Now()
	_, err := client.Analyze(ctx, request)
	if !errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("Analyze() error = %v, want deadline exceeded", err)
	}
	if time.Since(started) > 2*time.Second {
		t.Fatalf("cancellation took %s", time.Since(started))
	}
	if _, err := client.Analyze(context.Background(), request); !errors.Is(err, compilerfacts.ErrClientClosed) {
		t.Fatalf("Analyze() after cancellation = %v, want ErrClientClosed", err)
	}
}

func TestClientFailsClosedWhenSidecarCrashesOrReturnsUnknownFields(t *testing.T) {
	request := compilerfacts.NewRequest("/workspace/App.tsx", []byte("const view = <div />;"), compilerfacts.CompilerOptions{ModuleName: "dom", Generate: "dom"})
	for _, mode := range []string{"crash", "unknown-field", "invalid-error"} {
		t.Run(mode, func(t *testing.T) {
			client := startHelperClient(t, mode)
			if _, err := client.Analyze(context.Background(), request); err == nil {
				t.Fatal("Analyze() succeeded with invalid sidecar behavior")
			}
		})
	}
}

func startHelperClient(t *testing.T, mode string) *compilerfacts.Client {
	t.Helper()
	client, err := compilerfacts.Start(context.Background(), compilerfacts.ProcessConfig{
		Executable: os.Args[0],
		Args:       []string{"-test.run=TestCompilerFactsHelperProcess", "--", mode},
		Env:        []string{"GO_WANT_COMPILER_FACTS_HELPER=1"},
	})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = client.Close() })
	return client
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
		if mode == "hang" {
			time.Sleep(time.Hour)
		}
		if mode == "crash" {
			os.Exit(7)
		}
		if mode == "invalid-error" {
			_ = encoder.Encode(map[string]any{"ok": false, "error": map[string]any{"message": "missing code"}})
			continue
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
		if mode == "unknown-field" {
			response["unexpected"] = true
		}
		if err := encoder.Encode(response); err != nil {
			_, _ = fmt.Fprintln(os.Stderr, err)
			os.Exit(3)
		}
	}
	os.Exit(0)
}
