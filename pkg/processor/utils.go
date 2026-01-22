package processor

import (
	"regexp"
	"strings"
)

// RemoveSQLComments removes both single-line (--) and multi-line (/* */) SQL comments.
func RemoveSQLComments(content string) string {
	// Remove multi-line comments /* ... */
	reMulti := regexp.MustCompile(`(?s)/\*.*?\*/`)
	content = reMulti.ReplaceAllString(content, "")

	// Remove single-line comments -- ...
	reSingle := regexp.MustCompile(`--.*`)
	content = reSingle.ReplaceAllString(content, "")

	return content
}

// IsPLSQL determines if the given SQL content likely contains PL/SQL code.
func IsPLSQL(content string) bool {
	upperContent := strings.ToUpper(content)
	trimmed := strings.TrimSpace(upperContent)

	// If it ends with END; or END, it's very likely PL/SQL
	if strings.HasSuffix(trimmed, "END;") || strings.HasSuffix(trimmed, "END") {
		return true
	}

	// Check for other common PL/SQL indicators
	// We only want to match things that REQUIRE a slash (/) in SQL*Plus/Command Window.
	indicators := []string{
		`\bBEGIN\b`,
		`\bDECLARE\b`,
		// These objects in Oracle require a slash to execute in SQL*Plus
		`\bCREATE\s+(OR\s+REPLACE\s+)?(PROCEDURE|FUNCTION|PACKAGE|TRIGGER|TYPE|BODY|LIBRARY)\b`,
	}
	for _, ind := range indicators {
		if regexp.MustCompile("(?i)"+ind).MatchString(content) {
			return true
		}
	}

	return false
}

// splitSQLIgnoringComments splits a SQL script into individual statements using semicolons,
// while being careful not to split inside comments.
func splitSQLIgnoringComments(sql string) []string {
	var result []string
	var current strings.Builder

	inSingleComment := false
	inMultiComment := false
	inString := false

	runes := []rune(sql)
	for i := 0; i < len(runes); i++ {
		r := runes[i]

		if inString {
			current.WriteRune(r)
			if r == '\'' {
				if i+1 < len(runes) && runes[i+1] == '\'' {
					current.WriteRune('\'')
					i++
				} else {
					inString = false
				}
			}
			continue
		}

		if inSingleComment {
			current.WriteRune(r)
			if r == '\n' {
				inSingleComment = false
			}
			continue
		}

		if inMultiComment {
			current.WriteRune(r)
			if r == '*' && i+1 < len(runes) && runes[i+1] == '/' {
				current.WriteRune('/')
				i++
				inMultiComment = false
			}
			continue
		}

		// Check for comment start
		if r == '-' && i+1 < len(runes) && runes[i+1] == '-' {
			inSingleComment = true
			current.WriteRune('-')
			current.WriteRune('-')
			i++
			continue
		}

		if r == '/' && i+1 < len(runes) && runes[i+1] == '*' {
			inMultiComment = true
			current.WriteRune('/')
			current.WriteRune('*')
			i++
			continue
		}

		if r == '\'' {
			inString = true
			current.WriteRune(r)
			continue
		}

		if r == ';' {
			result = append(result, current.String())
			current.Reset()
			continue
		}

		current.WriteRune(r)
	}

	if current.Len() > 0 {
		result = append(result, current.String())
	}

	return result
}

// countLines counts the number of lines up to a certain position in a string.
func countLines(s string, pos int) int {
	if pos > len(s) {
		pos = len(s)
	}
	return strings.Count(s[:pos], "\n") + 1
}