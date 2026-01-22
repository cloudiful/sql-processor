package processor

import (
	"fmt"
	"regexp"
	"strings"
)

// ProcessSQLText processes a UTF-8 SQL string, applying schema injection, idempotency wrapping,
// and missing COMMIT/slash injection. It returns the processed SQL, a list of informative messages, and any error.
func ProcessSQLText(content string, targetSchema string) (string, []string, error) {
	var messages []string

	if targetSchema != "" {
		newContent, warnings, err := CheckAndFixSchema(content, targetSchema)
		if err != nil {
			return "", nil, fmt.Errorf("failed to inject schema: %w", err)
		}
		for _, w := range warnings {
			messages = append(messages, fmt.Sprintf("Warning: %s", w))
		}
		if newContent != content {
			messages = append(messages, fmt.Sprintf("Processed for schema injection (Target: %s)", targetSchema))
			content = newContent
		}
	}

	trimmed := strings.TrimSpace(content)
	if trimmed == "" {
		return content, messages, nil
	}

	// 1. Check if we can make this idempotent (CREATE/DROP INDEX/TABLE/VIEW/SEQUENCE/SYNONYM)
	wrappedContent, modified := TryMakeIdempotent(content)
	if modified {
		messages = append(messages, "Processed for idempotency")
		content = wrappedContent
	}

	// 2. Remove comments for analysis
	cleanContent := RemoveSQLComments(content)
	trimmedContent := strings.TrimSpace(content)

	// Determine if the script needs a COMMIT or a slash (/) based on the last functional statement.
	isPL := IsPLSQL(cleanContent)
	if isPL && !strings.HasSuffix(trimmedContent, "/") {
		lastSlash := strings.LastIndex(content, "/")
		var lastPart string
		if lastSlash != -1 {
			lastPart = content[lastSlash+1:]
		} else {
			lastPart = content
		}

		cleanLast := RemoveSQLComments(lastPart)
		if strings.TrimSpace(cleanLast) != "" && !IsPLSQL(cleanLast) {
			isPL = false
		}
	}

	// Flags for injection
	hasDML := regexp.MustCompile(`(?i)\b(INSERT|UPDATE|DELETE|MERGE)\b`).MatchString(cleanContent)
	hasDDL := regexp.MustCompile(`(?i)\b(CREATE|DROP|ALTER|TRUNCATE)\b`).MatchString(cleanContent)
	hasCommit := regexp.MustCompile(`(?i)\bCOMMIT\b`).MatchString(cleanContent)
	needsSlash := isPL && !strings.HasSuffix(trimmedContent, "/")

	var toAdd strings.Builder

	if needsSlash {
		messages = append(messages, "Missing slash (PL/SQL detected)")
		if !strings.HasSuffix(content, "\n") {
			toAdd.WriteString("\n")
		}
		toAdd.WriteString("/\n")
	}

	if !hasCommit && (hasDML || (hasDDL && !isPL)) {
		messages = append(messages, "Missing commit command")
		if toAdd.Len() > 0 {
			if !strings.HasSuffix(toAdd.String(), "\n") {
				toAdd.WriteString("\n")
			}
		} else if !strings.HasSuffix(content, "\n") {
			toAdd.WriteString("\n")
		}
		toAdd.WriteString("-- [AUTO-GENERATED] Commit added for DML\nCOMMIT;\n")
	}

	toAddStr := toAdd.String()
	finalContent := content + toAddStr
	if modified || toAddStr != "" {
		finalContent = "-- [AUTO-GENERATED] This file has been automatically processed for idempotency or missing COMMIT/slash\n" + finalContent
	}

	if !strings.HasSuffix(finalContent, "\n") {
		finalContent += "\n"
	}

	return finalContent, messages, nil
}