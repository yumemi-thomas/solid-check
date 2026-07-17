// Package lsp adapts certification project sessions to the Language Server Protocol.
// It contains presentation and transport logic only; all analysis remains in engine.
package lsp

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/url"
	"path/filepath"
	"strconv"
	"strings"
)

type request struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      json.RawMessage `json:"id,omitempty"`
	Method  string          `json:"method"`
	Params  json.RawMessage `json:"params,omitempty"`
}

type response struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      json.RawMessage `json:"id"`
	Result  any             `json:"result,omitempty"`
	Error   *responseError  `json:"error,omitempty"`
}

type responseError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}
type notification struct {
	JSONRPC string `json:"jsonrpc"`
	Method  string `json:"method"`
	Params  any    `json:"params"`
}

type initializeParams struct {
	InitializationOptions *initializationOptions `json:"initializationOptions,omitempty"`
}
type initializationOptions struct {
	Project   string   `json:"project,omitempty"`
	Contracts []string `json:"contracts,omitempty"`
}
type textDocumentIdentifier struct {
	URI string `json:"uri"`
}
type versionedTextDocumentIdentifier struct {
	URI     string `json:"uri"`
	Version int    `json:"version"`
}
type textDocumentItem struct {
	URI     string `json:"uri"`
	Version int    `json:"version"`
	Text    string `json:"text"`
}
type didOpenParams struct {
	TextDocument textDocumentItem `json:"textDocument"`
}
type didChangeParams struct {
	TextDocument   versionedTextDocumentIdentifier  `json:"textDocument"`
	ContentChanges []textDocumentContentChangeEvent `json:"contentChanges"`
}
type textDocumentContentChangeEvent struct {
	Text string `json:"text"`
}
type didCloseParams struct {
	TextDocument textDocumentIdentifier `json:"textDocument"`
}
type explainParams struct {
	FindingID string `json:"findingId"`
	URI       string `json:"uri,omitempty"`
	StartByte *int   `json:"startByte,omitempty"`
}
type codeActionParams struct {
	TextDocument textDocumentIdentifier `json:"textDocument"`
}

type position struct {
	Line      int `json:"line"`
	Character int `json:"character"`
}
type lspRange struct {
	Start position `json:"start"`
	End   position `json:"end"`
}
type location struct {
	URI   string   `json:"uri"`
	Range lspRange `json:"range"`
}
type diagnosticRelatedInformation struct {
	Location location `json:"location"`
	Message  string   `json:"message"`
}
type diagnostic struct {
	Range              lspRange                       `json:"range"`
	Severity           int                            `json:"severity"`
	Code               string                         `json:"code"`
	Source             string                         `json:"source"`
	Message            string                         `json:"message"`
	RelatedInformation []diagnosticRelatedInformation `json:"relatedInformation,omitempty"`
	Data               any                            `json:"data,omitempty"`
}
type textEdit struct {
	Range   lspRange `json:"range"`
	NewText string   `json:"newText"`
}
type workspaceEdit struct {
	Changes map[string][]textEdit `json:"changes"`
}
type codeAction struct {
	Title       string        `json:"title"`
	Kind        string        `json:"kind"`
	IsPreferred bool          `json:"isPreferred,omitempty"`
	Edit        workspaceEdit `json:"edit"`
	Data        any           `json:"data,omitempty"`
}

func readMessage(reader io.Reader) (json.RawMessage, error) {
	length := -1
	for {
		line, err := readHeaderLine(reader)
		if err != nil {
			return nil, err
		}
		line = strings.TrimRight(line, "\r\n")
		if line == "" {
			break
		}
		name, value, found := strings.Cut(line, ":")
		if !found {
			return nil, fmt.Errorf("invalid LSP header %q", line)
		}
		if strings.EqualFold(strings.TrimSpace(name), "Content-Length") {
			parsed, err := strconv.Atoi(strings.TrimSpace(value))
			if err != nil || parsed < 0 {
				return nil, errors.New("invalid Content-Length")
			}
			length = parsed
		}
	}
	if length < 0 {
		return nil, errors.New("missing Content-Length")
	}
	payload := make([]byte, length)
	if _, err := io.ReadFull(reader, payload); err != nil {
		return nil, err
	}
	return payload, nil
}

func readHeaderLine(reader io.Reader) (string, error) {
	var line []byte
	var one [1]byte
	for {
		_, err := io.ReadFull(reader, one[:])
		if err != nil {
			return "", err
		}
		line = append(line, one[0])
		if one[0] == '\n' {
			return string(line), nil
		}
		if len(line) > 8192 {
			return "", errors.New("LSP header line too long")
		}
	}
}

func writeFrame(writer io.Writer, value any) error {
	payload, err := json.Marshal(value)
	if err != nil {
		return err
	}
	if _, err = fmt.Fprintf(writer, "Content-Length: %d\r\n\r\n", len(payload)); err != nil {
		return err
	}
	_, err = writer.Write(payload)
	return err
}

func pathToURI(path string) string {
	absolute, _ := filepath.Abs(path)
	return (&url.URL{Scheme: "file", Path: filepath.ToSlash(absolute)}).String()
}
func uriToPath(uri string) (string, error) {
	parsed, err := url.Parse(uri)
	if err != nil {
		return "", err
	}
	if parsed.Scheme != "file" {
		return "", fmt.Errorf("unsupported document URI scheme %q", parsed.Scheme)
	}
	return filepath.Clean(filepath.FromSlash(parsed.Path)), nil
}

func rawID(id json.RawMessage) bool { return len(bytes.TrimSpace(id)) != 0 }
