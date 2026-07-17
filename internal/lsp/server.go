package lsp

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"unicode/utf16"
	"unicode/utf8"

	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

type Options struct {
	Project  engine.ProjectConfig
	ReadFile func(string) ([]byte, error)
}

type Server struct {
	mu          sync.RWMutex
	backend     engine.CertificationEngine
	options     Options
	session     engine.ProjectSession
	snapshot    certification.Snapshot
	documents   map[string]document
	versions    map[string]uint64
	published   map[string]string
	writer      io.Writer
	shutdown    bool
	projectRoot string
}
type document struct {
	source  []byte
	version int
}

func New(ctx context.Context, backend engine.CertificationEngine, options Options) (*Server, error) {
	if backend == nil {
		return nil, errors.New("certification engine is required")
	}
	if options.ReadFile == nil {
		options.ReadFile = os.ReadFile
	}
	session, err := backend.OpenProject(ctx, options.Project)
	if err != nil {
		return nil, err
	}
	snapshot, err := session.Snapshot(ctx, nil)
	if err != nil {
		_ = session.Close()
		return nil, err
	}
	projectRoot := ""
	if options.Project.ConfigPath != "" {
		if absoluteConfig, absoluteErr := filepath.Abs(options.Project.ConfigPath); absoluteErr == nil {
			projectRoot = filepath.Dir(absoluteConfig)
		}
	}
	return &Server{backend: backend, options: options, session: session, snapshot: snapshot, documents: map[string]document{}, versions: map[string]uint64{}, published: map[string]string{}, projectRoot: projectRoot}, nil
}

// clientPath restores the project root spelling supplied by the user. Some
// TypeScript hosts canonicalize paths to lower case on case-insensitive file
// systems; LSP clients still require diagnostic URIs to match worktree casing.
func (s *Server) clientPath(path string) string {
	path = filepath.Clean(path)
	root := filepath.Clean(s.projectRoot)
	if root == "." || len(path) < len(root) || !strings.EqualFold(path[:len(root)], root) {
		return path
	}
	if len(path) > len(root) && !os.IsPathSeparator(path[len(root)]) {
		return path
	}
	return root + path[len(root):]
}

func (s *Server) samePath(left, right string) bool {
	return s.clientPath(left) == s.clientPath(right)
}

func (s *Server) Close() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.session == nil {
		return nil
	}
	err := s.session.Close()
	s.session = nil
	return err
}
func (s *Server) Snapshot() certification.Snapshot {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.snapshot
}

func (s *Server) Serve(ctx context.Context, input io.Reader, output io.Writer) error {
	s.writer = output
	for {
		payload, err := readMessage(input)
		if errors.Is(err, io.EOF) {
			return nil
		}
		if err != nil {
			return err
		}
		var message request
		if err := json.Unmarshal(payload, &message); err != nil {
			if writeErr := writeFrame(output, response{JSONRPC: "2.0", Error: &responseError{Code: -32700, Message: err.Error()}}); writeErr != nil {
				return writeErr
			}
			continue
		}
		result, responseErr, exit := s.dispatch(ctx, message)
		if rawID(message.ID) {
			if err := writeFrame(output, response{JSONRPC: "2.0", ID: message.ID, Result: result, Error: responseErr}); err != nil {
				return err
			}
		}
		if exit {
			return nil
		}
	}
}

func (s *Server) dispatch(ctx context.Context, message request) (any, *responseError, bool) {
	decode := func(target any) *responseError {
		if len(message.Params) == 0 {
			return nil
		}
		if err := json.Unmarshal(message.Params, target); err != nil {
			return &responseError{Code: -32602, Message: err.Error()}
		}
		return nil
	}
	switch message.Method {
	case "initialize":
		var params initializeParams
		if problem := decode(&params); problem != nil {
			return nil, problem, false
		}
		// Diagnostics are pushed after every open/change/close. Do not advertise
		// diagnosticProvider: clients such as Zed switch to the pull model when it
		// is present and then wait for workspace/diagnostic/refresh requests that
		// this server does not send. The textDocument/diagnostic handler remains
		// available for clients that explicitly request a snapshot.
		return map[string]any{"capabilities": map[string]any{"positionEncoding": "utf-16", "textDocumentSync": map[string]any{"openClose": true, "change": 1}, "codeActionProvider": true, "experimental": map[string]any{"solid/checkSnapshot": true, "solid/explainFinding": true}}, "serverInfo": map[string]string{"name": "solid-checkd"}}, nil, false
	case "initialized":
		if err := s.publishDiagnostics(true); err != nil {
			return nil, internalError(err), false
		}
		return nil, nil, false
	case "shutdown":
		s.shutdown = true
		return nil, nil, false
	case "exit":
		return nil, nil, true
	case "textDocument/didOpen":
		var params didOpenParams
		if problem := decode(&params); problem != nil {
			return nil, problem, false
		}
		if err := s.didOpen(ctx, params); err != nil {
			return nil, internalError(err), false
		}
		return nil, nil, false
	case "textDocument/didChange":
		var params didChangeParams
		if problem := decode(&params); problem != nil {
			return nil, problem, false
		}
		if err := s.didChange(ctx, params); err != nil {
			return nil, internalError(err), false
		}
		return nil, nil, false
	case "textDocument/didClose":
		var params didCloseParams
		if problem := decode(&params); problem != nil {
			return nil, problem, false
		}
		if err := s.didClose(ctx, params); err != nil {
			return nil, internalError(err), false
		}
		return nil, nil, false
	case "solid/checkSnapshot":
		return s.Snapshot(), nil, false
	case "solid/explainFinding":
		var params explainParams
		if problem := decode(&params); problem != nil {
			return nil, problem, false
		}
		finding, err := s.explain(params)
		if err != nil {
			return nil, &responseError{Code: -32001, Message: err.Error()}, false
		}
		return finding, nil, false
	case "textDocument/diagnostic":
		var params struct {
			TextDocument textDocumentIdentifier `json:"textDocument"`
		}
		if problem := decode(&params); problem != nil {
			return nil, problem, false
		}
		items, err := s.diagnosticsForURI(params.TextDocument.URI)
		if err != nil {
			return nil, internalError(err), false
		}
		return map[string]any{"kind": "full", "items": items}, nil, false
	case "textDocument/codeAction":
		var params codeActionParams
		if problem := decode(&params); problem != nil {
			return nil, problem, false
		}
		actions, err := s.codeActions(params)
		if err != nil {
			return nil, internalError(err), false
		}
		return actions, nil, false
	default:
		if rawID(message.ID) {
			return nil, &responseError{Code: -32601, Message: "method not found"}, false
		}
		return nil, nil, false
	}
}

func (s *Server) diagnosticsForURI(uri string) ([]diagnostic, error) {
	path, err := uriToPath(uri)
	if err != nil {
		return nil, err
	}
	items := []diagnostic{}
	for _, finding := range s.snapshot.Findings {
		if finding.PrimaryLocation == nil || !s.samePath(finding.PrimaryLocation.Path, path) {
			continue
		}
		item, itemErr := s.diagnostic(finding)
		if itemErr != nil {
			return nil, itemErr
		}
		items = append(items, item)
	}
	return items, nil
}
func internalError(err error) *responseError {
	return &responseError{Code: -32603, Message: err.Error()}
}

func (s *Server) didOpen(ctx context.Context, params didOpenParams) error {
	path, err := uriToPath(params.TextDocument.URI)
	if err != nil {
		return err
	}
	previous, existed := s.documents[path]
	s.documents[path] = document{source: []byte(params.TextDocument.Text), version: params.TextDocument.Version}
	if err := s.update(ctx, path, params.TextDocument.Version, []byte(params.TextDocument.Text)); err != nil {
		if existed {
			s.documents[path] = previous
		} else {
			delete(s.documents, path)
		}
		return err
	}
	return nil
}
func (s *Server) didChange(ctx context.Context, params didChangeParams) error {
	if len(params.ContentChanges) != 1 {
		return errors.New("solid-checkd requires exactly one full-document content change")
	}
	path, err := uriToPath(params.TextDocument.URI)
	if err != nil {
		return err
	}
	if current, ok := s.documents[path]; ok && params.TextDocument.Version <= current.version {
		return nil
	}
	source := []byte(params.ContentChanges[0].Text)
	previous, existed := s.documents[path]
	s.documents[path] = document{source: source, version: params.TextDocument.Version}
	if err := s.update(ctx, path, params.TextDocument.Version, source); err != nil {
		if existed {
			s.documents[path] = previous
		} else {
			delete(s.documents, path)
		}
		return err
	}
	return nil
}
func (s *Server) didClose(ctx context.Context, params didCloseParams) error {
	path, err := uriToPath(params.TextDocument.URI)
	if err != nil {
		return err
	}
	previous, existed := s.documents[path]
	delete(s.documents, path)
	source, err := s.options.ReadFile(path)
	if errors.Is(err, os.ErrNotExist) {
		err = s.updateDeleted(ctx, path)
	} else if err == nil {
		err = s.update(ctx, path, 0, source)
	}
	if err != nil {
		if existed {
			s.documents[path] = previous
		}
		return err
	}
	return nil
}
func (s *Server) nextVersion(path string, editorVersion int) uint64 {
	candidate := uint64(editorVersion)
	if candidate <= s.versions[path] {
		candidate = s.versions[path] + 1
	}
	s.versions[path] = candidate
	return candidate
}
func (s *Server) update(ctx context.Context, path string, editorVersion int, source []byte) error {
	version := s.nextVersion(path, editorVersion)
	_, err := s.session.Update(ctx, []engine.FileChange{{Path: path, Version: version, Source: source}})
	if err != nil {
		return err
	}
	return s.refresh(ctx)
}
func (s *Server) updateDeleted(ctx context.Context, path string) error {
	version := s.nextVersion(path, 0)
	_, err := s.session.Update(ctx, []engine.FileChange{{Path: path, Version: version, Deleted: true}})
	if err != nil {
		return err
	}
	return s.refresh(ctx)
}
func (s *Server) refresh(ctx context.Context) error {
	snapshot, err := s.session.Snapshot(ctx, nil)
	if err != nil {
		return err
	}
	s.snapshot = snapshot
	if s.writer != nil {
		return s.publishDiagnostics(false)
	}
	return nil
}

func (s *Server) publishDiagnostics(force bool) error {
	byPath := map[string][]diagnostic{}
	for _, finding := range s.snapshot.Findings {
		if finding.PrimaryLocation == nil {
			continue
		}
		item, err := s.diagnostic(finding)
		if err != nil {
			return err
		}
		path := s.clientPath(finding.PrimaryLocation.Path)
		byPath[path] = append(byPath[path], item)
	}
	pathsSet := make(map[string]struct{}, len(byPath)+len(s.published)+len(s.documents))
	for path := range byPath {
		pathsSet[path] = struct{}{}
	}
	if !force {
		for path := range s.published {
			pathsSet[path] = struct{}{}
		}
		for path := range s.documents {
			pathsSet[path] = struct{}{}
		}
	}
	paths := make([]string, 0, len(pathsSet))
	for path := range pathsSet {
		paths = append(paths, path)
	}
	sort.Strings(paths)
	for _, path := range paths {
		diagnostics := byPath[path]
		if diagnostics == nil {
			diagnostics = []diagnostic{}
		}
		encoded, err := json.Marshal(diagnostics)
		if err != nil {
			return err
		}
		fingerprint := string(encoded)
		if !force && s.published[path] == fingerprint {
			continue
		}
		params := map[string]any{"uri": pathToURI(path), "diagnostics": diagnostics}
		if document, open := s.documents[path]; open {
			params["version"] = document.version
		}
		if err := writeFrame(s.writer, notification{JSONRPC: "2.0", Method: "textDocument/publishDiagnostics", Params: params}); err != nil {
			return err
		}
		s.published[path] = fingerprint
	}
	return nil
}
func (s *Server) diagnostic(finding certification.Finding) (diagnostic, error) {
	rng, err := s.locationRange(*finding.PrimaryLocation)
	if err != nil {
		return diagnostic{}, err
	}
	severity := 1
	if finding.Severity == certification.SeverityWarning {
		severity = 2
	}
	related := []diagnosticRelatedInformation{}
	for _, item := range finding.RelatedLocations {
		if strings.Contains(item.Path, "://") {
			continue
		}
		itemRange, rangeErr := s.locationRange(item)
		if rangeErr != nil {
			return diagnostic{}, rangeErr
		}
		related = append(related, diagnosticRelatedInformation{Location: location{URI: pathToURI(s.clientPath(item.Path)), Range: itemRange}, Message: "related evidence for " + finding.ID})
	}
	for _, evidence := range finding.Evidence {
		if evidence.Location == nil || strings.Contains(evidence.Location.Path, "://") {
			continue
		}
		itemRange, rangeErr := s.locationRange(*evidence.Location)
		if rangeErr != nil {
			return diagnostic{}, rangeErr
		}
		related = append(related, diagnosticRelatedInformation{Location: location{URI: pathToURI(s.clientPath(evidence.Location.Path)), Range: itemRange}, Message: evidence.Message})
	}
	return diagnostic{Range: rng, Severity: severity, Code: finding.ID, Source: "solid-check", Message: finding.Message, RelatedInformation: related, Data: map[string]any{"findingId": finding.ID, "rule": finding.Rule, "kind": finding.Kind}}, nil
}
func (s *Server) explain(params explainParams) (certification.Finding, error) {
	for _, finding := range s.snapshot.Findings {
		if finding.ID != params.FindingID {
			continue
		}
		if params.URI != "" && finding.PrimaryLocation != nil {
			path, err := uriToPath(params.URI)
			if err != nil {
				return certification.Finding{}, err
			}
			if !s.samePath(finding.PrimaryLocation.Path, path) {
				continue
			}
		}
		if params.StartByte != nil && (finding.PrimaryLocation == nil || finding.PrimaryLocation.StartByte != *params.StartByte) {
			continue
		}
		return finding, nil
	}
	return certification.Finding{}, fmt.Errorf("finding %q not found", params.FindingID)
}
func (s *Server) codeActions(params codeActionParams) ([]codeAction, error) {
	path, err := uriToPath(params.TextDocument.URI)
	if err != nil {
		return nil, err
	}
	actions := []codeAction{}
	for _, finding := range s.snapshot.Findings {
		if finding.PrimaryLocation == nil || !s.samePath(finding.PrimaryLocation.Path, path) {
			continue
		}
		for _, fix := range finding.Fixes {
			changes := map[string][]textEdit{}
			for _, edit := range fix.Edits {
				rng, rangeErr := s.locationRange(edit.Location)
				if rangeErr != nil {
					return nil, rangeErr
				}
				uri := pathToURI(s.clientPath(edit.Location.Path))
				changes[uri] = append(changes[uri], textEdit{Range: rng, NewText: edit.NewText})
			}
			actions = append(actions, codeAction{Title: fix.Message, Kind: "quickfix", IsPreferred: fix.Applicability == certification.FixSafe, Edit: workspaceEdit{Changes: changes}, Data: map[string]any{"findingId": finding.ID}})
		}
	}
	return actions, nil
}
func (s *Server) source(path string) ([]byte, error) {
	path = filepath.Clean(path)
	if document, ok := s.documents[path]; ok {
		return document.source, nil
	}
	return s.options.ReadFile(path)
}
func (s *Server) locationRange(item certification.SourceLocation) (lspRange, error) {
	source, err := s.source(item.Path)
	if err != nil {
		return lspRange{}, err
	}
	return lspRange{Start: bytePosition(source, item.StartByte), End: bytePosition(source, item.EndByte)}, nil
}
func bytePosition(source []byte, offset int) position {
	if offset < 0 {
		offset = 0
	}
	if offset > len(source) {
		offset = len(source)
	}
	lineStart := 0
	line := 0
	for index, value := range source[:offset] {
		if value == '\n' {
			line++
			lineStart = index + 1
		}
	}
	character := 0
	for len(source[lineStart:offset]) > 0 {
		runeValue, size := utf8.DecodeRune(source[lineStart:offset])
		character += len(utf16.Encode([]rune{runeValue}))
		lineStart += size
	}
	return position{Line: line, Character: character}
}
