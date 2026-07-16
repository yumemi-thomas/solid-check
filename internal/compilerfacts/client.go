package compilerfacts

import (
	"bufio"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"sync"
)

var ErrClientClosed = errors.New("compiler facts client is closed")

type ProcessConfig struct {
	Executable string
	Args       []string
	Env        []string
}

// Client serializes requests over one persistent JSON-lines sidecar process.
type Client struct {
	mu      sync.Mutex
	command *exec.Cmd
	input   io.WriteCloser
	encoder *json.Encoder
	decoder *json.Decoder
	closed  bool
}

func Start(ctx context.Context, config ProcessConfig) (*Client, error) {
	if config.Executable == "" {
		return nil, errors.New("compiler facts sidecar executable is required")
	}
	command := exec.CommandContext(ctx, config.Executable, config.Args...)
	command.Env = append(os.Environ(), config.Env...)
	input, err := command.StdinPipe()
	if err != nil {
		return nil, fmt.Errorf("open compiler facts stdin: %w", err)
	}
	output, err := command.StdoutPipe()
	if err != nil {
		_ = input.Close()
		return nil, fmt.Errorf("open compiler facts stdout: %w", err)
	}
	if err := command.Start(); err != nil {
		_ = input.Close()
		return nil, fmt.Errorf("start compiler facts sidecar: %w", err)
	}
	return &Client{
		command: command,
		input:   input,
		encoder: json.NewEncoder(input),
		decoder: json.NewDecoder(bufio.NewReader(output)),
	}, nil
}

type sidecarResponse struct {
	OK           bool         `json:"ok"`
	ExecutionMap ExecutionMap `json:"executionMap"`
	Error        struct {
		Code    string `json:"code"`
		Message string `json:"message"`
	} `json:"error"`
}

func (c *Client) Analyze(ctx context.Context, request AnalysisRequest) (ExecutionMap, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.closed {
		return ExecutionMap{}, ErrClientClosed
	}
	if err := ctx.Err(); err != nil {
		return ExecutionMap{}, err
	}
	if err := c.encoder.Encode(request); err != nil {
		return ExecutionMap{}, fmt.Errorf("send compiler facts request: %w", err)
	}
	var response sidecarResponse
	if err := c.decoder.Decode(&response); err != nil {
		return ExecutionMap{}, fmt.Errorf("read compiler facts response: %w", err)
	}
	if !response.OK {
		return ExecutionMap{}, fmt.Errorf("compiler facts %s: %s", response.Error.Code, response.Error.Message)
	}
	if err := Validate(request, response.ExecutionMap); err != nil {
		return ExecutionMap{}, fmt.Errorf("validate compiler facts response: %w", err)
	}
	return response.ExecutionMap, nil
}

func (c *Client) Close() error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.closed {
		return ErrClientClosed
	}
	c.closed = true
	closeErr := c.input.Close()
	waitErr := c.command.Wait()
	if closeErr != nil {
		return closeErr
	}
	if waitErr != nil {
		return fmt.Errorf("compiler facts sidecar exited: %w", waitErr)
	}
	return nil
}
