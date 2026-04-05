import { type Expression, infer } from './index.ts'

const expr: Expression = {
	tag: 'function',
	parameter: { tag: 'variable', name: 'y' },
	body: {
		tag: 'call',
		callee: {
			tag: 'function',
			parameter: { tag: 'variable', name: 'x' },
			body: { tag: 'variable', name: 'x' },
		},
		argument: { tag: 'number', value: 114514 },
	},
}
const type = infer(expr)
console.log(expr, ':', type)
