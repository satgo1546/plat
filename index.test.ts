import { test, expect } from 'vitest'

import { inferProgram, newTypeVariable, builtinTypes, functionType, type Expression, type Type, type Scope } from './index.ts'

function listType(element: Type): Type {
	return {
		tag: 'reify',
		generic: builtinTypes.list,
		argument: element,
	}
}

function pairType(left: Type, right: Type): Type {
	return {
		tag: 'reify',
		generic: {
			tag: 'reify',
			generic: builtinTypes.pair,
			argument: left,
		},
		argument: right,
	}
}

const T: Type = { tag: 'typeVariable', name: 'T', level: Infinity }
const T2: Type = { tag: 'typeVariable', name: 'T2', level: Infinity }
const std = {
	'+': functionType(builtinTypes.int, functionType(builtinTypes.int, builtinTypes.int)),
	'++': functionType(builtinTypes.string, functionType(builtinTypes.string, builtinTypes.string)),
	cons: functionType(T, functionType(listType(T), listType(T))),
	nil: listType(T),
	',': functionType(T, functionType(T2, pairType(T, T2))),
} satisfies Scope

function expr(x: string | number | Expression): Expression {
	if (typeof x === 'string') {
		return { tag: 'literal', value: { tag: x.length === 1 ? 'char' : 'string', value: x } }
	} else if (typeof x === 'number') {
		return { tag: 'literal', value: { tag: Number.isInteger(x) ? 'int' : 'float', value: x } }
	} else {
		return x
	}
}

type CallableExpression = Expression & ((...args: (string | number | Expression)[]) => Expression)

function makeCallable(x: Expression): CallableExpression {
	const f: CallableExpression = (...args) =>
		// @ts-ignore
		args.reduce((callee, x) => ({ tag: 'call', callee, argument: expr(x) }), f)
	Object.defineProperty(f, 'name', {
		value: undefined,
		writable: true,
		enumerable: true,
		configurable: true,
	})
	return Object.assign(f, x)
}

function $(name: string): CallableExpression {
	return makeCallable({ tag: 'variable', name })
}

const $a = $('a')
const $b = $('b')
const $c = $('c')
const $f = $('f')
const $g = $('g')
const $k = $('k')
const $n = $('n')
const $s = $('s')
const $x = $('x')
const $y = $('y')
const $z = $('z')
const $foo = $('foo')
const $bar = $('bar')
const $nil = $('nil')
const $cons = $('cons')
const $identity = $('identity')

function λ(parameterName: string, body: string | number | Expression): CallableExpression {
	return makeCallable({
		tag: 'function',
		parameterName,
		body: expr(body),
	})
}

function let$(variableName: string, variableValue: string | number | Expression, body: string | number | Expression): Expression {
	return {
		tag: 'let',
		variableName,
		variableValue: expr(variableValue),
		body: expr(body),
	}
}

const letrec$ = let$

function call(callee: Expression | string, ...args: (string | number | Expression)[]): Expression {
	return args.reduce<Expression>(
		(callee, x) => ({ tag: 'call', callee, argument: expr(x) }),
		typeof callee === 'string' ? { tag: 'variable', name: callee } : callee
	)
}

function plus(x: Expression | number, y: Expression | number): Expression {
	return call('+', expr(x), expr(y))
}

function concat(x: Expression | string, y: Expression | string): Expression {
	return call('++', expr(x), expr(y))
}

function pprint(x: Expression) {
	let out = ''
		; (function recurse(x: any) {
			if (x && typeof x === 'object' || typeof x === 'function') {
				out += '{'
				for (const key in x) if (x[key] !== undefined) {
					out += key
					out += ':'
					recurse(x[key])
					out += ','
				}
				out += '}'
			} else {
				out += JSON.stringify(x)
			}
		})(x)
	console.log(out)
}

// steal an unexported class out of Vitest
const AsymmetricMatcher = Object.getPrototypeOf(expect.anything().constructor)
class SomeTypeVariable extends AsymmetricMatcher {
	name?: string
	asymmetricMatch(other: any) {
		if (!this.name) {
			this.name = other?.name
		}
		return other?.tag === 'typeVariable' && other.name === this.name
	}
	toString() {
		return `SomeTypeVariable(${this.name})`
	}
	toAsymmetricMatcher() {
		return this.toString()
	}
}
const someTypeVariable = (): any => new SomeTypeVariable

test('someUndefinedVariable', () => {
	expect(() => inferProgram(std,
		$('someUndefinedVariable')
	)).toThrow('undefined')
})

test('let a = 10 in [a, a]', () => {
	expect(inferProgram(std,
		let$('a', 10, $cons($a, $cons($a, $nil)))
	)).toEqual(listType(builtinTypes.int))
})

test('let a = "foo" in let a = 10 in a + a', () => {
	expect(inferProgram(std,
		let$('a', 'foo', let$('a', 10, plus($a, $a)))
	)).toEqual(builtinTypes.int)
})

test("(\\x -> 'q')", () => {
	expect(inferProgram(std,
		λ('x', 'q')
	)).toEqual(functionType(someTypeVariable(), builtinTypes.char))
})

test('(\\x -> x)', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		λ('x', $x)
	)).toEqual(functionType(a, a))
})

test('(\\x -> x + x)', () => {
	expect(inferProgram(std,
		λ('x', plus($x, $x))
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('(\\x -> (x 123) + x)', () => {
	expect(() => inferProgram(std,
		λ('x', plus($x(123), $x))
	)).toThrow('match')
})

test('(\\foo -> foo 123)', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		λ('foo', $foo(123))
	)).toEqual(functionType(functionType(builtinTypes.int, a), a))
})

test('(\\foo -> foo 1 + foo 2)', () => {
	expect(inferProgram(std,
		λ('foo', plus($foo(1), $foo(2)))
	)).toEqual(functionType(
		functionType(builtinTypes.int, builtinTypes.int),
		builtinTypes.int,
	))
})

test('(\\a -> let b = a in b)', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		λ('a', let$('b', $a, $b))
	)).toEqual(functionType(a, a))
})

test('(\\a -> let b = a in b + b)', () => {
	expect(inferProgram(std,
		λ('a', let$('b', $a, plus($b, $b)))
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let a = 123 in (\\x -> a)', () => {
	expect(inferProgram(std,
		let$('a', 123, λ('x', $a))
	)).toEqual(functionType(someTypeVariable(), builtinTypes.int))
})

test('123 "foo"', () => {
	expect(() => inferProgram(std,
		call(expr(123), 'foo')
	)).toThrow('match')
})

test('(\\s -> "s is: " ++ s) 99', () => {
	expect(() => inferProgram(std,
		λ('s', concat(expr('s is: '), $s))(99)
	)).toThrow('match')
})

test('let identity = (\\x -> x) in identity "foo"', () => {
	expect(inferProgram(std,
		let$('identity', λ('x', $x), $identity('foo'))
	)).toEqual(builtinTypes.string)
})

test('(\\s -> let identity = (\\x -> x) in "foo" ++ (identity s))', () => {
	expect(inferProgram(std,
		λ('s', let$('identity', λ('x', $x), concat('foo', $identity($s))))
	)).toEqual(functionType(builtinTypes.string, builtinTypes.string))
})

test('(\\a -> let identity = (\\x -> x) in identity a)', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		λ('a', let$('identity', λ('x', $x), $identity($a)))
	)).toEqual(functionType(a, a))
})

test('(\\f -> f f)', () => {
	expect(() => inferProgram(std,
		λ('f', $f($f))
	)).toThrow('infinite')
})

test('let identity = (\\x -> x) in identity identity', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		let$('identity', λ('x', $x), $identity($identity))
	)).toEqual(functionType(a, a))
})

test('let identity = (\\x -> x) in (identity 42, identity "foo")', () => {
	expect(inferProgram(std,
		let$('identity', λ('x', $x), call(',', $identity(42), $identity('foo')))
	)).toEqual(pairType(builtinTypes.int, builtinTypes.string))
})

test('\\x -> let y = x in (y + 1, y ++ [])', () => {
	expect(() => inferProgram(std,
		λ('x', let$('y', $x, call(',', plus($y, 1), concat($y, $nil))))
	)).toThrow('match')
})

test('\\x -> let y = (\\z -> z) in y', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		λ('x', let$('y', λ('z', $z), $y))
	)).toEqual(functionType(someTypeVariable(), a, a))
})

test('\\x -> let y = x in y', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		λ('x', let$('y', $x, $y))
	)).toEqual(functionType(a, a))
})

test('\\x -> let y = (\\z -> x) in y', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		λ('x', let$('y', λ('z', $x), $y))
	)).toEqual(functionType(a, someTypeVariable(), a))
})

test('\\x -> \\y -> x y', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std,
		λ('x', λ('y', $x($y)))
	)).toEqual(functionType(functionType(a, b), a, b))
})

test('let c = (\\x -> \\y -> x y) in c', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std,
		let$('c', λ('x', λ('y', $x($y))), $c)
	)).toEqual(functionType(functionType(a, b), a, b))
})

test('let y = (\\z -> z) in y', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		let$('y', λ('z', $z), $y)
	)).toEqual(functionType(a, a))
})

test('\\x -> let y = (\\z -> z) in y', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std,
		λ('x', let$('y', λ('z', $z), $y))
	)).toEqual(functionType(a, functionType(b, b)))
})

test('\\x -> let y = (\\z -> z) in y x', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		λ('x', let$('y', λ('z', $z), $y($x)))
	)).toEqual(functionType(a, a))
})

test('\\x -> x x', () => {
	expect(() => inferProgram(std,
		λ('x', $x($x))
	)).toThrow('infinite')
})

test('let x = x in x', () => {
	expect(() => inferProgram(std,
		let$('x', $x, $x)
	)).toThrow('undefined')
})

test('\\y -> y (\\z -> y z)', () => {
	expect(() => inferProgram(std,
		λ('y', $y(λ('z', $y($z))))
	)).toThrow('infinite')
})

test('\\x -> \\y -> \\k -> k (k x y)', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std,
		λ('x', λ('y', λ('k', $k($k($x, $y)))))
	)).toEqual(functionType(a, b, functionType(a, b, a), b, a))
})

test('\\x -> \\y -> \\k -> k (k x y) (k y x)', () => {
	// This involves unifying two type variables with the same name.
	const a = someTypeVariable()
	expect(inferProgram(std,
		λ('x', λ('y', λ('k', $k($k($x, $y), $k($y, $x)))))
	)).toEqual(functionType(a, a, functionType(a, a, a), a))
})

test('let identity = (\\x -> x) in identity identity', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		let$('identity', λ('x', $x), $identity($identity))
	)).toEqual(functionType(a, a))
})

test('let x = (\\a -> \\b -> a b) in let y = let z = x (\\a -> a) in z in y', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		let$('x', λ('a', λ('b', $a($b))),
			let$('y', let$('z', $x(λ('a', $a)), $z), $y))
	)).toEqual(functionType(a, a))
})

test('\\x -> \\y -> let z = x y in \\x -> y x', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	const c = someTypeVariable()
	expect(inferProgram(std,
		λ('x', λ('y', let$('z', $x($y), λ('x', $y($x)))))
	)).toEqual(functionType(functionType(functionType(a, b), c), functionType(functionType(a, b), functionType(a, b))))
})

test('\\x -> let y = (\\z -> x z) in y', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std,
		λ('x', let$('y', λ('z', $x($z)), $y))
	)).toEqual(functionType(functionType(a, b), a, b))
})

test('\\x -> \\y -> let x = x y in x y', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std,
		λ('x', λ('y', let$('x', $x($y), $x($y))))
	)).toEqual(functionType(functionType(a, a, b), a, b))
})

test('\\x -> let y = x in y y', () => {
	expect(() => inferProgram(std,
		λ('x', let$('y', $x, $y($y)))
	)).toThrowError('infinite')
})

test('\\x -> let y = let z = x (\\x -> x) in z in y', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std,
		λ('x', let$('y', let$('z', $x(λ('x', $x)), $z), $y))
	)).toEqual(functionType(functionType(functionType(a, a), b), b))
})

// test("let foo = (\\a -> foo a) in foo 'x'", () => {
// 	expect(inferProgram(std,
// 		letrec$('foo', λ('a', $foo($a)), $foo('x'))
// 	)).toEqual(someTypeVariable())
// })

// test('let bar = bar in bar', () => {
// 	expect(inferProgram(std,
// 		letrec$('bar', $bar, $bar)
// 	)).toEqual(someTypeVariable())
// })

// test('let f x = 2 + f (x + 1) in f', () => {
// 	expect(inferProgram(std,
// 		letrec$('f', λ('x', plus(2, $f(plus($x, 1)))), $f)
// 	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
// })

// test('let f x = 2 + g x; g x = f (x + 1) in (f, g)', () => {
// 	expect(inferProgram(std,
// 		letrec$({
// 			f: λ('x', plus(2, $g($x))),
// 			g: λ('x', $f(plus($x, 1)))
// 		}, $cons($f, $cons($g, $nil)))
// 	)).toEqual(listType(functionType(builtinTypes.int, builtinTypes.int)))
// })

// test('let identity x = x; foo n = identity identity n in foo identity', () => {
// 	const a = someTypeVariable()
// 	expect(inferProgram(std,
// 		letrec$({
// 			identity: λ('x', $x),
// 			foo: λ('n', $identity($identity,$n))
// 		}, $foo($identity))
// 	)).toEqual(functionType(a, a))
// })

// test('let (f :: a -> a) = \\x -> x + 1 in f', () => {
// 	expect(() => inferProgram(std,
// 		letrec$('f', λ('x', plus($x, 1)), $f)
// 	)).toThrow('too general')
// })

// test('let a x = [b x]; b y = let foo = c "c" in y; c z = "foo" ++ a z in a', () => {
// 	const a = someTypeVariable()
// 	expect(inferProgram(std,
// 		letrec$({
// 			a: λ('x', $cons($b($x), $nil)),
// 			b: λ('y', let$('foo', $c('c'), $y)),
// 			c: λ('z', plus('foo', $a($z)))
// 		}, $a)
// 	)).toEqual(functionType(a, listType(a)))
// })
