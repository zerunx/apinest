import { EditorView } from '@codemirror/view'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'
import type { Extension } from '@codemirror/state'

const highlightStyle = HighlightStyle.define([
  { tag: t.propertyName, color: 'var(--raw-hl-key)' },
  { tag: t.definition(t.propertyName), color: 'var(--raw-hl-key)' },
  { tag: t.attributeName, color: 'var(--raw-hl-attr)' },
  { tag: t.string, color: 'var(--raw-hl-string)' },
  { tag: t.number, color: 'var(--raw-hl-number)' },
  { tag: t.bool, color: 'var(--raw-hl-bool)' },
  { tag: t.null, color: 'var(--raw-hl-null)' },
  { tag: t.keyword, color: 'var(--raw-hl-bool)' },
  { tag: t.comment, color: 'var(--raw-hl-comment)', fontStyle: 'italic' },
  { tag: t.tagName, color: 'var(--raw-hl-tag)' },
  { tag: t.variableName, color: 'var(--raw-hl-plain)' },
  { tag: t.operator, color: 'var(--raw-hl-punct)' },
  { tag: t.punctuation, color: 'var(--raw-hl-punct)' },
  { tag: t.separator, color: 'var(--raw-hl-punct)' },
  { tag: t.bracket, color: 'var(--raw-hl-punct)' },
  { tag: t.squareBracket, color: 'var(--raw-hl-punct)' },
  { tag: t.brace, color: 'var(--raw-hl-punct)' },
  { tag: t.invalid, color: 'var(--raw-hl-error)' },
])

function chromeTheme(dark: boolean): Extension {
  return EditorView.theme(
    {
      '&': {
        height: '100%',
        backgroundColor: 'var(--raw-editor-bg)',
        color: 'var(--raw-hl-plain)',
        colorScheme: dark ? 'dark' : 'light',
        fontSize: '13px',
      },
      '&.cm-focused': {
        outline: 'none',
      },
      '.cm-scroller': {
        fontFamily: 'var(--raw-editor-font-family)',
        fontSize: '13px',
        lineHeight: '20px',
      },
      '.cm-content': {
        caretColor: 'var(--color-interface-field-text)',
      },
      '.cm-line ::selection, .cm-selectionBackground': {
        backgroundColor: 'var(--raw-editor-selection) !important',
      },
      '.cm-cursor, .cm-dropCursor': {
        borderLeftColor: 'var(--color-interface-field-text)',
      },
      '.cm-gutters': {
        backgroundColor: 'var(--raw-editor-gutter-bg)',
        color: 'var(--raw-editor-gutter-fg)',
        border: 'none',
        borderRight: '1px solid var(--raw-editor-gutter-border)',
      },
      '.cm-lineNumbers .cm-gutterElement': {
        color: 'var(--raw-editor-gutter-fg)',
        padding: '0 4px 0 12px',
        minWidth: '32px',
      },
      '.cm-foldGutter': {
        width: '14px',
      },
      '.cm-foldGutter .cm-gutterElement': {
        padding: '0 2px 0 0',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        cursor: 'pointer',
        color: 'var(--raw-editor-gutter-fg)',
      },
      '.cm-foldGutter .cm-gutterElement:hover': {
        color: 'var(--color-interface-field-text)',
      },
      '.cm-fold-chevron': {
        display: 'block',
        width: 0,
        height: 0,
        borderLeft: '3.5px solid transparent',
        borderRight: '3.5px solid transparent',
        borderTop: '5px solid currentColor',
      },
      '.cm-fold-chevron--closed': {
        borderTop: '3.5px solid transparent',
        borderBottom: '3.5px solid transparent',
        borderLeft: '5px solid currentColor',
        borderRight: 'none',
      },
      '.cm-foldPlaceholder': {
        backgroundColor: 'transparent',
        border: 'none',
        padding: '0 4px',
        color: 'var(--raw-hl-punct)',
        cursor: 'pointer',
      },
      '.cm-activeLineGutter': {
        backgroundColor: 'transparent',
      },
      '.cm-placeholder': {
        color: 'var(--color-interface-placeholder)',
      },
      '.cm-matchingBracket': {
        backgroundColor: 'var(--raw-editor-bracket-match)',
        outline: 'none',
        color: 'inherit',
      },
      '.cm-nonmatchingBracket': {
        backgroundColor: 'transparent',
        color: 'var(--raw-hl-error)',
      },
    },
    { dark },
  )
}

export function editorTheme(dark: boolean): Extension {
  return [chromeTheme(dark), syntaxHighlighting(highlightStyle)]
}
