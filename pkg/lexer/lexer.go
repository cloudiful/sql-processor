package lexer

import (
	"strings"
	"unicode"
)

// TokenType represents the type of a SQL token
type TokenType int

const (
	TokenError TokenType = iota
	TokenEOF
	TokenKeyword
	TokenIdentifier
	TokenString
	TokenNumber
	TokenSymbol
	TokenComment
	TokenWhitespace
)

// Token represents a Lexical token
type Token struct {
	Type  TokenType
	Value string
	Pos   int
}

var keywords = map[string]bool{
	"SELECT": true, "FROM": true, "WHERE": true, "INSERT": true, "INTO": true,
	"UPDATE": true, "DELETE": true, "CREATE": true, "DROP": true, "ALTER": true,
	"TABLE": true, "INDEX": true, "VIEW": true, "SEQUENCE": true, "SYNONYM": true,
	"JOIN": true, "LEFT": true, "RIGHT": true, "INNER": true, "OUTER": true,
	"ON": true, "AND": true, "OR": true, "AS": true, "GROUP": true, "BY": true,
	"ORDER": true, "HAVING": true, "DISTINCT": true, "UNION": true, "VALUES": true,
	"SET": true, "TRUNCATE": true, "GRANT": true, "REVOKE": true, "COMMIT": true, "ROLLBACK": true,
	"TRIGGER": true, "PROCEDURE": true, "FUNCTION": true, "PACKAGE": true, "BODY": true,
	"CONSTRAINT": true, "PRIMARY": true, "KEY": true, "FOREIGN": true, "REFERENCES": true,
	"CHECK": true, "UNIQUE": true, "DEFAULT": true, "NULL": true, "NOT": true, "MERGE": true,
}

// LexSQL lexes the input SQL string into a slice of tokens
func LexSQL(input string) []Token {
	var tokens []Token
	runes := []rune(input)
	length := len(runes)
	pos := 0

	for pos < length {
		r := runes[pos]

		// Whitespace
		if unicode.IsSpace(r) {
			start := pos
			for pos < length && unicode.IsSpace(runes[pos]) {
				pos++
			}
			tokens = append(tokens, Token{Type: TokenWhitespace, Value: string(runes[start:pos]), Pos: start})
			continue
		}

		// Comments
		if r == '-' && pos+1 < length && runes[pos+1] == '-' {
			start := pos
			for pos < length && runes[pos] != '\n' {
				pos++
			}
			tokens = append(tokens, Token{Type: TokenComment, Value: string(runes[start:pos]), Pos: start})
			continue
		}
		if r == '/' && pos+1 < length && runes[pos+1] == '*' {
			start := pos
			pos += 2
			for pos < length {
				if runes[pos] == '*' && pos+1 < length && runes[pos+1] == '/' {
					pos += 2
					break
				}
				pos++
			}
			tokens = append(tokens, Token{Type: TokenComment, Value: string(runes[start:pos]), Pos: start})
			continue
		}

		// Strings (Single quotes)
		if r == '\'' {
			start := pos
			pos++
			for pos < length {
				if runes[pos] == '\'' {
					if pos+1 < length && runes[pos+1] == '\'' {
						pos += 2 // Escaped quote
						continue
					}
					pos++
					break
				}
				pos++
			}
			tokens = append(tokens, Token{Type: TokenString, Value: string(runes[start:pos]), Pos: start})
			continue
		}

		// Identifiers (double quoted)
		if r == '"' {
			start := pos
			pos++
			for pos < length {
				if runes[pos] == '"' {
					pos++
					break
				}
				pos++
			}
			tokens = append(tokens, Token{Type: TokenIdentifier, Value: string(runes[start:pos]), Pos: start})
			continue
		}

		// Identifiers or Keywords (unquoted)
		if isIdentStart(r) {
			start := pos
			for pos < length && isIdentPart(runes[pos]) {
				pos++
			}
			value := string(runes[start:pos])
			typ := TokenIdentifier
			if isKeyword(value) {
				typ = TokenKeyword
			}
			tokens = append(tokens, Token{Type: typ, Value: value, Pos: start})
			continue
		}

		// Numbers
		if unicode.IsDigit(r) {
			start := pos
			for pos < length && (unicode.IsDigit(runes[pos]) || runes[pos] == '.') {
				pos++
			}
			tokens = append(tokens, Token{Type: TokenNumber, Value: string(runes[start:pos]), Pos: start})
			continue
		}

		// Symbols
		tokens = append(tokens, Token{Type: TokenSymbol, Value: string(r), Pos: pos})
		pos++
	}

	return tokens
}

func isIdentStart(r rune) bool {
	return unicode.IsLetter(r) || r == '_'
}

func isIdentPart(r rune) bool {
	return unicode.IsLetter(r) || unicode.IsDigit(r) || r == '_' || r == '$' || r == '#'
}

func isKeyword(s string) bool {
	return keywords[strings.ToUpper(s)]
}