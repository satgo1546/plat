export type Variable = { tag: 'variable', name: string, type?: Type }

export type Expression =
	| Variable
	| { tag: 'number', value: number }
	| { tag: 'function', parameter: Variable, body: Expression }
	| { tag: 'call', callee: Expression, argument: Expression }

export type TypeVariable = {
	tag: 'typeVariable',
	link?: Type,
	name: string,
}

export type Type =
	| TypeVariable
	| { tag: 'number' }
	| { tag: 'function', parameter: Type, returnType: Type }

export function functionType(...types: Type[]): Type {
	return types.reduceRight((result, parameter) => ({
		tag: 'function',
		parameter,
		returnType: result,
	}))
}

function find(type: Type): Type {
	switch (type.tag) {
		case 'typeVariable':
			if (type.link) {
				return type.link = find(type.link)
			}
			break
		case 'function':
			type.parameter = find(type.parameter)
			type.returnType = find(type.returnType)
			break
	}
	return type
}

function substitute_expression(expression: Expression) {
	switch (expression.tag) {
		case 'variable':
			if (expression.type) expression.type = find(expression.type)
			break
		case 'function':
			if (expression.parameter.type) expression.parameter.type = find(expression.parameter.type)
			substitute_expression(expression.body)
			break
		case 'call':
			substitute_expression(expression.callee)
			substitute_expression(expression.argument)
			break
	}
}

export type Scope = Record<string, Type>

type Constraint =
	| { tag: '=', 0: Type, 1: Type }

type TypeError =
	| { tag: 'typeMismatch', 0: Type, 1: Type }
	| { tag: 'infiniteType', typeVariable: TypeVariable, type: Type }

class Inferrer {
	constraints: Constraint[] = []

	newTypeVariable(): TypeVariable {
		return { tag: 'typeVariable', name: crypto.randomUUID() }
	}

	infer(scope: Scope, expression: Expression): Type {
		switch (expression.tag) {
			case 'variable':
				return expression.type = scope[expression.name]
			case 'number':
				return { tag: 'number' }
			case 'function': {
				const parameterType = expression.parameter.type = this.newTypeVariable()
				const returnType = this.infer({
					__proto__: null!,
					...scope,
					[expression.parameter.name]: parameterType,
				}, expression.body)
				return functionType(parameterType, returnType)
			}
			case 'call': {
				const argumentType = this.infer(scope, expression.argument)
				const returnType = this.newTypeVariable()
				const type = functionType(argumentType, returnType)
				this.check(scope, expression.callee, type)
				return returnType
			}
		}
	}

	check(scope: Scope, expression: Expression, type: Type): void {
		if (expression.tag === 'number' && type.tag === 'number') {
			// nothing to do
		} else if (expression.tag === 'function' && type.tag === 'function') {
			expression.parameter.type = type.parameter
			this.check({
				__proto__: null!,
				...scope,
				[expression.parameter.name]: type.parameter,
			}, expression.body, type.returnType)
		} else {
			const actualType = this.infer(scope, expression)
			this.constraints.push({ tag: '=', 0: type, 1: actualType })
		}
	}

	solve(): TypeError | undefined {
		for (const constraint of this.constraints) switch (constraint.tag) {
			case '=': {
				const error = this.unify(constraint[0], constraint[1])
				if (error) return error
				break
			}
		}
	}

	unify(a: Type, b: Type): TypeError | undefined {
		a = find(a)
		b = find(b)
		if (a.tag === 'number' && b.tag === 'number') {
			// nothing to do
		} else if (a.tag === 'function' && b.tag === 'function') {
			return this.unify(a.parameter, b.parameter)
				?? this.unify(a.returnType, b.returnType)
		} else if (a.tag === 'typeVariable' && b.tag === 'typeVariable') {
			if (a.name === b.name) return
			a.link = b
		} else if (a.tag === 'typeVariable') {
			const occurs = (function recurse(type: Type): boolean {
				type = find(type)
				switch (type.tag) {
					case 'typeVariable':
						return type.name === a.name
					case 'number':
						return false
					case 'function':
						return recurse(type.parameter) || recurse(type.returnType)
				}
			})(b)
			if (occurs) return { tag: 'infiniteType', typeVariable: a, type: b }
			a.link = b
		} else if (b.tag === 'typeVariable') {
			this.unify(b, a)
		} else {
			return { tag: 'typeMismatch', 0: a, 1: b }
		}
	}
}

/**
 * @param expression The expression to type.
 * Variables therein will be annotated with `type` field.
 */
export function infer(expression: Expression): Type {
	const inferrer = new Inferrer
	const type = inferrer.infer(Object.create(null), expression)
	console.log('Constraints:')
	for (const constraint of inferrer.constraints) {
		console.log('•', constraint)
	}
	const error = inferrer.solve()
	if (error) throw error
	substitute_expression(expression)
	return find(type)
}
