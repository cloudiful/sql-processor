package processor

import (
	"fmt"
	"regexp"
	"strings"
)

// TryMakeIdempotent checks if the SQL contains CREATE or DROP statements for INDEX/TABLE/SEQUENCE/SYNONYM
// and wraps them in PL/SQL blocks to make them idempotent.
// It handles multiple statements by splitting them and wrapping each compatible one.
func TryMakeIdempotent(sql string) (string, bool) {
	// 1. Split into individual statements by semicolon
	// We use a split that doesn't split on semicolons inside comments.
	rawStmts := splitSQLIgnoringComments(sql)
	var resultStmts []string
	modified := false

	for _, rawStmt := range rawStmts {
		trimmedRaw := strings.TrimSpace(rawStmt)
		if trimmedRaw == "" {
			continue
		}

		cleanStmt := RemoveSQLComments(trimmedRaw)
		trimmedClean := strings.TrimSpace(cleanStmt)

		// Regex for CREATE or DROP
		reCreate := regexp.MustCompile(`(?i)^CREATE\s+(?:OR\s+REPLACE\s+)?(?:(?:UNIQUE|BITMAP)\s+)?(INDEX|TABLE|SEQUENCE|SYNONYM)\b`)
		reDrop := regexp.MustCompile(`(?i)^DROP\s+(?:PUBLIC\s+)?(INDEX|TABLE|VIEW|SEQUENCE|SYNONYM)\b`)

		if reCreate.MatchString(trimmedClean) {
			// Handle CREATE (ORA-00955)
			stmt := strings.ReplaceAll(trimmedClean, "'", "''")
			commentedStmt := "-- " + strings.ReplaceAll(trimmedClean, "\n", "\n-- ")
			block := fmt.Sprintf(`-- [AUTO-GENERATED] Idempotent wrapper for:
%s
DECLARE
    e_exists EXCEPTION;
    PRAGMA EXCEPTION_INIT(e_exists, -955); -- ORA-00955: name is already used by an existing object
BEGIN
    EXECUTE IMMEDIATE '%s';
EXCEPTION
    WHEN e_exists THEN
        DBMS_OUTPUT.PUT_LINE('Warning: Object already exists, skipping creation.');
        NULL;
END;
/`, commentedStmt, stmt)
			resultStmts = append(resultStmts, block)
			modified = true
		} else if reDrop.MatchString(trimmedClean) {
			// Handle DROP (ORA-00942, ORA-01418, ORA-02289)
			errorCode := -942 // Default for table or view
			comment := "table or view does not exist"
			upper := strings.ToUpper(trimmedClean)

			if strings.Contains(upper, "INDEX") {
				errorCode = -1418
				comment = "specified index does not exist"
			} else if strings.Contains(upper, "SEQUENCE") {
				errorCode = -2289
				comment = "sequence does not exist"
			} else if strings.Contains(upper, "SYNONYM") {
				if strings.Contains(upper, "PUBLIC") {
					errorCode = -1432
					comment = "public synonym does not exist"
				} else {
					errorCode = -1434
					comment = "private synonym does not exist"
				}
			}

			stmt := strings.ReplaceAll(trimmedClean, "'", "''")
			commentedStmt := "-- " + strings.ReplaceAll(trimmedClean, "\n", "\n-- ")
			block := fmt.Sprintf(`-- [AUTO-GENERATED] Idempotent wrapper for:
%s
DECLARE
    e_not_exists EXCEPTION;
    PRAGMA EXCEPTION_INIT(e_not_exists, %d); -- ORA-%05d: %s
BEGIN
    EXECUTE IMMEDIATE '%s';
EXCEPTION
    WHEN e_not_exists THEN
        DBMS_OUTPUT.PUT_LINE('Warning: Object does not exist, skipping drop.');
        NULL;
END;
/`, commentedStmt, errorCode, -errorCode, comment, stmt)
			resultStmts = append(resultStmts, block)
			modified = true
		} else {
			// Not a DDL we want to wrap, keep original
			resultStmts = append(resultStmts, trimmedRaw+";")
		}
	}

	if !modified {
		return sql, false
	}

	return strings.Join(resultStmts, "\n\n"), true
}