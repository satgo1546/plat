export type Expression =
	| { tag: 'variable', name: string }
	| {
		tag: 'literal', value:
		| { tag: 'int', value: number }
		| { tag: 'char', value: string }
		| { tag: 'float', value: number }
		| { tag: 'string', value: string }
	}
	| { tag: 'call', callee: Expression, argument: Expression }
	| { tag: 'function', parameterName: string, body: Expression }
	| { tag: 'let', variableName: string, variableValue: Expression, body: Expression }

export type Type =
	| { tag: 'namedType', name: string }
	| { tag: 'reify', generic: Type, argument: Type }
	| { tag: 'typeVariable', link?: Type, name: string, quantified: boolean }

export const builtinTypes = {
	null: { tag: 'namedType', name: 'null' },
	int: { tag: 'namedType', name: 'int' },
	char: { tag: 'namedType', name: 'char' },
	float: { tag: 'namedType', name: 'float' },
	list: { tag: 'namedType', name: 'list' },
	string: {
		tag: 'reify',
		generic: { tag: 'namedType', name: 'list' },
		argument: { tag: 'namedType', name: 'char' },
	},
	function: { tag: 'namedType', name: 'function' },
	pair: { tag: 'namedType', name: 'pair' },
} satisfies Record<string, Type>

export function functionType(parameter: Type, result: Type): Type {
	return {
		tag: 'reify',
		generic: {
			tag: 'reify',
			generic: builtinTypes.function,
			argument: parameter,
		},
		argument: result,
	}
}

export function newTypeVariable(): Type & { tag: 'typeVariable' } {
	return { tag: 'typeVariable', name: crypto.randomUUID(), quantified: false }
}

function find(type: Type): Type {
	if (type.tag === 'typeVariable' && type.link) {
		return type.link = find(type.link)
	}
	return type
}

function occurs(type: Type, typeVariable: Type & { tag: 'typeVariable' }): boolean {
	type = find(type)
	switch (type.tag) {
		case 'namedType':
			return false
		case 'reify':
			return occurs(type.generic, typeVariable) || occurs(type.argument, typeVariable)
		case 'typeVariable':
			return type.name === typeVariable.name
	}
}

function unify(a: Type, b: Type): void {
	a = find(a)
	b = find(b)
	if (a.tag === 'typeVariable') {
		if (occurs(b, a)) throw new Error('infinite type')
		a.link = b
	} else if (b.tag === 'typeVariable') {
		unify(b, a)
	} else if (a.tag === 'namedType' && b.tag === 'namedType' && a.name === b.name) {
		// nothing to do
	} else if (a.tag === 'reify' && b.tag === 'reify') {
		unify(a.generic, b.generic)
		unify(a.argument, b.argument)
	} else {
		throw new Error('type mismatch')
	}
}

export type Scope = Record<string, Type>

function inferExpression(scope: Scope, expression: Expression): Type {
	switch (expression.tag) {
		case 'variable':
			if (Object.hasOwn(scope, expression.name)) {
				return scope[expression.name]
			} else {
				throw new Error(`undefined variable '${expression.name}'`)
			}
		case 'literal':
			return {
				char: builtinTypes.char,
				int: builtinTypes.int,
				string: builtinTypes.string,
				float: builtinTypes.float,
			}[expression.value.tag]
		case 'call':
			const resultType = newTypeVariable()
			unify(
				inferExpression(scope, expression.callee),
				functionType(inferExpression(scope, expression.argument), resultType),
			)
			return resultType
		case 'function':
			const parameterType = newTypeVariable()
			return functionType(parameterType, inferExpression({
				...scope,
				[expression.parameterName]: parameterType,
			}, expression.body))
		case 'let':
			return inferExpression({
				...scope,
				[expression.variableName]: inferExpression(scope, expression.variableValue),
			}, expression.body)
	}
}

function recursiveFind(type: Type): Type {
	switch (type.tag) {
		case 'namedType':
			return type
		case 'reify':
			return {
				tag: 'reify',
				generic: recursiveFind(type.generic),
				argument: recursiveFind(type.argument),
			}
		case 'typeVariable':
			return type.link ? recursiveFind(type.link) : type
	}
}

export function inferProgram(scope: Scope, expression: Expression): Type {
	return recursiveFind(inferExpression(scope, expression))
}
