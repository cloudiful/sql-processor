package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sql-processor/pkg/processor"
)

func main() {
	if err := run(os.Args[1:], os.Stdin, os.Stdout, os.Stderr); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

func run(args []string, stdin io.Reader, stdout, stderr io.Writer) error {
	var (
		targetSchema string
		inputPath    string
		outputPath   string
		isGB2312     bool
	)

	fs := flag.NewFlagSet("sql-processor", flag.ContinueOnError)
	fs.SetOutput(stderr)
	fs.StringVar(&targetSchema, "schema", "", "Target schema name to inject into DDL statements")
	fs.StringVar(&inputPath, "input", "", "Input SQL file path (if empty, reads from stdin)")
	fs.StringVar(&outputPath, "output", "", "Output SQL file path (if empty, writes to stdout)")
	fs.BoolVar(&isGB2312, "gb2312", false, "Enable GB2312/GB18030 encoding handling")

	fs.Usage = func() {
		fmt.Fprintf(stderr, "Usage of sql-processor:\n")
		fmt.Fprintf(stderr, "  sql-processor [flags]\n\n")
		fmt.Fprintf(stderr, "Examples:\n")
		fmt.Fprintf(stderr, "  sql-processor -schema EBANK -input init.sql -output out.sql\n")
		fmt.Fprintf(stderr, "  cat init.sql | sql-processor -schema EBANK > out.sql\n\n")
		fmt.Fprintf(stderr, "Flags:\n")
		fs.PrintDefaults()
	}

	if err := fs.Parse(args); err != nil {
		return err
	}

	var inputData []byte
	var err error

	// 1. Read input
	if inputPath != "" {
		inputData, err = os.ReadFile(inputPath)
		if err != nil {
			return fmt.Errorf("error reading input file: %w", err)
		}
	} else {
		inputData, err = io.ReadAll(stdin)
		if err != nil {
			return fmt.Errorf("error reading from stdin: %w", err)
		}
	}

	if len(inputData) == 0 {
		return nil
	}

	var finalData []byte
	var logs []string

	// 2. Process
	if isGB2312 {
		// Handle with GB2312 logic
		name := "stdin"
		if inputPath != "" {
			name = filepath.Base(inputPath)
		}
		gbData, subLogs, err := processor.EnsureGB2312Memory(name, inputData)
		logs = append(logs, subLogs...)
		if err != nil {
			return fmt.Errorf("error ensuring GB2312: %w", err)
		}

		finalData, subLogs, err = processor.ProcessGB2312SQLBytes(name, gbData, targetSchema)
		logs = append(logs, subLogs...)
		if err != nil {
			return fmt.Errorf("error processing SQL bytes: %w", err)
		}
	} else {
		// Handle as UTF-8
		content := string(inputData)
		processedContent, messages, err := processor.ProcessSQLText(content, targetSchema)
		if err != nil {
			return fmt.Errorf("error processing SQL text: %w", err)
		}
		finalData = []byte(processedContent)
		logs = messages
	}

	// 3. Print messages/logs to stderr
	for _, log := range logs {
		fmt.Fprintf(stderr, "[INFO] %s\n", log)
	}

	// 4. Write output
	if outputPath != "" {
		err = os.WriteFile(outputPath, finalData, 0644)
		if err != nil {
			return fmt.Errorf("error writing output file: %w", err)
		}
	} else {
		_, err = stdout.Write(finalData)
		if err != nil {
			return fmt.Errorf("error writing to stdout: %w", err)
		}
	}

	return nil
}
