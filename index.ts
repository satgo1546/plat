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
	| { tag: 'let', variables: Record<string, Expression>, body: Expression }

export type Type =
	| { tag: 'namedType', name: string }
	| { tag: 'reify', generic: Type, argument: Type }
	| { tag: 'typeVariable', link?: Type, name: string, level: number }

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

let currentLevel = 0

export function functionType(...types: Type[]): Type {
	return types.reduceRight((result, parameter) => ({
		tag: 'reify',
		generic: {
			tag: 'reify',
			generic: builtinTypes.function,
			argument: parameter,
		},
		argument: result,
	}))
}

export function newTypeVariable(): Type & { tag: 'typeVariable' } {
	return { tag: 'typeVariable', name: crypto.randomUUID(), level: currentLevel }
}

function find(type: Type): Type {
	if (type.tag === 'typeVariable' && type.link) {
		return type.link = find(type.link)
	}
	return type
}

function unify(a: Type, b: Type): void {
	a = find(a)
	b = find(b)
	if (a.tag === 'typeVariable') {
		if (b.tag === 'typeVariable' && a.name === b.name) {
			if (a.level !== b.level) throw new Error('?')
			return
		}
		(function recurse(type: Type) {
			type = find(type)
			switch (type.tag) {
				case 'namedType':
					break
				case 'reify':
					recurse(type.generic)
					recurse(type.argument)
					break
				case 'typeVariable':
					if (type.name === a.name) throw new Error('infinite type')
					type.level = Math.min(type.level, a.level)
					break
			}
		})(b)
		a.link = b
		a.level = NaN // should not matter, meant for debugging
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

function generalize(type: Type): Type {
	type = find(type)
	switch (type.tag) {
		case 'namedType':
			return type
		case 'reify':
			return {
				tag: 'reify',
				generic: generalize(type.generic),
				argument: generalize(type.argument),
			}
		case 'typeVariable':
			if (type.level > currentLevel) type.level = Infinity
			return type
	}
}

function instantiate(type: Type): Type {
	const map: Record<string, Type> = Object.create(null)
	return function recurse(type: Type): Type {
		type = find(type)
		switch (type.tag) {
			case 'namedType':
				return type
			case 'reify':
				return {
					tag: 'reify',
					generic: recurse(type.generic),
					argument: recurse(type.argument),
				}
			case 'typeVariable':
				return type.level === Infinity ? map[type.name] ??= newTypeVariable() : type
		}
	}(type)
}

export type Scope = Record<string, Type>

function inferExpression(scope: Scope, expression: Expression): Type {
	switch (expression.tag) {
		case 'variable':
			if (!Object.hasOwn(scope, expression.name)) {
				throw new Error(`undefined variable '${expression.name}'`)
			}
			return instantiate(scope[expression.name])
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
			currentLevel++
			const innerScope = { ...scope }
			for (const name in expression.variables) {
				innerScope[name] = newTypeVariable()
			}
			for (const name in expression.variables) {
				unify(innerScope[name], inferExpression(innerScope, expression.variables[name]))
			}
			currentLevel--
			for (const name in expression.variables) {
				innerScope[name] = generalize(innerScope[name])
			}
			return inferExpression(innerScope, expression.body)
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
	currentLevel = 0
	return recursiveFind(inferExpression(scope, expression))
}
