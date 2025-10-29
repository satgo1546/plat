import { test, expect } from 'vitest'
import { tokenize } from './index.ts'

test('tokenizer', () => {
  expect(tokenize(`// this is a comment
(( )){} // grouping stuff
!*+-/=<> <= == // operators`)).toMatchInlineSnapshot(`
  [
    {
      "lexeme": "(",
      "line": 2,
      "literal": undefined,
      "type": "(",
    },
    {
      "lexeme": "(",
      "line": 2,
      "literal": undefined,
      "type": "(",
    },
    {
      "lexeme": ")",
      "line": 2,
      "literal": undefined,
      "type": ")",
    },
    {
      "lexeme": ")",
      "line": 2,
      "literal": undefined,
      "type": ")",
    },
    {
      "lexeme": "{",
      "line": 2,
      "literal": undefined,
      "type": "{",
    },
    {
      "lexeme": "}",
      "line": 2,
      "literal": undefined,
      "type": "}",
    },
    {
      "lexeme": "!",
      "line": 3,
      "literal": undefined,
      "type": "!",
    },
    {
      "lexeme": "*",
      "line": 3,
      "literal": undefined,
      "type": "*",
    },
    {
      "lexeme": "+",
      "line": 3,
      "literal": undefined,
      "type": "+",
    },
    {
      "lexeme": "-",
      "line": 3,
      "literal": undefined,
      "type": "-",
    },
    {
      "lexeme": "/",
      "line": 3,
      "literal": undefined,
      "type": "/",
    },
    {
      "lexeme": "=",
      "line": 3,
      "literal": undefined,
      "type": "=",
    },
    {
      "lexeme": "<",
      "line": 3,
      "literal": undefined,
      "type": "<",
    },
    {
      "lexeme": ">",
      "line": 3,
      "literal": undefined,
      "type": ">",
    },
    {
      "lexeme": "<=",
      "line": 3,
      "literal": undefined,
      "type": "<=",
    },
    {
      "lexeme": "==",
      "line": 3,
      "literal": undefined,
      "type": "==",
    },
    {
      "lexeme": "",
      "line": 3,
      "type": undefined,
    },
  ]
`)
})
