package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRun(t *testing.T) {
	tests := []struct {
		name         string
		args         []string
		input        string
		wantStdout   string
		wantStderr   string
		wantErr      bool
	}{
		{
			name:       "basic schema injection",
			args:       []string{"-schema", "TEST_SCHEMA"},
			input:      "CREATE TABLE MY_TABLE (ID NUMBER);",
			wantStdout: "CREATE TABLE TEST_SCHEMA.MY_TABLE",
			wantStderr: "[INFO] Processed for schema injection",
		},
		{
			name:       "DML warning",
			args:       []string{"-schema", "TEST_SCHEMA"},
			input:      "INSERT INTO T1 VALUES (1);",
			wantStderr: "DML statement 'INSERT' detected",
		},
		{
			name:       "empty input",
			input:      "",
			wantStdout: "",
		},
		{
			name:       "invalid flag",
			args:       []string{"-unknown-flag"},
			wantErr:    true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			stdin := bytes.NewBufferString(tt.input)
			stdout := &bytes.Buffer{}
			stderr := &bytes.Buffer{}

			err := run(tt.args, stdin, stdout, stderr)

			if (err != nil) != tt.wantErr {
				t.Errorf("run() error = %v, wantErr %v", err, tt.wantErr)
				return
			}

			if tt.wantStdout != "" && !strings.Contains(stdout.String(), tt.wantStdout) {
				t.Errorf("stdout got = %q, want to contain %q", stdout.String(), tt.wantStdout)
			}

			if tt.wantStderr != "" && !strings.Contains(stderr.String(), tt.wantStderr) {
				t.Errorf("stderr got = %q, want to contain %q", stderr.String(), tt.wantStderr)
			}
		})
	}
}

func TestRun_Files(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "sql-proc-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	inputPath := filepath.Join(tmpDir, "input.sql")
	outputPath := filepath.Join(tmpDir, "output.sql")
	content := "CREATE TABLE T1 (ID NUMBER);"

	if err := os.WriteFile(inputPath, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	args := []string{"-schema", "EBANK", "-input", inputPath, "-output", outputPath}
	err = run(args, nil, os.Stdout, os.Stderr)
	if err != nil {
		t.Fatalf("run with files failed: %v", err)
	}

	outputData, err := os.ReadFile(outputPath)
	if err != nil {
		t.Fatal(err)
	}

	if !strings.Contains(string(outputData), "EBANK.T1") {
		t.Errorf("output file missing expected content, got: %s", string(outputData))
	}
}

func TestRun_GB2312(t *testing.T) {
	// "测试" in GB18030: 0xb2 0xe2 0xca 0xd4
	input := []byte{0xb2, 0xe2, 0xca, 0xd4}
	stdin := bytes.NewBuffer(input)
	stdout := &bytes.Buffer{}
	stderr := &bytes.Buffer{}

	args := []string{"-gb2312"}
	err := run(args, stdin, stdout, stderr)
	if err != nil {
		t.Fatalf("run with gb2312 failed: %v", err)
	}

	if stdout.Len() == 0 {
		t.Error("expected output for gb2312 input")
	}
}