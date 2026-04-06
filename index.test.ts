import { test, expect, assert } from 'vitest'
import { type Expression, functionType, infer, type Variable } from './index.ts'

function v(variableName: string): Variable {
	return { tag: 'variable', name: variableName }
}

function n(value: number): Expression & { tag: 'number' } {
	return { tag: 'number', value }
}

function λ(parameterName: string, body: Expression): Expression & { tag: 'function' } {
	return { tag: 'function', parameter: v(parameterName), body }
}

function $(callee: Expression, arg0: Expression, ...args: Expression[]): Expression & { tag: 'call' } {
	return args.reduce<Expression & { tag: 'call' }>(
		(callee, argument) => ({ tag: 'call', callee, argument }),
		{ tag: 'call', callee, argument: arg0 },
	)
}

test('infers_int', () => {
	const expr = n(114514)
	const { type, errors } = infer(expr)
	expect(type).toStrictEqual({ tag: 'number' })
	expect(expr).toStrictEqual(n(114514))
	expect(errors.size).toBe(0)
})

test('infers_id_fun', () => {
	const expr = λ('x', v('x'))
	const { type, errors } = infer(expr)
	assert(type.tag === 'function')
	expect(type.parameter).toBe(type.returnType)
	expect(type.parameter.tag).toBe('typeVariable')
	expect(expr.parameter.type).toBe(type.parameter)
	expect(errors.size).toBe(0)
})

test('infers_k_combinator', () => {
	const expr = λ('x', λ('y', v('x')))
	const { type, errors } = infer(expr)
	assert(type.tag === 'function')
	assert(type.returnType.tag === 'function')
	expect(type.parameter).toBe(type.returnType.returnType)
	expect(type.parameter.tag).toBe('typeVariable')
	expect(type.returnType.parameter.tag).toBe('typeVariable')
	expect(expr.parameter.type).toBe(type.parameter)
	expect(errors.size).toBe(0)
})

test('infers_s_combinator', () => {
	const expr = λ('x', λ('y', λ('z', $($(v('x'), v('z')), $(v('y'), v('z'))))))
	const { type, errors } = infer(expr)
	assert(type.tag === 'function')
	assert(type.parameter.tag === 'function')
	assert(type.parameter.parameter.tag === 'typeVariable')
	assert(type.parameter.returnType.tag === 'function')
	assert(type.parameter.returnType.parameter.tag === 'typeVariable')
	assert(type.parameter.returnType.returnType.tag === 'typeVariable')
	assert(type.returnType.tag === 'function')
	assert(type.returnType.parameter.tag === 'function')
	assert(type.returnType.parameter.parameter.tag === 'typeVariable')
	assert(type.returnType.parameter.returnType.tag === 'typeVariable')
	assert(type.returnType.returnType.tag === 'function')
	assert(type.returnType.returnType.parameter.tag === 'typeVariable')
	assert(type.returnType.returnType.returnType.tag === 'typeVariable')
	expect(type.parameter.parameter).toBe(type.returnType.parameter.parameter)
	expect(type.parameter.parameter).toBe(type.returnType.returnType.parameter)
	expect(type.parameter.parameter).not.toBe(type.parameter.returnType.parameter)
	expect(type.parameter.parameter).not.toBe(type.returnType.returnType.returnType)
	expect(type.parameter.returnType.parameter).toBe(type.returnType.parameter.returnType)
	expect(type.returnType.returnType.returnType).toBe(type.parameter.returnType.returnType)
	expect(type.returnType.returnType.returnType).not.toBe(type.parameter.returnType.parameter)
	expect(expr.parameter.type).toBe(type.parameter)
	expect(errors.size).toBe(0)
})

test('type_infer_fails', () => {
	const expr = $(λ('x', $(v('x'), n(3))), n(1))
	const { type, errors } = infer(expr)
	expect(type).toBeTruthy()
	expect(errors.size).toBe(1)
	expect([...errors.values()][0]).toStrictEqual({
		tag: 'typeMismatch',
		checked: functionType({ tag: 'number' }, { tag: 'typeVariable', name: expect.any(String) }),
		inferred: { tag: 'number' },
		message: 'type mismatch',
	})
})

test('type_infer_fails_with_meaningful_error', () => {
	const expr = $(λ('y', $(v('y'), n(3), n(4))), λ('f', λ('x', $(v('f'), v('x')))))
	const { type, errors } = infer(expr)
	expect(type).toBeTruthy()
	expect(errors.size).toBe(1)
	expect([...errors.values()][0]).toStrictEqual({
		tag: 'typeMismatch',
		checked: { tag: 'number' },
		inferred: functionType({ tag: 'number' }, { tag: 'typeVariable', name: expect.any(String) }),
		message: 'expected function',
	})
})
