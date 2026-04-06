export type Variable = { tag: 'variable', name: string, type?: Type }

export type Expression =
	| Variable
	| { tag: 'number', value: number }
	| { tag: 'function', parameter: Variable, body: Expression }
	| { tag: 'call', callee: Expression, argument: Expression }
	| { tag: 'error', variable: Variable }

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
		case 'error':
			if (expression.variable.type) expression.variable.type = find(expression.variable.type)
			break
	}
}

export type Scope = Record<string, Type>

type Constraint =
	| { tag: '=', 0: Type, 1: Type, expression: Expression, message: string }

type TypeError =
	| { tag: 'typeMismatch', checked: Type, inferred: Type, message: string }
	| { tag: 'infiniteType', typeVariable: TypeVariable, type: Type }

class Inferrer {
	constraints: Constraint[] = []
	errors = new Map<Expression, TypeError>

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
				let calleeType = this.infer(scope, expression.callee)
				if (calleeType.tag !== 'function') {
					this.constraints.push({
						tag: '=',
						expression,
						0: calleeType,
						1: calleeType = {
							tag: 'function',
							parameter: this.newTypeVariable(),
							returnType: this.newTypeVariable(),
						},
						message: 'expected function',
					})
				}
				this.check(scope, expression.argument, calleeType.parameter)
				return calleeType.returnType
			}
			case 'error':
				return expression.variable.type = this.newTypeVariable()
		}
	}

	check(scope: Scope, expression: Expression, type: Type): void {
		if (expression.tag === 'number' && type.tag === 'number') {
			// nothing to do
		} else if (expression.tag === 'function') {
			if (type.tag !== 'function') {
				this.constraints.push({
					tag: '=',
					expression,
					0: type,
					1: type = {
						tag: 'function',
						parameter: this.newTypeVariable(),
						returnType: this.newTypeVariable(),
					},
					message: 'unexpected function',
				})
			}
			expression.parameter.type = type.parameter
			this.check({
				__proto__: null!,
				...scope,
				[expression.parameter.name]: type.parameter,
			}, expression.body, type.returnType)
		} else {
			const actualType = this.infer(scope, expression)
			this.constraints.push({ tag: '=', expression, 0: type, 1: actualType, message: 'type mismatch' })
		}
	}

	solve(): Map<Expression, TypeError> {
		for (const constraint of this.constraints) switch (constraint.tag) {
			case '=': {
				const error = this.unify(constraint[0], constraint[1])
				if (error?.tag === 'typeMismatch') {
					error.message ||= constraint.message
				}
				if (error) {
					this.errors.set(constraint.expression, error)
				}
				break
			}
		}
		for (const error of this.errors.values()) {
			if (error.tag === 'typeMismatch') {
				error.checked = find(error.checked)
				error.inferred = find(error.inferred)
			}
		}
		return this.errors
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
			return { tag: 'typeMismatch', checked: a, inferred: b, message: '' }
		}
	}
}

/**
 * @param expression The expression to type.
 * Variables therein will be annotated with `type` field.
 */
export function infer(expression: Expression): {
	type: Type,
	errors: Map<Expression, TypeError>
} {
	const inferrer = new Inferrer
	const type = inferrer.infer(Object.create(null), expression)
	console.log('Constraints:')
	for (const constraint of inferrer.constraints) {
		console.log('•', constraint)
	}
	const errors = inferrer.solve()
	substitute_expression(expression)
	return { type: find(type), errors }
}
