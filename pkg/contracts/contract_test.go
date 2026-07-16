package contracts_test

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/yumemi-thomas/solid-check/pkg/contracts"
)

func TestLoadFileRejectsUnsupportedCompilerFactsProtocol(t *testing.T) {
	contract := validContract(t)
	contract["compilerFactsProtocol"] = 2

	_, err := contracts.LoadFile(writeContract(t, contract))
	if err == nil || !strings.Contains(err.Error(), "compiler facts protocol") {
		t.Fatalf("LoadFile() error = %v, want unsupported compiler facts protocol", err)
	}
}

func TestLoadFileRejectsUnknownFields(t *testing.T) {
	contract := validContract(t)
	contract["unexpected"] = true

	_, err := contracts.LoadFile(writeContract(t, contract))
	if err == nil || !strings.Contains(err.Error(), "unknown field") {
		t.Fatalf("LoadFile() error = %v, want unknown field", err)
	}
}

func TestBundledContractsIncludeSolidTrackedPrimitives(t *testing.T) {
	bundled, err := contracts.Bundled()
	if err != nil {
		t.Fatal(err)
	}
	var solid *contracts.Contract
	for index := range bundled {
		if bundled[index].Package.Name == "solid-js" {
			solid = &bundled[index]
			break
		}
	}
	if solid == nil {
		t.Fatal("Bundled() does not contain solid-js")
	}
	if len(bundled) < 2 {
		t.Fatalf("Bundled() = %#v, want core and DOM contracts", bundled)
	}
	createMemo := solid.Exports["createMemo"]
	if createMemo.Returns == nil || createMemo.Returns.Kind != "accessor" {
		t.Fatalf("createMemo = %#v, want returned accessor", createMemo)
	}
	if len(createMemo.Callbacks) != 1 || createMemo.Callbacks[0].Execution != "tracked" {
		t.Fatalf("createMemo = %#v, want tracked compute callback", createMemo)
	}
}

func TestLoadFileRejectsStaleDeclarationArtifact(t *testing.T) {
	fixture := filepath.Join("..", "..", "internal", "reactiveir", "testdata", "package-consumer", "node_modules", "reactive-package")
	contractBytes, err := os.ReadFile(filepath.Join(fixture, "solid-reactivity.json"))
	if err != nil {
		t.Fatal(err)
	}
	directory := t.TempDir()
	if err := os.WriteFile(filepath.Join(directory, "solid-reactivity.json"), contractBytes, 0o600); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(directory, "index.d.ts"), []byte("export declare function readCount(): string;\n"), 0o600); err != nil {
		t.Fatal(err)
	}

	_, err = contracts.LoadFile(filepath.Join(directory, "solid-reactivity.json"))
	if err == nil || !strings.Contains(err.Error(), "declaration hash") {
		t.Fatalf("LoadFile() error = %v, want declaration hash mismatch", err)
	}
}

func validContract(t *testing.T) map[string]any {
	t.Helper()
	return map[string]any{
		"schemaVersion":         1,
		"compilerFactsProtocol": 1,
		"package":               map[string]any{"name": "reactive-package"},
		"exports": map[string]any{
			"readCount": map[string]any{"kind": "function"},
		},
		"evidence": map[string]any{"kind": "generated"},
	}
}

func writeContract(t *testing.T, contract map[string]any) string {
	t.Helper()
	data, err := json.Marshal(contract)
	if err != nil {
		t.Fatal(err)
	}
	path := filepath.Join(t.TempDir(), "solid-reactivity.json")
	if err := os.WriteFile(path, data, 0o600); err != nil {
		t.Fatal(err)
	}
	return path
}
