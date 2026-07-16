package engine_test

import (
	"context"
	"errors"
	"path/filepath"
	"testing"

	"github.com/yumemi-thomas/solid-check/internal/engine"
	"github.com/yumemi-thomas/solid-check/internal/typefacts/tsgo"
	"github.com/yumemi-thomas/solid-check/pkg/certification"
)

func TestNativeSessionUsesTypeFactsForProjectUpdates(t *testing.T) {
	fixture := filepath.Join("..", "typefacts", "testdata", "aliased-import")
	session, err := (engine.NativeEngine{OpenTypeFacts: tsgo.OpenProject}).OpenProject(
		context.Background(),
		engine.ProjectConfig{ConfigPath: filepath.Join(fixture, "tsconfig.json")},
	)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = session.Close() })

	delta, err := session.Update(context.Background(), []engine.FileChange{{
		Path:    filepath.Join(fixture, "source.ts"),
		Version: 1,
		Source:  []byte("export const count = () => \"updated\";\n"),
	}})
	if err != nil {
		t.Fatal(err)
	}
	if delta.Version != 1 {
		t.Errorf("analysis version = %d, want 1", delta.Version)
	}
	wantNames := []string{"consumer.ts", "source.ts", "use.ts"}
	if len(delta.AffectedPaths) != len(wantNames) {
		t.Fatalf("affected paths = %#v, want %v", delta.AffectedPaths, wantNames)
	}
	for i, path := range delta.AffectedPaths {
		if filepath.Base(path) != wantNames[i] {
			t.Errorf("affected path %d = %q, want suffix %q", i, path, wantNames[i])
		}
	}

	snapshot, err := session.Snapshot(context.Background(), nil)
	if err != nil {
		t.Fatal(err)
	}
	if snapshot.Status != certification.StatusUncertifiable {
		t.Errorf("snapshot status = %q, want uncertifiable", snapshot.Status)
	}
	if len(snapshot.Findings) != 1 || snapshot.Findings[0].Rule != "execution-map-unavailable" {
		t.Errorf("snapshot findings = %#v", snapshot.Findings)
	}
	if err := session.Close(); err != nil {
		t.Fatal(err)
	}
	if _, err := session.Snapshot(context.Background(), nil); !errors.Is(err, engine.ErrSessionClosed) {
		t.Errorf("snapshot after close error = %v, want ErrSessionClosed", err)
	}
}
