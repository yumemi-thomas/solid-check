// Package contracts defines and validates the non-executable package contract
// consumed by solid-check across package build boundaries.
package contracts

import (
	"bytes"
	"crypto/sha256"
	"embed"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

const (
	SchemaVersion                  = 1
	SupportedCompilerFactsProtocol = 1
)

//go:embed bundled/*.json
var bundledFiles embed.FS

type Contract struct {
	SchemaVersion         int                      `json:"schemaVersion"`
	Package               PackageIdentity          `json:"package"`
	CompilerFactsProtocol int                      `json:"compilerFactsProtocol,omitempty"`
	Artifacts             Artifacts                `json:"artifacts,omitempty"`
	Exports               map[string]ExportSummary `json:"exports"`
	Evidence              Evidence                 `json:"evidence"`
	ContractHash          string                   `json:"-"`
	Path                  string                   `json:"-"`
}

type PackageIdentity struct {
	Name    string `json:"name"`
	Version string `json:"version,omitempty"`
}

type Artifacts struct {
	Declaration    *Artifact `json:"declaration,omitempty"`
	Implementation *Artifact `json:"implementation,omitempty"`
}

type Artifact struct {
	Path string `json:"path"`
	Hash string `json:"hash"`
}

type ExportSummary struct {
	Kind          string            `json:"kind"`
	ReactiveReads []ReactiveRead    `json:"reactiveReads,omitempty"`
	Returns       *ReactiveReturn   `json:"returns,omitempty"`
	Callbacks     []CallbackSummary `json:"callbacks,omitempty"`
	AsyncBehavior string            `json:"asyncBehavior,omitempty"`
}

type ReactiveReturn struct {
	Kind  string `json:"kind"`
	Label string `json:"label"`
}

type CallbackSummary struct {
	Parameter int    `json:"parameter"`
	Execution string `json:"execution"`
}

type ReactiveRead struct {
	Kind  string `json:"kind"`
	Label string `json:"label"`
}

type Evidence struct {
	Kind      string `json:"kind"`
	Generator string `json:"generator,omitempty"`
}

// ArtifactForFile describes an artifact relative to the contract location and
// hashes its exact bytes. Artifacts outside the contract directory are rejected
// so a published contract cannot refer to unrelated files.
func ArtifactForFile(contractPath, artifactPath string) (*Artifact, error) {
	contractDirectory, err := filepath.Abs(filepath.Dir(contractPath))
	if err != nil {
		return nil, fmt.Errorf("resolve package contract directory: %w", err)
	}
	absoluteArtifact, err := filepath.Abs(artifactPath)
	if err != nil {
		return nil, fmt.Errorf("resolve package contract artifact: %w", err)
	}
	relative, err := filepath.Rel(contractDirectory, absoluteArtifact)
	if err != nil {
		return nil, fmt.Errorf("resolve package contract artifact path: %w", err)
	}
	if relative == ".." || strings.HasPrefix(relative, ".."+string(filepath.Separator)) || relative == "." {
		return nil, errors.New("package contract artifact must be a file inside the contract directory")
	}
	data, err := os.ReadFile(absoluteArtifact)
	if err != nil {
		return nil, fmt.Errorf("read package contract artifact: %w", err)
	}
	sum := sha256.Sum256(data)
	return &Artifact{Path: filepath.ToSlash(relative), Hash: "sha256:" + hex.EncodeToString(sum[:])}, nil
}

func LoadFile(path string) (Contract, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return Contract{}, fmt.Errorf("read package contract: %w", err)
	}
	contract, err := decode(data, path)
	if err != nil {
		return Contract{}, err
	}
	if err := contract.validateArtifacts(filepath.Dir(path)); err != nil {
		return Contract{}, err
	}
	return contract, nil
}

// Bundled returns validated contracts shipped with this checker. Callers own
// the returned slice and maps.
func Bundled() ([]Contract, error) {
	entries, err := bundledFiles.ReadDir("bundled")
	if err != nil {
		return nil, fmt.Errorf("read bundled package contracts: %w", err)
	}
	sort.Slice(entries, func(i, j int) bool { return entries[i].Name() < entries[j].Name() })
	result := make([]Contract, 0, len(entries))
	for _, entry := range entries {
		if entry.IsDir() || filepath.Ext(entry.Name()) != ".json" {
			continue
		}
		data, err := bundledFiles.ReadFile("bundled/" + entry.Name())
		if err != nil {
			return nil, fmt.Errorf("read bundled package contract %s: %w", entry.Name(), err)
		}
		contract, err := decode(data, "bundled://"+entry.Name())
		if err != nil {
			return nil, fmt.Errorf("load bundled package contract %s: %w", entry.Name(), err)
		}
		result = append(result, contract)
	}
	return result, nil
}

func decode(data []byte, path string) (Contract, error) {
	decoder := json.NewDecoder(bytes.NewReader(data))
	decoder.DisallowUnknownFields()
	var contract Contract
	if err := decoder.Decode(&contract); err != nil {
		return Contract{}, fmt.Errorf("decode package contract: %w", err)
	}
	var trailing any
	if err := decoder.Decode(&trailing); !errors.Is(err, io.EOF) {
		return Contract{}, errors.New("decode package contract: trailing JSON value")
	}
	if err := contract.Validate(); err != nil {
		return Contract{}, err
	}
	sum := sha256.Sum256(data)
	contract.ContractHash = "sha256:" + hex.EncodeToString(sum[:])
	contract.Path = path
	return contract, nil
}

func WriteFile(path string, contract Contract) error {
	if err := contract.Validate(); err != nil {
		return err
	}
	data, err := json.MarshalIndent(contract, "", "  ")
	if err != nil {
		return fmt.Errorf("encode package contract: %w", err)
	}
	data = append(data, '\n')
	if err := os.WriteFile(path, data, 0o644); err != nil {
		return fmt.Errorf("write package contract: %w", err)
	}
	return nil
}

func (c Contract) Validate() error {
	if c.SchemaVersion != SchemaVersion {
		return fmt.Errorf("package contract schema version %d is unsupported", c.SchemaVersion)
	}
	if c.CompilerFactsProtocol != 0 && c.CompilerFactsProtocol != SupportedCompilerFactsProtocol {
		return fmt.Errorf("package contract compiler facts protocol %d is unsupported", c.CompilerFactsProtocol)
	}
	if c.Package.Name == "" {
		return errors.New("package contract requires package.name")
	}
	if len(c.Exports) == 0 {
		return errors.New("package contract requires at least one export")
	}
	if c.Evidence.Kind != "generated" && c.Evidence.Kind != "reviewed" && c.Evidence.Kind != "trusted" {
		return fmt.Errorf("package contract evidence kind %q is unsupported", c.Evidence.Kind)
	}
	for name, summary := range c.Exports {
		if name == "" || (summary.Kind != "function" && summary.Kind != "value") {
			return fmt.Errorf("package contract export %q has unsupported kind %q", name, summary.Kind)
		}
		if summary.Kind == "value" && (len(summary.ReactiveReads) != 0 || summary.Returns != nil || len(summary.Callbacks) != 0 || summary.AsyncBehavior != "") {
			return fmt.Errorf("package contract value export %q cannot have function effects", name)
		}
		for _, read := range summary.ReactiveReads {
			if read.Kind != "accessor" && read.Kind != "store-path" {
				return fmt.Errorf("package contract export %q has unsupported reactive read kind %q", name, read.Kind)
			}
			if read.Label == "" {
				return fmt.Errorf("package contract export %q has an empty reactive read label", name)
			}
		}
		if summary.Returns != nil {
			if summary.Returns.Kind != "accessor" && summary.Returns.Kind != "store-path" {
				return fmt.Errorf("package contract export %q has unsupported returned reactive kind %q", name, summary.Returns.Kind)
			}
			if summary.Returns.Label == "" {
				return fmt.Errorf("package contract export %q has an empty returned reactive label", name)
			}
		}
		for _, callback := range summary.Callbacks {
			if callback.Parameter < 0 {
				return fmt.Errorf("package contract export %q has a negative callback parameter", name)
			}
			if callback.Execution != "inline" && callback.Execution != "tracked" && callback.Execution != "deferred" {
				return fmt.Errorf("package contract export %q has unsupported callback execution %q", name, callback.Execution)
			}
		}
		if summary.AsyncBehavior != "" && summary.AsyncBehavior != "promise" && summary.AsyncBehavior != "async-iterable" {
			return fmt.Errorf("package contract export %q has unsupported async behavior %q", name, summary.AsyncBehavior)
		}
	}
	return nil
}

func (c Contract) validateArtifacts(directory string) error {
	artifacts := []struct {
		name     string
		artifact *Artifact
	}{
		{name: "declaration", artifact: c.Artifacts.Declaration},
		{name: "implementation", artifact: c.Artifacts.Implementation},
	}
	for _, entry := range artifacts {
		if entry.artifact == nil {
			continue
		}
		if entry.artifact.Path == "" || filepath.IsAbs(entry.artifact.Path) || strings.HasPrefix(filepath.Clean(entry.artifact.Path), "..") {
			return fmt.Errorf("package contract %s artifact path is invalid", entry.name)
		}
		if !strings.HasPrefix(entry.artifact.Hash, "sha256:") {
			return fmt.Errorf("package contract %s artifact hash must use sha256", entry.name)
		}
		data, err := os.ReadFile(filepath.Join(directory, entry.artifact.Path))
		if err != nil {
			return fmt.Errorf("read package contract %s artifact: %w", entry.name, err)
		}
		sum := sha256.Sum256(data)
		actual := "sha256:" + hex.EncodeToString(sum[:])
		if actual != entry.artifact.Hash {
			return fmt.Errorf("package contract %s hash %q does not match artifact hash %q", entry.name, entry.artifact.Hash, actual)
		}
	}
	return nil
}
