// Package engine owns project-session orchestration. Analysis backends feed
// this package facts; adapters consume only certification snapshots.
package engine

import (
	"context"
	"errors"

	"github.com/yumemi-thomas/solid-check/pkg/certification"
	"github.com/yumemi-thomas/solid-check/pkg/contracts"
)

var ErrSessionClosed = errors.New("project session is closed")

type ProjectConfig struct {
	ConfigPath    string
	ContractPaths []string
}

type FileChange struct {
	Path    string
	Version uint64
	Source  []byte
	Deleted bool
}

type AnalysisScope struct {
	Paths []string
}

type AnalysisDelta struct {
	Version       uint64
	AffectedPaths []string
}

type CertificationEngine interface {
	OpenProject(context.Context, ProjectConfig) (ProjectSession, error)
}

type ProjectSession interface {
	Update(context.Context, []FileChange) (AnalysisDelta, error)
	Snapshot(context.Context, *AnalysisScope) (certification.Snapshot, error)
	Close() error
}

type PackageContractOptions struct {
	Package               contracts.PackageIdentity
	CompilerFactsProtocol int
	Artifacts             contracts.Artifacts
}

type PackageContractEmitter interface {
	EmitPackageContract(context.Context, PackageContractOptions) (contracts.Contract, error)
}
