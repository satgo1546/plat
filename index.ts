export type Variable = { tag: 'variable', name: string, type?: Type }

export type Expression =
	| Variable
	| { tag: 'number', value: number }
	| { tag: 'function', parameter: Variable, body: Expression }
	| { tag: 'call', callee: Expression, argument: Expression }

export type TypeVariable = { tag: 'typeVariable', name: string }

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

export type Scope = Record<string, Type>

export type VariableWithType = Variable & { type: Type }

type Constraint =
	| { tag: '=', 0: Type, 1: Type }

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
	return type
}
