import { test, expect } from 'vitest'
import { evaluate, parse, pprint, tokenize } from './index.ts'

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

test('parser', () => {
  expect(parse(tokenize('6 / 3 - 1'))).toMatchInlineSnapshot(`
    {
      "left": {
        "left": {
          "type": "literal",
          "value": 6,
        },
        "operator": {
          "lexeme": "/",
          "line": 1,
          "literal": undefined,
          "type": "/",
        },
        "right": {
          "type": "literal",
          "value": 3,
        },
        "type": "binary",
      },
      "operator": {
        "lexeme": "-",
        "line": 1,
        "literal": undefined,
        "type": "-",
      },
      "right": {
        "type": "literal",
        "value": 1,
      },
      "type": "binary",
    }
  `)
  expect(parse(tokenize('"a" == "b" == "c"'))).toMatchInlineSnapshot(`
    {
      "left": {
        "left": {
          "type": "literal",
          "value": "a",
        },
        "operator": {
          "lexeme": "==",
          "line": 1,
          "literal": undefined,
          "type": "==",
        },
        "right": {
          "type": "literal",
          "value": "b",
        },
        "type": "binary",
      },
      "operator": {
        "lexeme": "==",
        "line": 1,
        "literal": undefined,
        "type": "==",
      },
      "right": {
        "type": "literal",
        "value": "c",
      },
      "type": "binary",
    }
  `)
})

test('interpreter', () => {
  // "scone" + (-4 * 5 - 1)
  expect(evaluate({
    type: 'binary',
    left: { type: 'literal', value: 'scone' },
    operator: { type: '+', lexeme: '+', line: 1 },
    right: {
      type: 'grouping',
      expression: {
        type: 'binary',
        left: {
          left: {
            operator: { type: '-', lexeme: '-', line: 1 },
            right: { type: 'literal', value: 4 },
            type: 'unary',
          },
          operator: { type: '*', lexeme: '*', line: 1 },
          right: { type: 'literal', value: 5 },
          type: 'binary',
        },
        operator: { type: '-', lexeme: '-', line: 1 },
        right: { type: 'literal', value: 1 },
      },
    },
  })).toBe('scone-21')
})
