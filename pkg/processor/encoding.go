package processor

import (
	"bytes"
	"fmt"
	"io"
	"unicode/utf8"

	"golang.org/x/text/encoding/simplifiedchinese"
	"golang.org/x/text/transform"
)

// SQLFileData represents the content of a SQL file in memory
type SQLFileData struct {
	Path    string
	Content []byte
}

// EnsureGB2312Memory checks if data is GB2312 and converts it if necessary, returning the result.
func EnsureGB2312Memory(path string, data []byte) ([]byte, []string, error) {
	var logs []string
	// If it's valid UTF-8 and has a BOM or non-ASCII characters, convert it.
	// Otherwise, we assume it's already GB2312/ASCII.
	hasBOM := bytes.HasPrefix(data, []byte("\xef\xbb\xbf"))
	isUtf8 := utf8.Valid(data)

	// Check for non-ASCII
	hasNonASCII := false
	for _, b := range data {
		if b > 127 {
			hasNonASCII = true
			break
		}
	}

	if !hasBOM && (!isUtf8 || !hasNonASCII) {
		// Likely already GB2312 or plain ASCII
		return data, logs, nil
	}

	logs = append(logs, fmt.Sprintf("File %s is UTF-8, auto converting to GB2312 in memory...", path))

	// Remove BOM if present
	data = bytes.TrimPrefix(data, []byte("\xef\xbb\xbf"))

	// Convert UTF-8 to GB2312
	encoder := simplifiedchinese.GB18030.NewEncoder()
	gbData, err := encoder.Bytes(data)
	if err != nil {
		return nil, logs, fmt.Errorf("failed to encode to GB2312: %w", err)
	}

	return gbData, logs, nil
}

// ProcessGB2312SQLBytes takes GB2312 encoded bytes, processes them as UTF-8 SQL text,
// and returns the result back as GB2312 bytes.
func ProcessGB2312SQLBytes(path string, data []byte, targetSchema string) ([]byte, []string, error) {
	var logs []string
	// Decode GB2312 to UTF-8 string for analysis and modification
	decoder := simplifiedchinese.GB18030.NewDecoder()
	decodedData, err := io.ReadAll(transform.NewReader(bytes.NewReader(data), decoder))
	if err != nil {
		return nil, logs, fmt.Errorf("failed to decode for commit check: %w", err)
	}

	content := string(decodedData)

	finalContent, messages, err := ProcessSQLText(content, targetSchema)
	if err != nil {
		return nil, logs, err
	}

	// Map generic messages back to path-specific logs
	for _, msg := range messages {
		logs = append(logs, fmt.Sprintf("File %s: %s", path, msg))
	}

	// Encode the final UTF-8 result back to GB2312
	encoder := simplifiedchinese.GB18030.NewEncoder()
	encodedData, err := encoder.String(finalContent)
	if err != nil {
		return nil, logs, fmt.Errorf("failed to re-encode after processing: %w", err)
	}

	return []byte(encodedData), logs, nil
}