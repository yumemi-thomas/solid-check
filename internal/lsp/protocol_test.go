package lsp

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"reflect"
	"sort"
	"strings"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/compilerfacts"
	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func TestServerPublishesOverlayDiagnosticsAndRestoresDiskOnClose(t *testing.T) {
	root := t.TempDir()
	path := filepath.Join(root, "App.tsx")
	disk := "const value = safe;\n"
	if err := os.WriteFile(path, []byte(disk), 0o600); err != nil {
		t.Fatal(err)
	}
	backend := &memoryEngine{files: map[string][]byte{path: []byte(disk)}}
	server, err := New(context.Background(), backend, Options{Project: engine.ProjectConfig{ConfigPath: filepath.Join(root, "tsconfig.json")}})
	if err != nil {
		t.Fatal(err)
	}
	defer server.Close()

	uri := pathToURI(path)
	if err := server.didOpen(context.Background(), didOpenParams{TextDocument: textDocumentItem{URI: uri, Version: 1, Text: "const value = unsafe;\n"}}); err != nil {
		t.Fatal(err)
	}
	if got := server.Snapshot(); len(got.Findings) != 1 {
		t.Fatalf("overlay findings = %#v", got.Findings)
	}
	if err := server.didClose(context.Background(), didCloseParams{TextDocument: textDocumentIdentifier{URI: uri}}); err != nil {
		t.Fatal(err)
	}
	if got := server.Snapshot(); len(got.Findings) != 0 {
		t.Fatalf("restored findings = %#v", got.Findings)
	}
}

func TestServerPublishesAnEmptyDiagnosticArrayForACleanDocument(t *testing.T) {
	root := t.TempDir()
	path := filepath.Join(root, "App.tsx")
	source := []byte("const value = safe;\n")
	backend := &memoryEngine{files: map[string][]byte{path: source}}
	input := bytes.NewBuffer(nil)
	for _, message := range []any{
		request{JSONRPC: "2.0", ID: json.RawMessage("1"), Method: "initialize", Params: mustJSON(initializeParams{})},
		request{JSONRPC: "2.0", Method: "textDocument/didOpen", Params: mustJSON(didOpenParams{TextDocument: textDocumentItem{URI: pathToURI(path), Version: 1, Text: string(source)}})},
		request{JSONRPC: "2.0", ID: json.RawMessage("2"), Method: "shutdown"},
		request{JSONRPC: "2.0", Method: "exit"},
	} {
		if err := writeMessage(input, message); err != nil {
			t.Fatal(err)
		}
	}
	var output bytes.Buffer
	server, err := New(context.Background(), backend, Options{})
	if err != nil {
		t.Fatal(err)
	}
	if err := server.Serve(context.Background(), input, &output); err != nil {
		t.Fatal(err)
	}
	frames := decodeFrames(t, output.Bytes())
	for _, frame := range frames {
		var message struct {
			Method string `json:"method"`
			Params struct {
				Diagnostics json.RawMessage `json:"diagnostics"`
			} `json:"params"`
		}
		if err := json.Unmarshal(frame, &message); err != nil {
			t.Fatal(err)
		}
		if message.Method == "textDocument/publishDiagnostics" {
			if string(message.Params.Diagnostics) != "[]" {
				t.Fatalf("clean diagnostics = %s, want []", message.Params.Diagnostics)
			}
			return
		}
	}
	t.Fatalf("publishDiagnostics notification missing: %s", output.String())
}

func TestClientPathRestoresProjectRootCasing(t *testing.T) {
	root := filepath.Join(string(filepath.Separator), "Users", "Thomas", "project")
	server := &Server{projectRoot: root}
	got := server.clientPath(filepath.Join(string(filepath.Separator), "users", "thomas", "project", "src", "App.tsx"))
	want := filepath.Join(root, "src", "App.tsx")
	if got != want {
		t.Fatalf("client path = %q, want %q", got, want)
	}
}

func TestPublishDiagnosticsIgnoresVirtualEvidenceLocations(t *testing.T) {
	root := t.TempDir()
	path := filepath.Join(root, "App.tsx")
	source := []byte("unsafe\n")
	if err := os.WriteFile(path, source, 0o600); err != nil {
		t.Fatal(err)
	}
	virtual := certification.SourceLocation{Path: "bundled://solid-js.json#createMemo", StartByte: 0, EndByte: 1}
	server := &Server{
		options: Options{ReadFile: os.ReadFile},
		snapshot: certification.Snapshot{Findings: []certification.Finding{{
			ID: "SC1001", Rule: "strict-read-untracked", Severity: certification.SeverityError,
			Message: "unsafe read", PrimaryLocation: &certification.SourceLocation{Path: path, StartByte: 0, EndByte: 6},
			Evidence: []certification.EvidenceStep{{Message: "bundled contract", Location: &virtual}},
		}}},
		published: map[string]string{}, documents: map[string]document{}, projectRoot: root,
	}
	var output bytes.Buffer
	server.writer = &output
	if err := server.publishDiagnostics(true); err != nil {
		t.Fatal(err)
	}
	assertPublishedDiagnosticPaths(t, output.Bytes(), []string{path}, false)
}

func TestInitializeAdvertisesPushDiagnostics(t *testing.T) {
	server, err := New(context.Background(), &memoryEngine{files: map[string][]byte{}}, Options{})
	if err != nil {
		t.Fatal(err)
	}
	defer server.Close()
	result, problem, _ := server.dispatch(context.Background(), request{JSONRPC: "2.0", ID: json.RawMessage("1"), Method: "initialize", Params: mustJSON(initializeParams{})})
	if problem != nil {
		t.Fatal(problem)
	}
	capabilities := result.(map[string]any)["capabilities"].(map[string]any)
	if _, advertised := capabilities["diagnosticProvider"]; advertised {
		t.Fatalf("capabilities = %#v; push server must not advertise pull diagnostics", capabilities)
	}
}

func TestInitializedPublishesWorkspaceDiagnosticsThenOnlyChangedFiles(t *testing.T) {
	root := t.TempDir()
	first := filepath.Join(root, "First.tsx")
	second := filepath.Join(root, "Second.tsx")
	files := map[string][]byte{first: []byte("unsafe\n"), second: []byte("unsafe\n")}
	for path, source := range files {
		if err := os.WriteFile(path, source, 0o600); err != nil {
			t.Fatal(err)
		}
	}
	var output bytes.Buffer
	server, err := New(context.Background(), &memoryEngine{files: files}, Options{})
	if err != nil {
		t.Fatal(err)
	}
	defer server.Close()
	server.writer = &output

	if _, problem, _ := server.dispatch(context.Background(), request{JSONRPC: "2.0", Method: "initialized"}); problem != nil {
		t.Fatal(problem)
	}
	assertPublishedDiagnosticPaths(t, output.Bytes(), []string{first, second}, false)

	output.Reset()
	if err := server.didOpen(context.Background(), didOpenParams{TextDocument: textDocumentItem{
		URI: pathToURI(first), Version: 1, Text: "safe\n",
	}}); err != nil {
		t.Fatal(err)
	}
	assertPublishedDiagnosticPaths(t, output.Bytes(), []string{first}, true)
}

func assertPublishedDiagnosticPaths(t *testing.T, output []byte, want []string, wantEmpty bool) {
	t.Helper()
	var got []string
	for _, frame := range decodeFrames(t, output) {
		var message struct {
			Method string `json:"method"`
			Params struct {
				URI         string            `json:"uri"`
				Diagnostics []json.RawMessage `json:"diagnostics"`
			} `json:"params"`
		}
		if err := json.Unmarshal(frame, &message); err != nil {
			t.Fatal(err)
		}
		if message.Method != "textDocument/publishDiagnostics" {
			continue
		}
		path, err := uriToPath(message.Params.URI)
		if err != nil {
			t.Fatal(err)
		}
		if wantEmpty && len(message.Params.Diagnostics) != 0 {
			t.Fatalf("diagnostics for %s = %s, want empty", path, message.Params.Diagnostics)
		}
		if !wantEmpty && len(message.Params.Diagnostics) == 0 {
			t.Fatalf("diagnostics for %s are empty", path)
		}
		got = append(got, path)
	}
	sort.Strings(got)
	sortedWant := append([]string(nil), want...)
	sort.Strings(sortedWant)
	if !reflect.DeepEqual(got, sortedWant) {
		t.Fatalf("published paths = %v, want %v", got, sortedWant)
	}
}

func TestIncrementalSnapshotsEqualCleanSnapshotsAfterEditSequences(t *testing.T) {
	root := t.TempDir()
	path := filepath.Join(root, "App.tsx")
	if err := os.WriteFile(path, []byte("safe"), 0o600); err != nil {
		t.Fatal(err)
	}
	files := map[string][]byte{path: []byte("safe")}
	incrementalEngine := &memoryEngine{files: cloneFiles(files)}
	server, err := New(context.Background(), incrementalEngine, Options{Project: engine.ProjectConfig{ConfigPath: filepath.Join(root, "tsconfig.json")}})
	if err != nil {
		t.Fatal(err)
	}
	defer server.Close()

	for version, source := range []string{"unsafe", "safe", "unsafe unsafe", "safe again"} {
		files[path] = []byte(source)
		if err := server.didChange(context.Background(), didChangeParams{
			TextDocument:   versionedTextDocumentIdentifier{URI: pathToURI(path), Version: version + 1},
			ContentChanges: []textDocumentContentChangeEvent{{Text: source}},
		}); err != nil {
			t.Fatal(err)
		}
		cleanEngine := &memoryEngine{files: cloneFiles(files)}
		clean, err := cleanEngine.OpenProject(context.Background(), engine.ProjectConfig{})
		if err != nil {
			t.Fatal(err)
		}
		want, err := clean.Snapshot(context.Background(), nil)
		_ = clean.Close()
		if err != nil {
			t.Fatal(err)
		}
		if !reflect.DeepEqual(server.Snapshot(), want) {
			t.Fatalf("edit %d incremental = %#v, clean = %#v", version, server.Snapshot(), want)
		}
	}
}

func TestNativeIncrementalSnapshotsEqualCleanEngineSnapshots(t *testing.T) {
	sourceFixture := filepath.Join("..", "reactiveir", "testdata", "tracer")
	root := t.TempDir()
	for _, name := range []string{"App.tsx", "source.ts", "solid-js.d.ts", "tsconfig.json"} {
		source, err := os.ReadFile(filepath.Join(sourceFixture, name))
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(root, name), source, 0o600); err != nil {
			t.Fatal(err)
		}
	}
	backend := nativeTestEngine()
	server, err := New(context.Background(), backend, Options{Project: engine.ProjectConfig{ConfigPath: filepath.Join(root, "tsconfig.json")}})
	if err != nil {
		t.Fatal(err)
	}
	defer server.Close()
	appPath := filepath.Join(root, "App.tsx")
	original, err := os.ReadFile(appPath)
	if err != nil {
		t.Fatal(err)
	}
	corrected := strings.Replace(string(original), "  const value = count();\n  return <div>{value}</div>;", "  return <div>{count()}</div>;", 1)
	for index, source := range [][]byte{[]byte(corrected), original, []byte(corrected)} {
		if err := server.didChange(context.Background(), didChangeParams{TextDocument: versionedTextDocumentIdentifier{URI: pathToURI(appPath), Version: index + 1}, ContentChanges: []textDocumentContentChangeEvent{{Text: string(source)}}}); err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(appPath, source, 0o600); err != nil {
			t.Fatal(err)
		}
		clean, err := nativeTestEngine().OpenProject(context.Background(), engine.ProjectConfig{ConfigPath: filepath.Join(root, "tsconfig.json")})
		if err != nil {
			t.Fatal(err)
		}
		want, err := clean.Snapshot(context.Background(), nil)
		_ = clean.Close()
		if err != nil {
			t.Fatal(err)
		}
		if !reflect.DeepEqual(server.Snapshot(), want) {
			t.Fatalf("edit %d native incremental != clean\nincremental: %#v\nclean: %#v", index, server.Snapshot(), want)
		}
	}
}

func TestNativeServerAnalyzesNewUnsavedProjectFile(t *testing.T) {
	root := t.TempDir()
	config := `{"compilerOptions":{"jsx":"preserve","module":"ESNext","moduleResolution":"Bundler","strict":true,"target":"ES2022"},"include":["*.ts","*.tsx"]}`
	declarations := `declare module "solid-js" {
  export function createSignal<T>(value: T): [() => T, (value: T) => void];
  export function createMemo<T>(compute: () => T): () => Awaited<T>;
}`
	for name, source := range map[string]string{"tsconfig.json": config, "solid-js.d.ts": declarations} {
		if err := os.WriteFile(filepath.Join(root, name), []byte(source), 0o600); err != nil {
			t.Fatal(err)
		}
	}
	server, err := New(context.Background(), nativeTestEngine(), Options{Project: engine.ProjectConfig{ConfigPath: filepath.Join(root, "tsconfig.json")}})
	if err != nil {
		t.Fatal(err)
	}
	defer server.Close()
	path := filepath.Join(root, "Unsaved.tsx")
	source := `import { createMemo, createSignal } from "solid-js";
const [count] = createSignal(0);
const doubled = createMemo(async () => {
  await Promise.resolve();
  return count() * 2;
});`
	if err := server.didOpen(context.Background(), didOpenParams{TextDocument: textDocumentItem{URI: pathToURI(path), Version: 1, Text: source}}); err != nil {
		t.Fatal(err)
	}
	found := false
	for _, finding := range server.Snapshot().Findings {
		found = found || finding.Rule == "reactive-read-after-await" && finding.PrimaryLocation != nil && filepath.Clean(finding.PrimaryLocation.Path) == filepath.Clean(path)
	}
	if !found {
		t.Fatalf("new overlay findings = %#v, want SC1002", server.Snapshot().Findings)
	}
}

func TestNativeServerClearsComponentPropsDiagnosticAfterQuickFixEdit(t *testing.T) {
	root := t.TempDir()
	config := `{"compilerOptions":{"jsx":"preserve","module":"ESNext","moduleResolution":"Bundler","strict":true,"target":"ES2022"},"include":["*.ts","*.tsx"]}`
	original := "interface Props { title: string }\nexport function Card({ title }: Props) {\n  return <h1>{title}</h1>;\n}\n"
	fixed := "interface Props { title: string }\nexport function Card(props: Props) {\n  return <h1>{props.title}</h1>;\n}\n"
	path := filepath.Join(root, "Card.tsx")
	for name, source := range map[string]string{"tsconfig.json": config, "Card.tsx": original} {
		if err := os.WriteFile(filepath.Join(root, name), []byte(source), 0o600); err != nil {
			t.Fatal(err)
		}
	}
	server, err := New(context.Background(), nativeTestEngine(), Options{Project: engine.ProjectConfig{ConfigPath: filepath.Join(root, "tsconfig.json")}})
	if err != nil {
		t.Fatal(err)
	}
	defer server.Close()
	if !snapshotHasRuleAtPath(server.Snapshot(), "component-props-destructure", path) {
		t.Fatalf("initial findings = %#v, want component-props-destructure", server.Snapshot().Findings)
	}
	var output bytes.Buffer
	server.writer = &output
	if _, problem, _ := server.dispatch(context.Background(), request{JSONRPC: "2.0", Method: "initialized"}); problem != nil {
		t.Fatal(problem)
	}
	output.Reset()
	if err := server.didOpen(context.Background(), didOpenParams{TextDocument: textDocumentItem{URI: pathToURI(path), Version: 1, Text: original}}); err != nil {
		t.Fatal(err)
	}
	output.Reset()
	if err := server.didChange(context.Background(), didChangeParams{
		TextDocument:   versionedTextDocumentIdentifier{URI: pathToURI(path), Version: 2},
		ContentChanges: []textDocumentContentChangeEvent{{Text: fixed}},
	}); err != nil {
		t.Fatal(err)
	}
	if snapshotHasRuleAtPath(server.Snapshot(), "component-props-destructure", path) {
		t.Fatalf("fixed findings = %#v, component-props-destructure is stale", server.Snapshot().Findings)
	}
	assertPublishedDiagnosticsExcludeCodeAtVersion(t, output.Bytes(), path, "SC1003", 2)
}

func assertPublishedDiagnosticsExcludeCodeAtVersion(t *testing.T, output []byte, wantPath, excludedCode string, wantVersion int) {
	t.Helper()
	found := false
	for _, frame := range decodeFrames(t, output) {
		var message struct {
			Method string `json:"method"`
			Params struct {
				URI         string `json:"uri"`
				Version     *int   `json:"version"`
				Diagnostics []struct {
					Code string `json:"code"`
				} `json:"diagnostics"`
			} `json:"params"`
		}
		if err := json.Unmarshal(frame, &message); err != nil {
			t.Fatal(err)
		}
		path, err := uriToPath(message.Params.URI)
		if message.Method != "textDocument/publishDiagnostics" || err != nil || filepath.Clean(path) != filepath.Clean(wantPath) {
			continue
		}
		found = true
		if message.Params.Version == nil || *message.Params.Version != wantVersion {
			t.Fatalf("published version = %v, want %d", message.Params.Version, wantVersion)
		}
		for _, diagnostic := range message.Params.Diagnostics {
			if diagnostic.Code == excludedCode {
				t.Fatalf("published diagnostics still contain %s", excludedCode)
			}
		}
	}
	if !found {
		t.Fatalf("no diagnostics published for %s", wantPath)
	}
}

func snapshotHasRuleAtPath(snapshot certification.Snapshot, rule, path string) bool {
	for _, finding := range snapshot.Findings {
		if finding.Rule == rule && finding.PrimaryLocation != nil && filepath.Clean(finding.PrimaryLocation.Path) == filepath.Clean(path) {
			return true
		}
	}
	return false
}

func nativeTestEngine() engine.NativeEngine {
	return engine.NativeEngine{OpenTypeFacts: tsgo.OpenProject, OpenCompilerFacts: func(context.Context) (compilerfacts.Analyzer, error) { return lspTracerAnalyzer{}, nil }}
}

type lspTracerAnalyzer struct{}

func (lspTracerAnalyzer) Analyze(_ context.Context, request compilerfacts.AnalysisRequest) (compilerfacts.ExecutionMap, error) {
	tracked := []compilerfacts.ExecutionRegion{}
	remaining := request.Source
	base := 0
	for {
		index := strings.Index(remaining, "count()")
		if index < 0 {
			break
		}
		start := base + index
		if start > 0 && request.Source[start-1] == '{' {
			tracked = append(tracked, compilerfacts.ExecutionRegion{Span: compilerfacts.Span{Start: start, End: start + len("count()")}, Reason: compilerfacts.RegionJSXChild})
		}
		base = start + len("count()")
		remaining = request.Source[base:]
	}
	callbackText := "() => count()"
	callbackStart := strings.LastIndex(request.Source, callbackText)
	callbacks := []compilerfacts.CallbackRole{}
	if callbackStart >= 0 {
		callbacks = append(callbacks, compilerfacts.CallbackRole{Span: compilerfacts.Span{Start: callbackStart, End: callbackStart + len(callbackText)}, Role: compilerfacts.CallbackEventHandler})
	}
	return compilerfacts.ExecutionMap{CompilerFactsProtocol: compilerfacts.ProtocolVersion, SourceHash: request.SourceHash, TrackedRegions: tracked, CallbackRoles: callbacks}, nil
}
func (lspTracerAnalyzer) Close() error { return nil }

func TestProtocolExposesSnapshotExplanationRelatedLocationsAndFixes(t *testing.T) {
	root := t.TempDir()
	path := filepath.Join(root, "App.tsx")
	source := []byte("unsafe\n")
	if err := os.WriteFile(path, source, 0o600); err != nil {
		t.Fatal(err)
	}
	backend := &memoryEngine{files: map[string][]byte{path: source}}
	input := bytes.NewBuffer(nil)
	for _, message := range []any{
		request{JSONRPC: "2.0", ID: json.RawMessage("1"), Method: "initialize", Params: mustJSON(initializeParams{})},
		request{JSONRPC: "2.0", ID: json.RawMessage("2"), Method: "solid/checkSnapshot"},
		request{JSONRPC: "2.0", ID: json.RawMessage("3"), Method: "solid/explainFinding", Params: mustJSON(explainParams{FindingID: "SC-TEST"})},
		request{JSONRPC: "2.0", ID: json.RawMessage("4"), Method: "textDocument/codeAction", Params: mustJSON(codeActionParams{TextDocument: textDocumentIdentifier{URI: pathToURI(path)}})},
		request{JSONRPC: "2.0", ID: json.RawMessage("5"), Method: "textDocument/diagnostic", Params: mustJSON(map[string]any{"textDocument": textDocumentIdentifier{URI: pathToURI(path)}})},
		request{JSONRPC: "2.0", ID: json.RawMessage("6"), Method: "shutdown"},
		request{JSONRPC: "2.0", Method: "exit"},
	} {
		if err := writeMessage(input, message); err != nil {
			t.Fatal(err)
		}
	}
	var output bytes.Buffer
	server, err := New(context.Background(), backend, Options{})
	if err != nil {
		t.Fatal(err)
	}
	if err := server.Serve(context.Background(), input, &output); err != nil {
		t.Fatal(err)
	}
	responses := decodeFrames(t, output.Bytes())
	if len(responses) != 6 {
		t.Fatalf("responses = %d, want 6: %s", len(responses), output.String())
	}
	encoded := string(output.Bytes())
	for _, token := range []string{"SC-TEST", "relatedInformation", "solid/checkSnapshot", "Replace unsafe"} {
		if !bytes.Contains(output.Bytes(), []byte(token)) {
			t.Errorf("output missing %q: %s", token, encoded)
		}
	}
}

func TestCodeActionExposesTheMultiEditComponentPropsFixToEditors(t *testing.T) {
	root := t.TempDir()
	path := filepath.Join(root, "Card.tsx")
	source := []byte("export function Card({ title }: Props) {\n  return <h1>{title}</h1>;\n}\n")
	patternStart := bytes.Index(source, []byte("{ title }"))
	readStart := bytes.LastIndex(source, []byte("title"))
	primary := certification.SourceLocation{Path: path, StartByte: patternStart, EndByte: patternStart + len("{ title }")}
	snapshot, err := certification.NewSnapshot([]certification.Finding{{
		ID: "SC1003", Rule: "component-props-destructure", Kind: certification.FindingViolation,
		Severity: certification.SeverityError, Message: "keep props intact", PrimaryLocation: &primary,
		Fixes: []certification.Fix{{
			Message: "Fix: Keep component props reactive", Applicability: certification.FixSafe,
			Edits: []certification.TextEdit{
				{Location: primary, NewText: "props"},
				{Location: certification.SourceLocation{Path: path, StartByte: readStart, EndByte: readStart + len("title")}, NewText: "props.title"},
			},
		}},
	}}, nil, certification.Metrics{})
	if err != nil {
		t.Fatal(err)
	}
	server := &Server{
		options:   Options{ReadFile: os.ReadFile},
		documents: map[string]document{path: {source: source, version: 1}},
		snapshot:  snapshot,
	}
	actions, err := server.codeActions(codeActionParams{TextDocument: textDocumentIdentifier{URI: pathToURI(path)}})
	if err != nil {
		t.Fatal(err)
	}
	if len(actions) != 1 {
		t.Fatalf("actions = %#v", actions)
	}
	action := actions[0]
	if action.Title != "Fix: Keep component props reactive" {
		t.Fatalf("action title = %q", action.Title)
	}
	if action.Kind != "quickfix" || !action.IsPreferred || action.Data.(map[string]any)["findingId"] != "SC1003" {
		t.Fatalf("action metadata = %#v", action)
	}
	edits := action.Edit.Changes[pathToURI(path)]
	if len(edits) != 2 || edits[0].NewText != "props" || edits[1].NewText != "props.title" {
		t.Fatalf("workspace edits = %#v", edits)
	}
}

func TestBytePositionUsesUTF16Coordinates(t *testing.T) {
	source := []byte("😀x\n語z")
	if got := bytePosition(source, len([]byte("😀x\n語"))); got != (position{Line: 1, Character: 1}) {
		t.Fatalf("position = %#v, want line 1 character 1", got)
	}
	if got := bytePosition(source, len([]byte("😀"))); got != (position{Line: 0, Character: 2}) {
		t.Fatalf("astral position = %#v, want two UTF-16 units", got)
	}
}

type memoryEngine struct{ files map[string][]byte }

func (e *memoryEngine) OpenProject(context.Context, engine.ProjectConfig) (engine.ProjectSession, error) {
	return &memorySession{files: cloneFiles(e.files)}, nil
}

type memorySession struct {
	files   map[string][]byte
	version uint64
}

func (s *memorySession) Update(_ context.Context, changes []engine.FileChange) (engine.AnalysisDelta, error) {
	paths := make([]string, 0, len(changes))
	for _, change := range changes {
		path, _ := filepath.Abs(change.Path)
		if change.Deleted {
			delete(s.files, path)
		} else {
			s.files[path] = append([]byte(nil), change.Source...)
		}
		paths = append(paths, path)
	}
	s.version++
	return engine.AnalysisDelta{Version: s.version, AffectedPaths: paths}, nil
}
func (s *memorySession) Snapshot(context.Context, *engine.AnalysisScope) (certification.Snapshot, error) {
	var findings []certification.Finding
	paths := make([]string, 0, len(s.files))
	for path := range s.files {
		paths = append(paths, path)
	}
	sort.Strings(paths)
	for _, path := range paths {
		source := s.files[path]
		for offset := 0; ; {
			index := bytes.Index(source[offset:], []byte("unsafe"))
			if index < 0 {
				break
			}
			start := offset + index
			location := certification.SourceLocation{Path: path, StartByte: start, EndByte: start + 6, Line: 1, Column: start + 1}
			findings = append(findings, certification.Finding{ID: "SC-TEST", Rule: "test", Kind: certification.FindingViolation, Severity: certification.SeverityError, Message: "unsafe marker", PrimaryLocation: &location, RelatedLocations: []certification.SourceLocation{location}, Evidence: []certification.EvidenceStep{{Message: "marker evidence", Location: &location}}, Fixes: []certification.Fix{{Message: "Replace unsafe", Applicability: certification.FixSafe, Edits: []certification.TextEdit{{Location: location, NewText: "safe"}}}}})
			offset = start + 6
		}
	}
	return certification.NewSnapshot(findings, nil, certification.Metrics{FilesAnalyzed: len(s.files), ProofObligations: len(findings)})
}
func (*memorySession) Close() error { return nil }
func cloneFiles(files map[string][]byte) map[string][]byte {
	cloned := map[string][]byte{}
	for path, source := range files {
		cloned[path] = append([]byte(nil), source...)
	}
	return cloned
}
func mustJSON(value any) json.RawMessage { encoded, _ := json.Marshal(value); return encoded }
func decodeFrames(t *testing.T, data []byte) []json.RawMessage {
	t.Helper()
	var frames []json.RawMessage
	reader := bytes.NewReader(data)
	for reader.Len() > 0 {
		payload, err := readMessage(reader)
		if err != nil {
			t.Fatal(err)
		}
		frames = append(frames, payload)
	}
	return frames
}
func writeMessage(writer io.Writer, value any) error {
	payload, err := json.Marshal(value)
	if err != nil {
		return err
	}
	_, err = fmt.Fprintf(writer, "Content-Length: %d\r\n\r\n%s", len(payload), payload)
	return err
}
