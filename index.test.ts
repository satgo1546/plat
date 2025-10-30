import { test, expect } from 'vitest'
import { pprint, tokenize } from './index.ts'

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

test('pretty printer', () => {
  expect(pprint({
    type: 'binary',
    left: {
      type: 'unary',
      operator: { type: '-', lexeme: '-', line: 1 },
      right: { type: 'literal', value: 123 },
    },
    operator: { type: '*', lexeme: '*', line: 1 },
    right: {
      type: 'grouping',
      expression: { type: 'literal', value: 45.67 },
    },
  })).toBe('(* (- 123) (group 45.67))')
})
