import { isDeepStrictEqual } from 'node:util'
import { type Expression, type Type as ExpressionType, type TypeVariable as ExpressionTypeVariable } from './index.ts'

export type Variable = { tag: 'variable', name: string, type: Type }

export type Node =
	| Variable
	| { tag: 'number', value: number }
	| { tag: 'function', parameter: Variable, body: Node }
	| { tag: 'call', callee: Node, argument: Node }
	| { tag: 'let', variable: Variable, value: Node, body: Node }

export type Type =
	| { tag: 'typeVariable', index: number }
	| { tag: 'number' }
	| { tag: 'function', parameter: Type, returnType: Type }
	| { tag: 'generic', parameter: Kind, definition: Type }

export type Kind = 'type'

function reify(type: Type, argument: Type, index = 0): Type {
	switch (type.tag) {
		case 'typeVariable':
			if (type.index === index) return argument
			if (type.index < index) return type
			return { tag: 'typeVariable', index: type.index - 1 }
		case 'number':
			return type
		case 'function':
			return {
				tag: 'function',
				parameter: reify(type.parameter, argument),
				returnType: reify(type.returnType, argument),
			}
		case 'generic':
			return {
				tag: 'generic',
				parameter: type.parameter,
				definition: reify(type.definition, argument, index + 1),
			}
	}
}

function getNodeType(node: Node): Type {
	switch (node.tag) {
		case 'variable':
			return node.type
		case 'number':
			return { tag: 'number' }
		case 'function':
			return { tag: 'function', parameter: node.parameter.type, returnType: getNodeType(node.body) }
		case 'call':
			const calleeType = getNodeType(node.callee)
			if (calleeType.tag !== 'function') throw new Error('impossible')
			if (!isDeepStrictEqual(getNodeType(node.argument), calleeType.parameter)) throw new Error('impossible')
			return calleeType.returnType
		case 'let':
			if (!isDeepStrictEqual(node.variable.type, getNodeType(node.value))) throw new Error('impossible')
			return getNodeType(node.body)
	}
}

export function lowerType(type: ExpressionType): Type {
	const typeVariables = new Map<string, number>
	let result = function recurse(type: ExpressionType): Type {
		switch (type.tag) {
			case 'typeVariable':
				return {
					tag: 'typeVariable',
					index: typeVariables.getOrInsert(type.name, typeVariables.size),
				}
			case 'number':
				return { tag: 'number' }
			case 'function':
				return {
					tag: 'function',
					parameter: recurse(type.parameter),
					returnType: recurse(type.returnType),
				}
		}
	}(type)
	console.log(typeVariables)
	for (const _ of typeVariables.keys()) {
		result = { tag: 'generic', parameter: 'type', definition: result }
	}
	return result
}

function lowerExpression(expression: Expression): Node {
	switch (expression.tag) {
		case 'variable':
			if (!expression.type) throw new Error('untyped expression')
			return { tag: 'variable', name: expression.name, type: lowerType(expression.type) }
		case 'number':
			return { tag: 'number', value: expression.value }
		case 'function':
			if (!expression.parameter.type) throw new Error('untyped expression')
			return {
				tag: 'function',
				parameter: { tag: 'variable', name: expression.parameter.name, type: lowerType(expression.parameter.type) },
				body: lowerExpression(expression.body),
			}
		case 'call':
			return {
				tag: 'call',
				callee: lowerExpression(expression.callee),
				argument: lowerExpression(expression.argument),
			}
		case 'error':
			throw new Error('expression with error')
	}
}

export function lower(expression: Expression, type: ExpressionType) {
	lowerType(type)
	throw Error('not implemented')
}
