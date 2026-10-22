package skills

import (
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	translationassets "github.com/voocel/ainovel-cli/assets/translation"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func isolatedLibrary(t *testing.T) {
	t.Helper()
	home := t.TempDir()
	t.Setenv("HOME", home)
	t.Setenv("USERPROFILE", home)
}

func skillWorkspace(t *testing.T, id string) workspace.Config {
	t.Helper()
	c, err := workspace.Create(t.TempDir(), id, id)
	if err != nil {
		t.Fatal(err)
	}
	if err := InstallDefaults(c); err != nil {
		t.Fatal(err)
	}
	return c
}

func TestWorkspacesShareCoreFilesAndKeepSeriesInstructionsPrivate(t *testing.T) {
	isolatedLibrary(t)
	a, b := skillWorkspace(t, "series-a"), skillWorkspace(t, "series-b")
	first, err := Load(a)
	if err != nil {
		t.Fatal(err)
	}
	second, err := Load(b)
	if err != nil {
		t.Fatal(err)
	}
	for i := range a.EnabledSkills {
		if first.Skills[i].Path != second.Skills[i].Path {
			t.Fatal("core skill copied per workspace")
		}
		if _, err := os.Stat(a.ReadPath(filepath.Join("Skills", "core"))); !os.IsNotExist(err) {
			t.Fatal("new workspace still has core copies")
		}
	}
	pos := domain.Position{Volume: 1}
	oldA, oldB := first.Fingerprint(pos, "猫"), second.Fingerprint(pos, "猫")
	p := first.Skills[0].Path
	data, err := os.ReadFile(p)
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(p, append(data, []byte("\nShared edit for every series.\n")...), 0600); err != nil {
		t.Fatal(err)
	}
	first, err = Load(a)
	if err != nil {
		t.Fatal(err)
	}
	second, err = Load(b)
	if err != nil {
		t.Fatal(err)
	}
	if first.Fingerprint(pos, "猫") == oldA || second.Fingerprint(pos, "猫") == oldB {
		t.Fatal("shared edit did not affect both workspaces")
	}
	unchangedB := second.Fingerprint(pos, "猫")
	private := a.ReadPath(a.SeriesSkill)
	data, err = os.ReadFile(private)
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(private, append(data, []byte("\nKeep honorifics for series A.\n")...), 0600); err != nil {
		t.Fatal(err)
	}
	second, err = Load(b)
	if err != nil || second.Fingerprint(pos, "猫") != unchangedB {
		t.Fatal("private convention leaked into another workspace", err)
	}
}

func TestMovingUnchangedDefaultsPreservesHistoricalFingerprint(t *testing.T) {
	isolatedLibrary(t)
	c := skillWorkspace(t, "legacy")
	current, err := Load(c)
	if err != nil {
		t.Fatal(err)
	}
	var legacy []Skill
	for _, id := range c.EnabledSkills {
		data, err := translationassets.Files.ReadFile("skills/" + id + ".md")
		if err != nil {
			t.Fatal(err)
		}
		path := c.ReadPath(filepath.Join("Skills", "core", id, "SKILL.md"))
		sk, err := Parse(path, data)
		if err != nil {
			t.Fatal(err)
		}
		legacy = append(legacy, sk)
	}
	data, err := os.ReadFile(c.ReadPath(c.SeriesSkill))
	if err != nil {
		t.Fatal(err)
	}
	sk, err := Parse(c.ReadPath(c.SeriesSkill), data)
	if err != nil {
		t.Fatal(err)
	}
	sk.Priority = 100
	legacy = append(legacy, sk)
	if got := current.Fingerprint(domain.Position{Volume: 1}, "猫"); got != domain.Digest(legacy) {
		t.Fatal("moving identical defaults would invalidate saved translations")
	}
	// The identity survives snapshot serialization, not just the in-memory load.
	if err := SaveSnapshot(c, current); err != nil {
		t.Fatal(err)
	}
	var restored Snapshot
	data, err = os.ReadFile(filepath.Join(c.Root, ".translation_work", "skill_snapshots", current.Digest, "snapshot.json"))
	if err != nil {
		t.Fatal(err)
	}
	if err := json.Unmarshal(data, &restored); err != nil {
		t.Fatal(err)
	}
	if restored.Fingerprint(domain.Position{Volume: 1}, "猫") != domain.Digest(legacy) {
		t.Fatal("stored snapshot lost stable identity")
	}
}

func TestSharedSelectionPersistsAndNeverOverwritesExistingSkills(t *testing.T) {
	isolatedLibrary(t)
	c := skillWorkspace(t, "series")
	directory := filepath.Join(t.TempDir(), "Shared skills có dấu")
	if err := SetSharedDirectory(context.Background(), directory); err != nil {
		t.Fatal(err)
	}
	s, err := Load(c)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.HasPrefix(s.Skills[0].Path, directory+string(filepath.Separator)) {
		t.Fatal("shared setting not applied to existing workspace")
	}
	data, err := os.ReadFile(s.Skills[0].Path)
	if err != nil {
		t.Fatal(err)
	}
	changed := append(data, []byte("\nPreserve this shared customization.\n")...)
	if err := os.WriteFile(s.Skills[0].Path, changed, 0600); err != nil {
		t.Fatal(err)
	}
	if err := InstallDefaults(c); err != nil {
		t.Fatal(err)
	}
	actual, err := os.ReadFile(s.Skills[0].Path)
	if err != nil || string(actual) != string(changed) {
		t.Fatal("shared customization overwritten", err)
	}
	if err := SaveSharedSkill(context.Background(), directory, s.Skills[0].ID, domain.Hash(data), string(data)); err == nil {
		t.Fatal("concurrent shared edit overwritten")
	}
}

func TestSharedReferencesResolveFromActualLibraryAndPrivateRulesWin(t *testing.T) {
	isolatedLibrary(t)
	c := skillWorkspace(t, "series")
	directory, err := SharedDirectory()
	if err != nil {
		t.Fatal(err)
	}
	p := filepath.Join(directory, "jp-vn-translation", "SKILL.md")
	if err := os.WriteFile(p, []byte("---\nid: jp-vn-translation\npriority: 900\nreferences: [notes.md]\n---\nShared default.\n"), 0600); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(filepath.Dir(p), "notes.md"), []byte("| 猫 | shared reference. |"), 0600); err != nil {
		t.Fatal(err)
	}
	s, err := Load(c)
	if err != nil {
		t.Fatal(err)
	}
	if s.Skills[0].Priority >= 100 {
		t.Fatal("shared priority can override series convention")
	}
	core, refs, _, err := s.Prompt(domain.Position{Volume: 1}, "猫", 16000)
	if err != nil || len(refs) != 1 || refs[0].Text != "| 猫 | shared reference. |" {
		t.Fatal("reference resolved against virtual old workspace path", err)
	}
	if !strings.Contains(core, "override conflicting shared defaults") {
		t.Fatal("private precedence absent from prompt")
	}
}
