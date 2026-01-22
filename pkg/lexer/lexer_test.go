package lexer

import (
	"reflect"
	"testing"
)

func TestLexSQL(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		wantType []TokenType
	}{
		{
			name:     "simple select",
			input:    "SELECT * FROM users",
			wantType: []TokenType{TokenKeyword, TokenWhitespace, TokenSymbol, TokenWhitespace, TokenKeyword, TokenWhitespace, TokenIdentifier},
		},
		{
			name:     "create table with schema",
			input:    "CREATE TABLE HR.EMPLOYEES (ID NUMBER)",
			wantType: []TokenType{TokenKeyword, TokenWhitespace, TokenKeyword, TokenWhitespace, TokenIdentifier, TokenSymbol, TokenIdentifier, TokenWhitespace, TokenSymbol, TokenIdentifier, TokenWhitespace, TokenIdentifier, TokenSymbol},
		},
		{
			name:     "string literal",
			input:    "INSERT INTO table VALUES ('hello')",
			wantType: []TokenType{TokenKeyword, TokenWhitespace, TokenKeyword, TokenWhitespace, TokenKeyword, TokenWhitespace, TokenKeyword, TokenWhitespace, TokenSymbol, TokenString, TokenSymbol},
		},
		{
			name:     "string with escaped quotes",
			input:    "'it''s a string'",
			wantType: []TokenType{TokenString},
		},
		{
			name:     "double quoted identifier",
			input:    "\"quoted_id\"",
			wantType: []TokenType{TokenIdentifier},
		},
		{
			name:     "comments",
			input:    "-- line comment\n/* block \n comment */",
			wantType: []TokenType{TokenComment, TokenWhitespace, TokenComment},
		},
		{
			name:     "numbers",
			input:    "123 45.67",
			wantType: []TokenType{TokenNumber, TokenWhitespace, TokenNumber},
		},
		{
			name:     "identifiers with special chars",
			input:    "col_name1$ #temp",
			wantType: []TokenType{TokenIdentifier, TokenWhitespace, TokenSymbol, TokenIdentifier},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tokens := LexSQL(tt.input)
			if len(tokens) != len(tt.wantType) {
				t.Errorf("LexSQL() got %d tokens, want %d", len(tokens), len(tt.wantType))
				return
			}
			for i, token := range tokens {
				if token.Type != tt.wantType[i] {
					t.Errorf("LexSQL() token[%d] type = %v, want %v (value: %q)", i, token.Type, tt.wantType[i], token.Value)
				}
			}
		})
	}
}

func TestLexSQL_Values(t *testing.T) {
	input := "SELECT id FROM users WHERE name = 'John'"
	tokens := LexSQL(input)

	expected := []Token{
		{Type: TokenKeyword, Value: "SELECT", Pos: 0},
		{Type: TokenWhitespace, Value: " ", Pos: 6},
		{Type: TokenIdentifier, Value: "id", Pos: 7},
		{Type: TokenWhitespace, Value: " ", Pos: 9},
		{Type: TokenKeyword, Value: "FROM", Pos: 10},
		{Type: TokenWhitespace, Value: " ", Pos: 14},
		{Type: TokenIdentifier, Value: "users", Pos: 15},
		{Type: TokenWhitespace, Value: " ", Pos: 20},
		{Type: TokenKeyword, Value: "WHERE", Pos: 21},
		{Type: TokenWhitespace, Value: " ", Pos: 26},
		{Type: TokenIdentifier, Value: "name", Pos: 27},
		{Type: TokenWhitespace, Value: " ", Pos: 31},
		{Type: TokenSymbol, Value: "=", Pos: 32},
		{Type: TokenWhitespace, Value: " ", Pos: 33},
		{Type: TokenString, Value: "'John'", Pos: 34},
	}

	if !reflect.DeepEqual(tokens, expected) {
		t.Errorf("LexSQL() tokens mismatch")
		for i := 0; i < len(tokens) && i < len(expected); i++ {
			if !reflect.DeepEqual(tokens[i], expected[i]) {
				t.Errorf("Index %d: got %+v, want %+v", i, tokens[i], expected[i])
			}
		}
	}
}

func TestIsKeyword(t *testing.T) {
	tests := []struct {
		input string
		want  bool
	}{
		{"SELECT", true},
		{"select", true},
		{"TABLE", true},
		{"NOT_A_KEYWORD", false},
		{"", false},
	}
	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			if got := isKeyword(tt.input); got != tt.want {
				t.Errorf("isKeyword(%q) = %v, want %v", tt.input, got, tt.want)
			}
		})
	}
}