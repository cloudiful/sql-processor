package processor

import (
	"strings"
	"testing"
)

func TestCheckAndFixSchema_Injection(t *testing.T) {
	tests := []struct {
		name         string
		input        string
		targetSchema string
		expected     string
	}{
		{
			name:         "Basic Create Table",
			input:        "CREATE TABLE MyTable (id NUMBER);",
			targetSchema: "EBANK",
			expected:     "CREATE TABLE EBANK.MyTable (id NUMBER);",
		},
		{
			name:         "Drop View",
			input:        "DROP VIEW OldView;",
			targetSchema: "EBANK",
			expected:     "DROP VIEW EBANK.OldView;",
		},
		{
			name:         "Already Qualified",
			input:        "CREATE TABLE EBANK.MyTable (id NUMBER);",
			targetSchema: "EBANK",
			expected:     "CREATE TABLE EBANK.MyTable (id NUMBER);",
		},
		{
			name:         "Case Insensitivity",
			input:        "create table mytable (id number);",
			targetSchema: "EBANK",
			expected:     "create table EBANK.mytable (id number);",
		},
		{
			name:         "Create Index On Table",
			input:        "CREATE INDEX idx_1 ON MyTable(col1);",
			targetSchema: "EBANK",
			expected:     "CREATE INDEX EBANK.idx_1 ON EBANK.MyTable(col1);",
		},
		{
			name:         "Create Unique Index",
			input:        "CREATE UNIQUE INDEX idx_u ON MyTable(col1);",
			targetSchema: "EBANK",
			expected:     "CREATE UNIQUE INDEX EBANK.idx_u ON EBANK.MyTable(col1);",
		},
		{
			name:         "Grant On Table",
			input:        "GRANT SELECT ON MyTable TO UserB;",
			targetSchema: "EBANK",
			expected:     "GRANT SELECT ON EBANK.MyTable TO UserB;",
		},
		{
			name:         "Package Body",
			input:        "CREATE OR REPLACE PACKAGE BODY MyPkg AS ... END;",
			targetSchema: "EBANK",
			expected:     "CREATE OR REPLACE PACKAGE BODY EBANK.MyPkg AS ... END;",
		},
		{
			name:         "Package Spec",
			input:        "CREATE OR REPLACE PACKAGE MyPkg AS ... END;",
			targetSchema: "EBANK",
			expected:     "CREATE OR REPLACE PACKAGE EBANK.MyPkg AS ... END;",
		},
		{
			name:         "Drop Sequence",
			input:        "DROP SEQUENCE seq_1;",
			targetSchema: "EBANK",
			expected:     "DROP SEQUENCE EBANK.seq_1;",
		},
		{
			name:         "Create Type",
			input:        "CREATE TYPE MyType AS OBJECT ...",
			targetSchema: "EBANK",
			expected:     "CREATE TYPE EBANK.MyType AS OBJECT ...",
		},
		{
			name:         "Double Quoted Identifier",
			input:        `CREATE TABLE "MyTable" (id NUMBER);`,
			targetSchema: "EBANK",
			expected:     `CREATE TABLE EBANK."MyTable" (id NUMBER);`,
		},
		{
			name:         "Truncate Table",
			input:        "TRUNCATE TABLE MyTable;",
			targetSchema: "EBANK",
			expected:     "TRUNCATE TABLE EBANK.MyTable;",
		},
		{
			name:         "Alter Table",
			input:        "ALTER TABLE MyTable ADD (col2 NUMBER);",
			targetSchema: "EBANK",
			expected:     "ALTER TABLE EBANK.MyTable ADD (col2 NUMBER);",
		},
		{
			name:         "No Target Schema",
			input:        "CREATE TABLE MyTable (id NUMBER);",
			targetSchema: "",
			expected:     "CREATE TABLE MyTable (id NUMBER);",
		},
		{
			name:         "Join ON (should not inject)",
			input:        "SELECT * FROM table_a JOIN table_b ON table_a.id = table_b.id",
			targetSchema: "EBANK",
			expected:     "SELECT * FROM table_a JOIN table_b ON table_a.id = table_b.id",
		},
		{
			name:         "Identifier at start (no prev token)",
			input:        "random_id",
			targetSchema: "EBANK",
			expected:     "random_id",
		},
		{
			name:         "Comment and space before dot",
			input:        "CREATE TABLE MySchema /* comment */ . MyTable (id NUMBER);",
			targetSchema: "EBANK",
			expected:     "CREATE TABLE MySchema /* comment */ . MyTable (id NUMBER);",
		},
		{
			name:         "Semicolon before ON",
			input:        "SELECT 1; ON MyTable",
			targetSchema: "EBANK",
			expected:     "SELECT 1; ON MyTable",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, _, err := CheckAndFixSchema(tt.input, tt.targetSchema)
			if err != nil {
				t.Fatalf("Unexpected error: %v", err)
			}
			if got != tt.expected {
				t.Errorf("CheckAndFixSchema() \nGot: %s\nWant: %s", got, tt.expected)
			}
		})
	}
}

func TestCheckAndFixSchema_DMLWarnings(t *testing.T) {
	tests := []struct {
		name         string
		input        string
		targetSchema string
		wantWarning  string
	}{
		{
			name:         "Select Statement",
			input:        "SELECT * FROM MyTable;",
			targetSchema: "EBANK",
			wantWarning:  "DML statement 'SELECT' detected",
		},
		{
			name:         "Insert Statement",
			input:        "INSERT INTO MyTable VALUES (1);",
			targetSchema: "EBANK",
			wantWarning:  "DML statement 'INSERT' detected",
		},
		{
			name:         "Update Statement",
			input:        "UPDATE MyTable SET col1 = 1;",
			targetSchema: "EBANK",
			wantWarning:  "DML statement 'UPDATE' detected",
		},
		{
			name:         "Delete Statement",
			input:        "DELETE FROM MyTable;",
			targetSchema: "EBANK",
			wantWarning:  "DML statement 'DELETE' detected",
		},
		{
			name:         "Merge Statement",
			input:        "MERGE INTO target USING source ON ...",
			targetSchema: "EBANK",
			wantWarning:  "DML statement 'MERGE' detected",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, warnings, err := CheckAndFixSchema(tt.input, tt.targetSchema)
			if err != nil {
				t.Fatalf("Unexpected error: %v", err)
			}
			if len(warnings) == 0 {
				t.Errorf("Expected warning containing %q, got none", tt.wantWarning)
				return
			}
			found := false
			for _, w := range warnings {
				if strings.Contains(w, tt.wantWarning) {
					found = true
					break
				}
			}
			if !found {
				t.Errorf("Expected warning containing %q, got: %v", tt.wantWarning, warnings)
			}
		})
	}
}