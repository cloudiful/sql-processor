package processor

import (
	"strings"
	"testing"
)

func TestRemoveSQLComments(t *testing.T) {
	content := `
-- Single line comment
SELECT * FROM table; /* Multi-line
comment */
`
	result := RemoveSQLComments(content)
	if strings.Contains(result, "--") || strings.Contains(result, "/*") {
		t.Errorf("Comments were not correctly removed: %s", result)
	}
	if !strings.Contains(result, "SELECT") {
		t.Errorf("Actual code was incorrectly removed")
	}
}

func TestIsPLSQL(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected bool
	}{
		{"Simple SQL", "SELECT * FROM dual;", false},
		{"Basic Block", "BEGIN NULL; END;", true},
		{"Procedure", "CREATE OR REPLACE PROCEDURE test AS BEGIN NULL; END;", true},
		{"Function", "CREATE FUNCTION get_val RETURN NUMBER AS BEGIN RETURN 1; END;", true},
		{"Trigger", "CREATE TRIGGER my_trig BEFORE INSERT ON my_table FOR EACH ROW BEGIN NULL; END;", true},
		{"Not PLSQL View", "CREATE VIEW my_view AS SELECT 1 FROM dual;", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := IsPLSQL(tt.input); got != tt.expected {
				t.Errorf("IsPLSQL() = %v, want %v for %s", got, tt.expected, tt.name)
			}
		})
	}
}

func TestSplitSQLIgnoringComments(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected []string
	}{
		{
			name:     "simple split",
			input:    "SELECT 1; SELECT 2",
			expected: []string{"SELECT 1", " SELECT 2"},
		},
		{
			name:     "semicolon in string",
			input:    "INSERT INTO t VALUES ('a;b'); SELECT 1",
			expected: []string{"INSERT INTO t VALUES ('a;b')", " SELECT 1"},
		},
		{
			name:     "semicolon in comment",
			input:    "SELECT 1; -- comment; with semicolon\nSELECT 2",
			expected: []string{"SELECT 1", " -- comment; with semicolon\nSELECT 2"},
		},
		{
			name:     "semicolon in multi-line comment",
			input:    "SELECT 1; /* comment; \n with semicolon */ SELECT 2",
			expected: []string{"SELECT 1", " /* comment; \n with semicolon */ SELECT 2"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := splitSQLIgnoringComments(tt.input)
			if len(got) != len(tt.expected) {
				t.Errorf("splitSQLIgnoringComments() returned %d parts, want %d", len(got), len(tt.expected))
				return
			}
			for i := range got {
				if got[i] != tt.expected[i] {
					t.Errorf("part %d = %q, want %q", i, got[i], tt.expected[i])
				}
			}
		})
	}
}

func TestCountLines(t *testing.T) {
	input := "line1\nline2\nline3"
	tests := []struct {
		pos      int
		expected int
	}{
		{0, 1},
		{5, 1},
		{6, 2},
		{11, 2},
		{12, 3},
		{100, 3}, // Out of bounds
	}

	for _, tt := range tests {
		if got := countLines(input, tt.pos); got != tt.expected {
			t.Errorf("countLines(pos=%d) = %d, want %d", tt.pos, got, tt.expected)
		}
	}
}