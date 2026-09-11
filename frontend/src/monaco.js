import * as monaco from 'monaco-editor/esm/vs/editor/editor.api'
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'
import 'monaco-editor/esm/vs/basic-languages/sql/sql.contribution'

self.MonacoEnvironment = {
  getWorker(_, label) {
    return new editorWorker({ name: label, type: 'module' })
  },
}

monaco.editor.defineTheme('sql-processor-night', {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: 'comment', foreground: '4C8C58', fontStyle: 'italic' },
    { token: 'keyword', foreground: '79FF97', fontStyle: 'bold' },
    { token: 'string', foreground: 'D2FF72' },
    { token: 'number', foreground: '8BFFB0' },
  ],
  colors: {
    'editor.background': '#050805',
    'editor.foreground': '#d6ffe1',
    'editor.lineHighlightBackground': '#0d160d',
    'editor.selectionBackground': '#16321d',
    'editor.inactiveSelectionBackground': '#102415',
    'editorCursor.foreground': '#79ff97',
    'editorLineNumber.foreground': '#3d6f46',
    'editorLineNumber.activeForeground': '#8bffb0',
  },
})

monaco.editor.defineTheme('sql-processor-day', {
  base: 'vs',
  inherit: true,
  rules: [
    { token: 'comment', foreground: '6a737d', fontStyle: 'italic' },
    { token: 'keyword', foreground: '005cc5', fontStyle: 'bold' },
    { token: 'string', foreground: 'a31515' },
    { token: 'number', foreground: '098658' },
  ],
  colors: {
    'editor.background': '#f7fafd',
    'editor.foreground': '#102033',
    'editor.lineHighlightBackground': '#eef4fa',
    'editor.selectionBackground': '#cfe5ff',
    'editor.inactiveSelectionBackground': '#dce8f6',
    'editorCursor.foreground': '#1455ff',
    'editorLineNumber.foreground': '#8ea2b6',
    'editorLineNumber.activeForeground': '#334961',
  },
})

export { monaco }
