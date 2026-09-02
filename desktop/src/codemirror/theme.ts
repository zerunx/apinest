import { EditorView } from '@codemirror/view'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'
import { Prec, type Extension } from '@codemirror/state'

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

/** Soft repeating squiggle (VS Code–like). Period must match SVG width. */
function lintWave(color: string): string {
  const svg =
    `<svg xmlns="http://www.w3.org/2000/svg" width="10" height="3" viewBox="0 0 10 3" fill="none">` +
    `<path d="M0 2.1C1.25 2.1 1.25.9 2.5.9S3.75 2.1 5 2.1 6.25.9 7.5.9 8.75 2.1 10 2.1" ` +
    `stroke="${color}" stroke-width="1.05" stroke-linecap="round" stroke-linejoin="round"/>` +
    `</svg>`
  return `url("data:image/svg+xml,${encodeURIComponent(svg)}")`
}

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
        // Stable CJK/Latin mixed metrics while selection overlays update.
        fontKerning: 'none',
        fontVariantLigatures: 'none',
      },
      // Prefer filling the first visual row (key + value) instead of wrapping
      // at the space after `: `, which parked the string on the next line.
      '.cm-lineWrapping': {
        wordBreak: 'break-all',
        overflowWrap: 'anywhere',
      },
      '.cm-content': {
        caretColor: 'transparent',
        lineHeight: '20px',
        fontKerning: 'none',
        fontVariantLigatures: 'none',
      },
      '.cm-line': {
        lineHeight: '20px',
        paddingTop: '0',
        paddingBottom: '0',
      },
      // Indent wrap rows only. Negative text-indent keeps the first row full-width.
      '.cm-wrapIndent': {
        paddingLeft: 'calc(6px + var(--cm-wrap-indent, 0ch))',
        textIndent: 'calc(-1 * var(--cm-wrap-indent, 0ch))',
      },
      '.cm-line::selection, .cm-line ::selection, .cm-content ::selection': {
        backgroundColor: 'transparent !important',
      },
      '.cm-roundedSelection': {
        backgroundColor: 'var(--raw-editor-selection)',
      },
      '.cm-roundedSelectionLayer': {
        pointerEvents: 'none',
      },
      '.cm-selectedSpaceLayer': {
        pointerEvents: 'none',
      },
      '.cm-selectedSpaceDot': {
        position: 'absolute',
        borderRadius: '50%',
        backgroundColor: 'var(--raw-hl-space-dot)',
        pointerEvents: 'none',
      },
      '.cm-cursor, .cm-dropCursor': {
        borderLeftWidth: 'var(--raw-editor-cursor-width)',
        borderLeftStyle: 'solid',
        borderLeftColor: 'var(--color-interface-field-text)',
        marginLeft: '-1px',
        marginTop: 'var(--raw-editor-cursor-nudge-y)',
        height: 'var(--raw-editor-cursor-height) !important',
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
        backgroundColor: 'transparent',
        outline: '1px solid var(--raw-editor-bracket-match)',
        outlineOffset: '0px',
        borderRadius: '1px',
        color: 'inherit',
      },
      '.cm-nonmatchingBracket': {
        backgroundColor: 'transparent',
        color: 'var(--raw-hl-error)',
      },
      '.cm-lintRange': {
        backgroundImage: 'none',
        backgroundPosition: 'left bottom',
        backgroundRepeat: 'repeat-x',
        paddingBottom: '1px',
      },
      '.cm-lintRange-error': {
        backgroundColor: dark ? 'rgba(241, 76, 76, 0.12)' : 'rgba(209, 52, 56, 0.1)',
        backgroundImage: lintWave(dark ? 'rgba(241, 76, 76, 0.95)' : 'rgba(209, 52, 56, 0.9)'),
        backgroundSize: '10px 3px',
        backgroundPosition: 'left bottom',
        backgroundRepeat: 'repeat-x',
        borderRadius: '2px',
      },
      '.cm-searchMatch': {
        backgroundColor: 'var(--raw-find-match)',
        borderRadius: '2px',
      },
      '.cm-searchMatch-selected': {
        backgroundColor: 'var(--raw-find-match-active)',
      },
    },
    { dark },
  )
}

export function editorTheme(dark: boolean): Extension {
  return [
    chromeTheme(dark),
    syntaxHighlighting(highlightStyle),
    // Keep native ::selection (avoids CM block selection) but hide the native caret.
    Prec.highest(
      EditorView.theme({
        '.cm-line::selection, .cm-line ::selection, .cm-content ::selection': {
          backgroundColor: 'transparent !important',
        },
        '.cm-content': {
          caretColor: 'transparent !important',
        },
        '.cm-selectionLayer': {
          display: 'none',
        },
      }),
    ),
  ]
}
