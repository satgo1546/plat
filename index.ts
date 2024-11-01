// 2

type Literal =
	| { type: 'int', value: number }
	| { type: 'char', value: string }
	| { type: 'rational', value: number }
	| { type: 'string', value: string }

type DefinitionGroup = Record<string, Expression>

type Expression =
	| { type: 'variable', id: string }
	| { type: 'literal', value: Literal }
	| { type: 'constructor', assumption: { id: string, type: Scope[string] } }
	| { type: 'call', callee: Expression, argument: Expression }
	| { type: 'function', parameter: string, body: Expression }
	| { type: 'let', definitionGroups: DefinitionGroup[], value: Expression }

// 5

type Type =
	| { type: 'namedType', id: string }
	| { type: 'typeVariable', id: string }
	| { type: 'instantiation', generic: Type, argument: Type }
	| { type: 'typeTemplateParameter', id: number }

const builtinTypes = {
	null: { type: 'namedType', id: 'null' },
	char: { type: 'namedType', id: 'char' },
	int: { type: 'namedType', id: 'int' },
	bigint: { type: 'namedType', id: 'bigint' },
	float: { type: 'namedType', id: 'float' },
	double: { type: 'namedType', id: 'double' },
	list: { type: 'namedType', id: 'list' },
	string: {
		type: 'instantiation',
		generic: { type: 'namedType', id: 'list' },
		argument: { type: 'namedType', id: 'char' },
	},
	function: { type: 'namedType', id: 'function' },
	pair: { type: 'namedType', id: 'pair' },
} satisfies Record<string, Type>

function functionType(parameter: Type, result: Type): Type {
	return {
		type: 'instantiation',
		generic: {
			type: 'instantiation',
			generic: builtinTypes.function,
			argument: parameter,
		},
		argument: result,
	}
}

function listType(element: Type): Type {
	return {
		type: 'instantiation',
		generic: builtinTypes.list,
		argument: element,
	}
}

function pairType(left: Type, right: Type): Type {
	return {
		type: 'instantiation',
		generic: {
			type: 'instantiation',
			generic: builtinTypes.pair,
			argument: left,
		},
		argument: right,
	}
}

type TypeTemplate = {
	parameterCount: number,
	type: Type,
}

function typeTemplateFromType(type: Type, typeVariablesToGeneralize?: Iterable<string>): TypeTemplate {
	if (typeVariablesToGeneralize) {
		const s: Substitutions = {}
		let count = 0
		const typeVariables = typeVariablesInType(type)
		for (const v of typeVariablesToGeneralize) {
			if (typeVariables.has(v)) {
				s[v] = { type: 'typeTemplateParameter', id: count++ }
			}
		}
		return {
			parameterCount: count,
			type: applySubstitutions(s, type)
		}
	} else {
		return {
			parameterCount: 0,
			type,
		}
	}
}

function instantiateTypeTemplate(typeTemplate: TypeTemplate): Type {
	const typeVariables = Array.from({ length: typeTemplate.parameterCount }, newTypeVariable)
	return (function instantiate(type: Type): Type {
		switch (type.type) {
			case 'instantiation':
				return {
					type: 'instantiation',
					generic: instantiate(type.generic),
					argument: instantiate(type.argument),
				}
			case 'typeTemplateParameter':
				return typeVariables[type.id]
			default:
				return type
		}
	})(typeTemplate.type)
}

// 6

type Scope = Record<string, TypeTemplate>

function inferExpression(scope: Scope, expression: Expression): Type {
	switch (expression.type) {
		case 'variable':
			if (Object.hasOwn(scope, expression.id)) {
				return instantiateTypeTemplate(scope[expression.id])
			} else {
				throw new Error(`undefined variable '${expression.id}'`)
			}
		case 'literal':
			return {
				char: builtinTypes.char,
				int: builtinTypes.int,
				string: builtinTypes.string,
				rational: builtinTypes.float,
			}[expression.value.type]
		case 'constructor':
			return instantiateTypeTemplate(expression.assumption.type)
		case 'call':
			const calleeType = inferExpression(scope, expression.callee)
			const argumentType = inferExpression(scope, expression.argument)
			const resultType = newTypeVariable()
			unify(functionType(argumentType, resultType), calleeType)
			return resultType
		case 'function':
			const parameterType = newTypeVariable()
			const bodyType = inferExpression({
				...scope,
				[expression.parameter]: typeTemplateFromType(parameterType),
			}, expression.body)
			return functionType(parameterType, bodyType)
		case 'let':
			return inferExpression({
				...scope,
				...inferLet(scope, expression.definitionGroups),
			}, expression.value)
	}
}

let typeVariableCount = 0
function newTypeVariable(): Type & { type: 'typeVariable' } {
	return { type: 'typeVariable', id: '_t' + typeVariableCount++ }
}

function inferLet(scope: Scope, definitionGroups: DefinitionGroup[]): Scope {
	scope = { ...scope }
	const result: Scope = {}
	for (const definitionGroup of definitionGroups) {
		const newScope: Scope = { ...scope }
		for (const id in definitionGroup) {
			newScope[id] = typeTemplateFromType(newTypeVariable())
		}

		for (const id in definitionGroup) {
			unify(newScope[id].type, inferExpression(newScope, definitionGroup[id]))
		}

		const types: Record<string, Type> = {}
		for (const id in definitionGroup) {
			types[id] = applySubstitutions(substitutions, newScope[id].type)
		}
		const gs = new Set<string>
		for (const type of Object.values(types)) {
			for (const g of typeVariablesInType(type)) {
				gs.add(g)
			}
		}
		for (const typeTemplate of Object.values(scope)) {
			for (const f of typeVariablesInType(applySubstitutions(substitutions, typeTemplate.type))) {
				gs.delete(f)
			}
		}
		for (const id in definitionGroup) {
			scope[id] = result[id] = typeTemplateFromType(types[id], gs)
		}
	}
	return result
}

function inferProgram(scope: Scope, expression: Expression): Type {
	substitutions = {}
	const t = inferExpression(scope, expression)
	return applySubstitutions(substitutions, t)
}

// 7

type Substitutions = Record<string, Type>

function applySubstitutions(substitutions: Substitutions, type: Type): Type {
	switch (type.type) {
		case 'typeVariable':
			return substitutions[type.id] ?? type
		case 'instantiation':
			return {
				type: 'instantiation',
				generic: applySubstitutions(substitutions, type.generic),
				argument: applySubstitutions(substitutions, type.argument),
			}
		default:
			return type
	}
}

function composeSubstitutions(sNew: Substitutions, sOld: Substitutions): Substitutions {
	const result = { ...sOld }
	for (const key in result) {
		result[key] = applySubstitutions(sNew, result[key])
	}
	Object.assign(result, sNew)
	return result
}

let substitutions: Substitutions = {}
function extendSubstitutions(s: Substitutions): void {
	substitutions = composeSubstitutions(s, substitutions)
}

// 8

function mostGeneralUnifier(left: Type, right: Type): Substitutions {
	if (left.type === 'instantiation' && right.type === 'instantiation') {
		const s1 = mostGeneralUnifier(left.generic, right.generic)
		const s2 = mostGeneralUnifier(applySubstitutions(s1, left.argument), applySubstitutions(s1, right.argument))
		return composeSubstitutions(s2, s1)
	} else if (left.type === 'typeVariable') {
		if (right.type === 'typeVariable' && left.id === right.id) {
			return {}
		} else if (typeVariablesInType(right).has(left.id)) {
			throw new Error('infinite type')
		} else {
			return { [left.id]: right }
		}
	} else if (right.type === 'typeVariable') {
		return mostGeneralUnifier(right, left)
	} else if (left.type === 'namedType' && right.type === 'namedType' && left.id === right.id) {
		return {}
	} else if (left.type === 'typeTemplateParameter' || right.type === 'typeTemplateParameter') {
		throw new Error('bug')
	} else {
		throw new Error('types do not match')
	}
}

function unify(type1: Type, type2: Type): void {
	extendSubstitutions(mostGeneralUnifier(applySubstitutions(substitutions, type1), applySubstitutions(substitutions, type2)))
}

// 9

function typeVariablesInType(type: Type): Set<string> {
	switch (type.type) {
		case 'typeVariable':
			return new Set([type.id])
		case 'instantiation':
			const result = typeVariablesInType(type.generic)
			for (const v of typeVariablesInType(type.argument)) {
				result.add(v)
			}
			return result
		default:
			return new Set
	}
}

// 3

import assert from 'node:assert'

const T = newTypeVariable()
const T2 = newTypeVariable()
const std = {
	'+': typeTemplateFromType(functionType(builtinTypes.int, functionType(builtinTypes.int, builtinTypes.int))),
	'++': typeTemplateFromType(functionType(builtinTypes.string, functionType(builtinTypes.string, builtinTypes.string))),
	cons: typeTemplateFromType(functionType(T, functionType(listType(T), listType(T))), [T.id]),
	nil: typeTemplateFromType(listType(T), [T.id]),
	',': typeTemplateFromType(functionType(T, functionType(T2, pairType(T, T2))), [T.id, T2.id]),
} satisfies Scope

function literal(x: string | number): Expression {
	return {
		type: 'literal',
		value: typeof x === 'number' ? { type: Number.isInteger(x) ? 'int' : 'rational', value: x }
			: { type: x.length === 1 ? 'char' : 'string', value: x }
	}
}

function call(callee: Expression | string, ...args: Expression[]): Expression {
	return args.reduce(
		(callee, argument) => ({ type: 'call', callee, argument }),
		typeof callee === 'string' ? { type: 'variable', id: callee } : callee
	)
}

function plus(x: Expression, y: Expression): Expression {
	return call('+', x, y)
}

function lastTypeVariable(n = 1): Type & { type: 'typeVariable' } {
	return { type: 'typeVariable', id: '_t' + (typeVariableCount - n) }
}

// someUndefinedVariable
assert.throws(
	() => inferProgram(std, { type: 'variable', id: 'someUndefinedVariable' }),
	/undefined/,
)

// let a = 10 in [a, a]
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'let',
		definitionGroups: [{
			a: literal(10),
		}],
		value: call('cons',
			{ type: 'variable', id: 'a' },
			call('cons',
				{ type: 'variable', id: 'a' },
				{ type: 'variable', id: 'nil' },
			),
		),
	}),
	listType(builtinTypes.int),
)

// let a = "foo" in let a = 10 in a + a
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'let',
		definitionGroups: [{ a: literal('foo') }],
		value: {
			type: 'let',
			definitionGroups: [{ a: literal(10) }],
			value: plus({ type: 'variable', id: 'a' }, { type: 'variable', id: 'a' }),
		},
	}),
	builtinTypes.int,
)

// (\x -> 'q')
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'function',
		parameter: 'x',
		body: literal('q'),
	}),
	functionType(lastTypeVariable(), builtinTypes.char),
)

// (\x -> x)
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'function',
		parameter: 'x',
		body: { type: 'variable', id: 'x' },
	}),
	functionType(lastTypeVariable(), lastTypeVariable()),
)

// (\x -> x + x)
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'function',
		parameter: 'x',
		body: plus({ type: 'variable', id: 'x' }, { type: 'variable', id: 'x' }),
	}),
	functionType(builtinTypes.int, builtinTypes.int),
)

// (\x -> (x 123) + x)
assert.throws(
	() => inferProgram(std, {
		type: 'function',
		parameter: 'x',
		body: plus(call('x', literal(123)), { type: 'variable', id: 'x' }),
	}),
	/match/,
)

// (\foo -> foo 123)
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'function',
		parameter: 'foo',
		body: call('foo', literal(123)),
	}),
	functionType(
		functionType(builtinTypes.int, lastTypeVariable()),
		lastTypeVariable(),
	),
)

// (\foo -> foo 1 + foo 2)
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'function',
		parameter: 'foo',
		body: plus(call('foo', literal(1)), call('foo', literal(2))),
	}),
	functionType(
		functionType(builtinTypes.int, builtinTypes.int),
		builtinTypes.int,
	),
)

// (\a -> let b = a in b)
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'function',
		parameter: 'a',
		body: {
			type: 'let',
			definitionGroups: [{ b: { type: 'variable', id: 'a' } }],
			value: { type: 'variable', id: 'b' },
		},
	}),
	functionType(lastTypeVariable(2), lastTypeVariable(2)),
)

// (\a -> let b = a in b + b)
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'function',
		parameter: 'a',
		body: {
			type: 'let',
			definitionGroups: [{ b: { type: 'variable', id: 'a' } }],
			value: plus({ type: 'variable', id: 'b' }, { type: 'variable', id: 'b' }),
		},
	}),
	functionType(builtinTypes.int, builtinTypes.int),
)

// let a = 123 in (\x -> a)
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'let',
		definitionGroups: [{ a: literal(123) }],
		value: {
			type: 'function',
			parameter: 'x',
			body: { type: 'variable', id: 'a' },
		}
	}),
	functionType(lastTypeVariable(), builtinTypes.int),
)

// 123 "foo"
assert.throws(
	() => inferProgram(std, {
		type: 'call',
		callee: literal(123),
		argument: literal('foo'),
	}),
	/match/,
)

// (\s -> "s is: " ++ s) 99
assert.throws(
	() => inferProgram(std, {
		type: 'call',
		callee: {
			type: 'function',
			parameter: 's',
			body: call('++', literal('s is : '), { type: 'variable', id: 's' }),
		},
		argument: literal(99),
	}),
	/match/,
)

// let identity = (\x -> x) in identity "foo"
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'let',
		definitionGroups: [{
			identity: {
				type: 'function',
				parameter: 'x',
				body: { type: 'variable', id: 'x' },
			},
		}],
		value: call('identity', literal('foo')),
	}),
	builtinTypes.string,
)

// (\s -> let identity = (\x -> x) in "foo" ++ (identity s))
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'function',
		parameter: 's',
		body: {
			type: 'let',
			definitionGroups: [{
				identity: {
					type: 'function',
					parameter: 'x',
					body: { type: 'variable', id: 'x' },
				},
			}],
			value: call('++', literal('foo'), call('identity', { type: 'variable', id: 's' })),
		},
	}),
	functionType(builtinTypes.string, builtinTypes.string),
)

// (\a -> let identity = (\x -> x) in identity a)
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'function',
		parameter: 'a',
		body: {
			type: 'let',
			definitionGroups: [{
				identity: {
					type: 'function',
					parameter: 'x',
					body: { type: 'variable', id: 'x' },
				},
			}],
			value: call('identity', { type: 'variable', id: 'a' }),
		},
	}),
	functionType(lastTypeVariable(2), lastTypeVariable(2)),
)

// (\f -> f f)
assert.throws(
	() => inferProgram(std, {
		type: 'function',
		parameter: 'f',
		body: call({ type: 'variable', id: 'f' }, { type: 'variable', id: 'f' }),
	}),
	/infinite/,
)

// let identity = (\x -> x) in identity identity
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'let',
		definitionGroups: [{
			identity: {
				type: 'function',
				parameter: 'x',
				body: { type: 'variable', id: 'x' },
			},
		}],
		value: {
			type: 'call',
			callee: { type: 'variable', id: 'identity' },
			argument: { type: 'variable', id: 'identity' },
		},
	}),
	functionType(lastTypeVariable(2), lastTypeVariable(2)),
)

// let foo = (\a -> foo a) in foo "x"
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'let',
		definitionGroups: [{
			foo: {
				type: 'function',
				parameter: 'a',
				body: {
					type: 'call',
					callee: { type: 'variable', id: 'foo' },
					argument: { type: 'variable', id: 'a' },
				},
			},
		}],
		value: {
			type: 'call',
			callee: { type: 'variable', id: 'foo' },
			argument: { type: 'literal', value: { type: 'string', value: 'x' } },
		},
	}),
	lastTypeVariable(2),
)

// let bar = bar in bar
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'let',
		definitionGroups: [{
			bar: { type: 'variable', id: 'bar' },
		}],
		value: { type: 'variable', id: 'bar' },
	}),
	lastTypeVariable(),
)

// let f x = 2 + f (x + 1) in f
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'let',
		definitionGroups: [{
			f: {
				type: 'function',
				parameter: 'x',
				body: plus(literal(2), call('f', plus({ type: 'variable', id: 'x' }, literal(1)))),
			},
		}],
		value: { type: 'variable', id: 'f' },
	}),
	functionType(builtinTypes.int, builtinTypes.int),
)

// let f x = 2 + g x; g x = f (x + 1) in (f, g)
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'let',
		definitionGroups: [{
			f: {
				type: 'function',
				parameter: 'x',
				body: plus(literal(2), call('g', { type: 'variable', id: 'x' })),
			},
			g: {
				type: 'function',
				parameter: 'x',
				body: call('f', plus({ type: 'variable', id: 'x' }, literal(1))),
			},
		}],
		value: call(',', { type: 'variable', id: 'f' }, { type: 'variable', id: 'g' }),
	}),
	pairType(functionType(builtinTypes.int, builtinTypes.int), functionType(builtinTypes.int, builtinTypes.int)),
)

// let identity x = x; foo n = identity identity n in foo identity
assert.deepStrictEqual(
	inferProgram(std, {
		type: 'let',
		definitionGroups: [{
			identity: {
				type: 'function',
				parameter: 'x',
				body: { type: 'variable', id: 'x' },
			},
		}, {
			foo: {
				type: 'function',
				parameter: 'n',
				body: call('identity', { type: 'variable', id: 'identity' }, { type: 'variable', id: 'n' }),
			},
		}],
		value: call('foo', { type: 'variable', id: 'identity' }),
	}),
	functionType(lastTypeVariable(2), lastTypeVariable(2)),
)
