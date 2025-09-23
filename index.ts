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
	| {
		tag: 'let',
		variables: Record<string, {
			type?: Type,
			clauses: Clause[],
		}>,
		body: Expression,
	}

export type Kind = undefined | [Kind, Kind]

export type Type =
	| { tag: 'namedType', name: string, kind?: Kind }
	| { tag: 'reify', generic: Type, argument: Type }
	| {
		tag: 'typeVariable',
		link?: Type,
		name: string,
		kind?: Kind,
		bounds: string[],
		level: number,
	}

export type Pattern =
	| { tag: 'wildcard' }
	| { tag: 'as', variableName: string, pattern: Pattern }
	| { tag: 'literal' } & Expression
	| { tag: 'structure', constructor: Type, fields: Pattern[] }

export type Clause = { parameters: Pattern[], value: Expression }

export const builtinTypes = {
	null: { tag: 'namedType', name: 'null' },
	bool: { tag: 'namedType', name: 'bool' },
	int: { tag: 'namedType', name: 'int' },
	char: { tag: 'namedType', name: 'char' },
	float: { tag: 'namedType', name: 'float' },
	list: { tag: 'namedType', name: 'list', kind: [, ,] },
	string: {
		tag: 'reify',
		generic: { tag: 'namedType', name: 'list', kind: [, ,] },
		argument: { tag: 'namedType', name: 'char' },
	},
	function: { tag: 'namedType', name: 'function', kind: [, [, ,]] },
	pair: { tag: 'namedType', name: 'pair', kind: [, [, ,]] },
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

export type InterfaceDefinitions = Record<string, {
	parents: string[],
	implementations: Type[],
}>
let interfaceDefinitions: InterfaceDefinitions

export function newTypeVariable(kind?: Kind, bounds: string[] = []): Type & { tag: 'typeVariable' } {
	return { tag: 'typeVariable', name: crypto.randomUUID(), kind, bounds, level: currentLevel }
}

function find(type: Type): Type {
	if (type.tag === 'typeVariable' && type.link) {
		return type.link = find(type.link)
	}
	return type
}

function head(type: Type): Type & { tag: 'namedType' | 'typeVariable' } {
	return type.tag === 'reify' ? head(type.generic) : type
}

function unify(a: Type, b: Type): void {
	a = find(a)
	b = find(b)
	if (a.tag === 'typeVariable') {
		if (b.tag === 'typeVariable' && a.name === b.name) {
			if (a !== b) throw new Error('?')
			return
		}
		if (!areEqualKinds(getKind(a), getKind(b))) {
			throw new Error('kind mismatch')
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
		// check bounds
		if (b.tag === 'typeVariable') {
			const impliedBounds = [a.bounds.concat(b.bounds)]
			do {
				impliedBounds.push(impliedBounds.at(-1)!.flatMap(x => interfaceDefinitions[x].parents))
			} while (impliedBounds.at(-1)!.length)
			const newBounds = new Set(impliedBounds.shift())
			for (const x of impliedBounds.flat()) newBounds.delete(x)
			b.bounds = [...newBounds]
		} else {
			const bHead = head(b)
			if (bHead.tag === 'namedType') {
				for (const bound of a.bounds) {
					const implementation = interfaceDefinitions[bound].implementations.find(i => {
						const iHead = head(i)
						return iHead.tag === 'namedType' && iHead.name === bHead.name
					})
					if (!implementation) throw new Error('interface not implemented')
					unify(instantiate(implementation), b)
				}
			} else {
				if (a.bounds.length) throw new Error('unsupported type constraint')
			}
		}
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
				return type.level === Infinity ? map[type.name] ??= newTypeVariable(type.kind, type.bounds) : type
		}
	}(type)
}

function areEqualTypes(a: Type, b: Type): boolean {
	const aToB: Record<string, string> = Object.create(null)
	const bToA: Record<string, string> = Object.create(null)
	return function recurse(a: Type, b: Type): boolean {
		a = find(a)
		b = find(b)
		if (a.tag === 'namedType' && b.tag === 'namedType') {
			return a.name === b.name
		} else if (a.tag === 'reify' && b.tag === 'reify') {
			return recurse(a.generic, b.generic) && recurse(a.argument, b.argument)
		} else if (a.tag === 'typeVariable' && b.tag === 'typeVariable') {
			if (a.level !== b.level) return false
			if (!areEqualKinds(a.kind, b.kind)) return false
			if (a.level === Infinity) {
				if (a.name in aToB) {
					return b.name === aToB[a.name]
				} else if (b.name in bToA) {
					return false
				} else {
					aToB[a.name] = b.name
					bToA[b.name] = a.name
					return true
				}
			} else {
				return a.name === b.name
			}
		} else {
			return false
		}
	}(a, b)
}

function getKind(type: Type): Kind {
	if (type.tag === 'reify') {
		const kind = getKind(type.generic)
		if (!kind || !areEqualKinds(kind[0], getKind(type.argument))) {
			throw new Error('wrong kind')
		}
		return kind[1]
	} else {
		return type.kind
	}
}

function areEqualKinds(a: Kind, b: Kind): boolean {
	return a === b || !!a && !!b && areEqualKinds(a[0], b[0]) && areEqualKinds(a[1], b[1])
}

export type Scope = Record<string, Type>

function inferPattern(pattern: Pattern): { scope: Scope, type: Type } {
	switch (pattern.tag) {
		case 'wildcard':
			return { scope: {}, type: newTypeVariable() }
		case 'as':
			const result = inferPattern(pattern.pattern)
			result.scope[pattern.variableName] = result.type
			return result
		case 'literal':
			return { scope: {}, type: inferExpression({}, pattern) }
		case 'structure':
			const fields = pattern.fields.map(inferPattern)
			const resultType = newTypeVariable()
			unify(
				instantiate(pattern.constructor),
				functionType(...fields.map(x => x.type), resultType),
			)
			return {
				scope: Object.assign({}, ...fields.map(x => x.scope)),
				type: resultType,
			}
	}
}

function inferClauses(scope: Scope, clauses: Clause[]): Type {
	const resultType = newTypeVariable()
	for (const clause of clauses) {
		const parameters = clause.parameters.map(inferPattern)
		unify(
			resultType,
			functionType(
				...parameters.map(x => x.type),
				inferExpression(Object.assign({}, scope, ...parameters.map(x => x.scope)), clause.value),
			)
		)
	}
	return resultType
}

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
				innerScope[name] = expression.variables[name].type ?? newTypeVariable()
			}
			for (const name in expression.variables) if (!expression.variables[name].type) {
				unify(innerScope[name], inferClauses(innerScope, expression.variables[name].clauses))
			}
			currentLevel--
			for (const name in expression.variables) if (!expression.variables[name].type) {
				innerScope[name] = generalize(innerScope[name])
			}
			for (const name in expression.variables) if (expression.variables[name].type) {
				currentLevel++
				if (getKind(expression.variables[name].type) !== undefined) {
					throw new Error('incomplete type')
				}
				let type = instantiate(expression.variables[name].type)
				unify(type, inferClauses(innerScope, expression.variables[name].clauses))
				currentLevel--
				type = generalize(type)
				if (!areEqualTypes(expression.variables[name].type, type)) {
					throw new Error('annotation too general')
				}
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

export function inferProgram(scope: Scope, interfaces: InterfaceDefinitions, expression: Expression): Type {
	try {
		currentLevel = 0
		interfaceDefinitions = interfaces
		return recursiveFind(inferExpression(scope, expression))
	} finally {
		interfaceDefinitions = {}
	}
}
