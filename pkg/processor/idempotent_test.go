package processor

import (
	"strings"
	"testing"
)

func TestTryMakeIdempotent(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		modified bool
		contains []string
	}{
		{
			name:     "Create Table",
			input:    "CREATE TABLE MY_TABLE (ID NUMBER);",
			modified: true,
			contains: []string{"DECLARE", "e_exists EXCEPTION", "PRAGMA EXCEPTION_INIT(e_exists, -955)", "EXECUTE IMMEDIATE 'CREATE TABLE MY_TABLE (ID NUMBER)'"},
		},
		{
			name:     "Drop Table",
			input:    "DROP TABLE MY_TABLE;",
			modified: true,
			contains: []string{"DECLARE", "e_not_exists EXCEPTION", "PRAGMA EXCEPTION_INIT(e_not_exists, -942)", "EXECUTE IMMEDIATE 'DROP TABLE MY_TABLE'"},
		},
		{
			name:     "Create Index",
			input:    "CREATE INDEX IDX_NAME ON MY_TABLE(NAME);",
			modified: true,
			contains: []string{"-955", "CREATE INDEX IDX_NAME ON MY_TABLE(NAME)"},
		},
		{
			name:     "Drop Index",
			input:    "DROP INDEX IDX_NAME;",
			modified: true,
			contains: []string{"PRAGMA EXCEPTION_INIT(e_not_exists, -1418)", "DROP INDEX IDX_NAME"},
		},
		{
			name:     "Create Sequence",
			input:    "CREATE SEQUENCE MY_SEQ;",
			modified: true,
			contains: []string{"-955", "CREATE SEQUENCE MY_SEQ"},
		},
		{
			name:     "Drop Sequence",
			input:    "DROP SEQUENCE MY_SEQ;",
			modified: true,
			contains: []string{"-2289", "DROP SEQUENCE MY_SEQ"},
		},
		{
			name:     "Drop Public Synonym",
			input:    "DROP PUBLIC SYNONYM MY_SYN;",
			modified: true,
			contains: []string{"-1432", "DROP PUBLIC SYNONYM MY_SYN"},
		},
		{
			name:     "Drop Private Synonym",
			input:    "DROP SYNONYM MY_SYN;",
			modified: true,
			contains: []string{"-1434", "DROP SYNONYM MY_SYN"},
		},
		{
			name:     "Multi-line Statement",
			input:    "CREATE TABLE TEST (\n  ID NUMBER\n);",
			modified: true,
			contains: []string{"-- CREATE TABLE TEST (", "--   ID NUMBER", "EXECUTE IMMEDIATE 'CREATE TABLE TEST ("},
		},
		{
			name:     "Non-DDL Statement",
			input:    "INSERT INTO T VALUES (1);",
			modified: false,
			contains: []string{"INSERT INTO T VALUES (1);"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, modified := TryMakeIdempotent(tt.input)
			if modified != tt.modified {
				t.Errorf("TryMakeIdempotent() modified = %v, want %v", modified, tt.modified)
			}
			for _, s := range tt.contains {
				if !strings.Contains(got, s) {
					t.Errorf("TryMakeIdempotent() output missing %q\nGot: %s", s, got)
				}
			}
		})
	}
}

func TestTryMakeIdempotent_MultiLineCommentHeader(t *testing.T) {
	sql := `CREATE TABLE test_table (
    id NUMBER,
    name VARCHAR2(100)
);`

	result, modified := TryMakeIdempotent(sql)
	if !modified {
		t.Fatal("Expected SQL to be modified for idempotency")
	}

	lines := strings.Split(result, "\n")
	foundCommentLine := false
	for i, line := range lines {
		if strings.Contains(line, "[AUTO-GENERATED] Idempotent wrapper for:") {
			foundCommentLine = true
			// Check if following lines are commented
			if i+1 < len(lines) && !strings.HasPrefix(strings.TrimSpace(lines[i+1]), "--") {
				t.Errorf("Line after header should be a comment, got: %q", lines[i+1])
			}
			break
		}
	}

	if !foundCommentLine {
		t.Error("Expected to find [AUTO-GENERATED] comment line")
	}
}