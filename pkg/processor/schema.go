package processor

import (
	"fmt"
	"strings"

	"sql-processor/pkg/lexer"
)

// CheckAndFixSchema processes the SQL content.
// 1. It automatically prefixes the target schema to object names in DDL statements (CREATE/DROP/ALTER/TRUNCATE).
// 2. It detects DML statements (SELECT/INSERT/UPDATE/DELETE) and issues warnings, as auto-fixing DML is risky.
// Returns the modified SQL, a list of warnings, and an error if any.
func CheckAndFixSchema(sqlContent string, targetSchema string) (string, []string, error) {
	if targetSchema == "" {
		return sqlContent, nil, nil
	}

	tokens := lexer.LexSQL(sqlContent)
	var sb strings.Builder
	var warnings []string
	seenDMLWarnings := make(map[string]bool)

	dmlKeywords := map[string]bool{
		"SELECT": true, "INSERT": true, "UPDATE": true, "DELETE": true, "MERGE": true,
	}

	objectKeywords := map[string]bool{
		"TABLE": true, "VIEW": true, "SEQUENCE": true, "SYNONYM": true,
		"PROCEDURE": true, "FUNCTION": true, "PACKAGE": true, "TRIGGER": true, "TYPE": true,
	}

	for i := 0; i < len(tokens); i++ {
		token := tokens[i]

		// DML Warning Detection
		if token.Type == lexer.TokenKeyword && dmlKeywords[strings.ToUpper(token.Value)] {
			line := countLines(sqlContent, token.Pos)
			warnMsg := fmt.Sprintf("Line %d: DML statement '%s' detected. Verify schema qualification manually.", line, strings.ToUpper(token.Value))
			if !seenDMLWarnings[warnMsg] {
				warnings = append(warnings, warnMsg)
				seenDMLWarnings[warnMsg] = true
			}
		}

		inject := false

		if token.Type == lexer.TokenIdentifier {
			// Check if we need to inject schema
			// We look at the previous meaningful token
			ptIdx := prevMeaningfulTokenIdx(tokens, i)
			if ptIdx != -1 {
				pt := tokens[ptIdx]

				// Ensure not already qualified
				// Check if previous token was '.'
				if pt.Value != "." {
					// Ensure not being qualified immediately after (Schema.Table)
					if !isNextMeaningfulDot(tokens, i) {

						upperPt := strings.ToUpper(pt.Value)

						// Case 1: Direct Object Keywords (e.g. CREATE TABLE x, DROP VIEW y)
						if objectKeywords[upperPt] {
							inject = true
						} else if upperPt == "BODY" {
							// PACKAGE BODY x
							pptIdx := prevMeaningfulTokenIdx(tokens, ptIdx)
							if pptIdx != -1 && strings.ToUpper(tokens[pptIdx].Value) == "PACKAGE" {
								inject = true
							}
						} else if upperPt == "ON" {
							// CREATE INDEX x ON y
							// GRANT ... ON y
							if isInsideCreateIdxOrGrant(tokens, ptIdx) {
								inject = true
							}
						} else if upperPt == "INDEX" {
							// CREATE INDEX x
							// DROP INDEX x
							// ALTER INDEX x
							inject = true
						}
					}
				}
			}
		}

		if inject {
			sb.WriteString(targetSchema)
			sb.WriteString(".")
		}
		sb.WriteString(token.Value)
	}

	return sb.String(), warnings, nil
}

func prevMeaningfulTokenIdx(tokens []lexer.Token, currentIdx int) int {
	for j := currentIdx - 1; j >= 0; j-- {
		if tokens[j].Type != lexer.TokenWhitespace && tokens[j].Type != lexer.TokenComment {
			return j
		}
	}
	return -1
}

func isNextMeaningfulDot(tokens []lexer.Token, currentIdx int) bool {
	for j := currentIdx + 1; j < len(tokens); j++ {
		if tokens[j].Type != lexer.TokenWhitespace && tokens[j].Type != lexer.TokenComment {
			return tokens[j].Value == "."
		}
	}
	return false
}

// isInsideCreateIdxOrGrant scans backwards to determine context
func isInsideCreateIdxOrGrant(tokens []lexer.Token, onIdx int) bool {
	for j := onIdx - 1; j >= 0; j-- {
		t := tokens[j]
		if t.Type == lexer.TokenSymbol && t.Value == ";" {
			return false
		}
		if t.Type == lexer.TokenKeyword {
			val := strings.ToUpper(t.Value)
			if val == "JOIN" {
				return false
			}
			if val == "GRANT" {
				return true
			}
			if val == "CREATE" {
				return true
			}
		}
	}
	return false
}