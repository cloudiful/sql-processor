package processor

import (
	"bytes"
	"testing"

	"golang.org/x/text/encoding/simplifiedchinese"
)

func TestEnsureGB2312Memory(t *testing.T) {
	tests := []struct {
		name        string
		path        string
		data        []byte
		wantConvert bool
	}{
		{
			name:        "ASCII content",
			path:        "test.sql",
			data:        []byte("SELECT * FROM table"),
			wantConvert: false,
		},
		{
			name:        "UTF-8 with BOM",
			path:        "test_bom.sql",
			data:        append([]byte("\xef\xbb\xbf"), []byte("SELECT * FROM table")...),
			wantConvert: true,
		},
		{
			name:        "UTF-8 with non-ASCII",
			path:        "test_utf8.sql",
			data:        []byte("SELECT * FROM 表"),
			wantConvert: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, logs, err := EnsureGB2312Memory(tt.path, tt.data)
			if err != nil {
				t.Fatalf("EnsureGB2312Memory() error = %v", err)
			}

			if tt.wantConvert {
				if len(logs) == 0 {
					t.Error("Expected logs for conversion, got none")
				}
				// Verify it's actually GB18030/GB2312 now by trying to decode it
				_, err := simplifiedchinese.GB18030.NewDecoder().Bytes(got)
				if err != nil {
					t.Errorf("Result is not valid GB18030: %v", err)
				}
			} else {
				if !bytes.Equal(got, tt.data) {
					t.Error("Content should not have changed for ASCII")
				}
			}
		})
	}
}

func TestProcessGB2312SQLBytes(t *testing.T) {
	// "测试" in GB18030
	gbData := []byte{0xb2, 0xe2, 0xca, 0xd4}
	path := "test_gb.sql"
	targetSchema := "MY_SCHEMA"

	got, logs, err := ProcessGB2312SQLBytes(path, gbData, targetSchema)
	if err != nil {
		t.Fatalf("ProcessGB2312SQLBytes() error = %v", err)
	}

	if len(got) == 0 {
		t.Error("Expected non-empty result")
	}

	// Verify logs contain the path
	foundPathInLogs := false
	for _, log := range logs {
		if bytes.Contains([]byte(log), []byte(path)) {
			foundPathInLogs = true
			break
		}
	}
	if !foundPathInLogs && len(logs) > 0 {
		t.Errorf("Expected path %s in logs, but not found", path)
	}
}