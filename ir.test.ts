import { test, expect, assert } from 'vitest'
import { lower, lowerType, type Type } from './ir.ts'

test('lowerType', () => {
	expect(lowerType({
		tag: 'function',
		parameter: { tag: 'typeVariable', name: 'a' },
		returnType: {
			tag: 'function',
			parameter: { tag: 'typeVariable', name: 'b' },
			returnType: { tag: 'typeVariable', name: 'c' },
		},
	})).toStrictEqual({
		tag: 'generic', parameter: 'type', definition: {
			tag: 'generic', parameter: 'type', definition: {
				tag: 'generic', parameter: 'type', definition: {
					tag: 'function',
					parameter: { tag: 'typeVariable', index: 0 },
					returnType: {
						tag: 'function',
						parameter: { tag: 'typeVariable', index: 1 },
						returnType: { tag: 'typeVariable', index: 2 },
					},
				}
			}
		}
	} satisfies Type)
})
