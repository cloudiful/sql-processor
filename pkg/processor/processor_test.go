package processor

import (
	"strings"
	"testing"
)

func TestProcessSQLText(t *testing.T) {
	tests := []struct {
		name         string
		input        string
		targetSchema string
		wantContains []string
	}{
		{
			name:         "Simple DML with Commit",
			input:        "INSERT INTO MY_TABLE (ID) VALUES (1);",
			targetSchema: "EBANK",
			wantContains: []string{"COMMIT;", "MY_TABLE", "Warning: Line 1: DML statement 'INSERT' detected."}, // DML is NOT auto-fixed with schema prefix
		},
		{
			name:         "DDL Idempotency",
			input:        "CREATE TABLE TEST_TABLE (ID NUMBER);",
			targetSchema: "EBANK",
			wantContains: []string{"DECLARE", "e_exists EXCEPTION", "EBANK.TEST_TABLE", "Processed for idempotency", "Processed for schema injection (Target: EBANK)"},
		},
		{
			name:         "PL/SQL Slash Injection",
			input:        "BEGIN NULL; END;",
			targetSchema: "EBANK",
			wantContains: []string{"/", "END;", "Missing slash (PL/SQL detected)"},
		},
		{
			name:         "PL/SQL with trailing Slash (no injection)",
			input:        "BEGIN NULL; END;\n/",
			targetSchema: "EBANK",
			wantContains: []string{"END;", "/"},
		},
		{
			name:         "DDL and DML mixed",
			input:        "CREATE TABLE T1 (ID NUMBER);\nINSERT INTO T1 VALUES (1);",
			targetSchema: "EBANK",
			wantContains: []string{"EBANK.T1", "COMMIT;", "Missing commit command"},
		},
		{
			name:         "Already has COMMIT",
			input:        "INSERT INTO T1 VALUES (1); COMMIT;",
			targetSchema: "EBANK",
			wantContains: []string{"INSERT", "COMMIT;"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, messages, err := ProcessSQLText(tt.input, tt.targetSchema)
			if err != nil {
				t.Fatalf("ProcessSQLText() error = %v", err)
			}

			combined := got + "\nMessages: " + strings.Join(messages, " | ")
			for _, want := range tt.wantContains {
				if !strings.Contains(combined, want) {
					t.Errorf("ProcessSQLText() output/messages missing %q\nCombined: %s", want, combined)
				}
			}
		})
	}
}

func TestProcessSQLText_Empty(t *testing.T) {
	got, _, err := ProcessSQLText("   ", "EBANK")
	if err != nil {
		t.Fatalf("ProcessSQLText failed: %v", err)
	}
	if strings.TrimSpace(got) != "" {
		t.Errorf("Expected empty output for empty input, got: %q", got)
	}
}

func TestProcessSQLText_PLSQLFollowedByDML(t *testing.T) {
	input := "CREATE OR REPLACE PROCEDURE MY_PROC AS BEGIN NULL; END;\n/\nINSERT INTO MY_LOGS VALUES ('DONE');"
	got, messages, err := ProcessSQLText(input, "EBANK")
	if err != nil {
		t.Fatalf("ProcessSQLText failed: %v", err)
	}

	if !strings.Contains(got, "COMMIT;") {
		t.Errorf("Expected COMMIT; for trailing DML, got: %s", got)
	}

	foundCommitMsg := false
	for _, m := range messages {
		if strings.Contains(m, "Missing commit command") {
			foundCommitMsg = true
			break
		}
	}
	if !foundCommitMsg {
		t.Error("Expected 'Missing commit command' message")
	}
}