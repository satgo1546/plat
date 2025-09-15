import { test, expect } from 'vitest'

import { inferProgram, builtinTypes, functionType, type Expression, type Type, type Scope } from './index.ts'

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
		variables: { [variableName]: { value: expr(variableValue) } },
		body: expr(body),
	}
}

function letrec$(variables: Record<string, string | number | Expression | [Type, string | number | Expression]>, body: string | number | Expression): Expression {
	const result: Expression & { tag: 'let' } = {
		tag: 'let',
		variables: {},
		body: expr(body),
	}
	for (const name in variables) {
		if (Array.isArray(variables[name])) {
			result.variables[name] = {
				type: variables[name][0],
				value: expr(variables[name][1]),
			}
		} else {
			result.variables[name] = {
				value: expr(variables[name]),
			}
		}
	}
	return result
}

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
		if (other?.tag !== 'typeVariable') return false
		return (this.name ??= other.name) === other.name
	}
	toString() {
		return `SomeTypeVariable(${this.name ?? '?'})`
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

test('let x = 123 in x', () => {
	expect(inferProgram(std,
		let$('x', 123, $x)
	)).toEqual(builtinTypes.int)
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

test('\\x -> \\y -> let z = x y in z y', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std,
		λ('x', λ('y', let$('z', $x($y), $z($y))))
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

test("let foo = (\\a -> foo a) in foo 'x'", () => {
	expect(inferProgram(std,
		let$('foo', λ('a', $foo($a)), $foo('x'))
	)).toEqual(someTypeVariable())
})

test('let bar = bar in bar', () => {
	expect(inferProgram(std,
		let$('bar', $bar, $bar)
	)).toEqual(someTypeVariable())
})

test('let f x = 2 + f (x + 1) in f', () => {
	expect(inferProgram(std,
		let$('f', λ('x', plus(2, $f(plus($x, 1)))), $f)
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let f x = 2 + g x; g x = f (x + 1) in f', () => {
	expect(inferProgram(std,
		letrec$({
			f: λ('x', plus(2, $g($x))),
			g: λ('x', $f(plus($x, 1))),
		}, $f)
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let f x = 2 + g x; g x = f (x + 1) in g', () => {
	expect(inferProgram(std,
		letrec$({
			f: λ('x', plus(2, $g($x))),
			g: λ('x', $f(plus($x, 1))),
		}, $g)
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let x = 123; y = 456 in [x, y]', () => {
	// x and y are not mutually recursive but it's okay.
	expect(inferProgram(std,
		letrec$({
			x: 123,
			y: 456,
		}, $cons($x, $cons($y, $nil)))
	)).toEqual(listType(builtinTypes.int))
})

test('let f x = 123; g y = 456 in [f, g]', () => {
	// f and g are not mutually recursive but it's okay.
	expect(inferProgram(std,
		letrec$({
			f: λ('x', 123),
			g: λ('y', 456),
		}, $cons($f, $cons($g, $nil)))
	)).toEqual(listType(functionType(someTypeVariable(), builtinTypes.int)))
})

test('let f x = 2 + g x; g x = f (x + 1) in [f, g]', () => {
	expect(inferProgram(std,
		letrec$({
			f: λ('x', plus(2, $g($x))),
			g: λ('x', $f(plus($x, 1))),
		}, $cons($f, $cons($g, $nil)))
	)).toEqual(listType(functionType(builtinTypes.int, builtinTypes.int)))
})

test('let identity x = x in let foo n = identity identity n in foo identity', () => {
	// This would not type check:
	//   let identity x = x; foo n = identity identity n in foo identity
	// for identity and foo are not mutually recursive.
	const a = someTypeVariable()
	expect(inferProgram(std,
		let$('identity', λ('x', $x),
			let$('foo', λ('n', $identity($identity, $n)),
				$foo($identity)))
	)).toEqual(functionType(a, a))
})

test('let f n = g (n - 1); g n = f (n - 1) in f', () => {
	expect(inferProgram(std,
		letrec$({
			f: λ('n', $g(plus($n, -1))),
			g: λ('n', $f(plus($n, -1))),
		}, $f)
	)).toEqual(functionType(builtinTypes.int, someTypeVariable()))
})

test('let f n = g (n - 1); g n = f (n - 1) + 1 in f', () => {
	expect(inferProgram(std,
		letrec$({
			f: λ('n', $g(plus($n, -1))),
			g: λ('n', plus($f(plus($n, -1)), 1)),
		}, $f)
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let f n = g (n - 1); g n = f (n - 1) in let a n = f n in a', () => {
	expect(inferProgram(std,
		letrec$({
			f: λ('n', $g(plus($n, -1))),
			g: λ('n', $f(plus($n, -1))),
		}, let$('a', λ('n', $f($n)), $a))
	)).toEqual(functionType(builtinTypes.int, someTypeVariable()))
})

test('let f n = g (n - 1); g n = f (n - 1) in let a n = f n + b n; b n = g n + a n in a', () => {
	expect(inferProgram(std,
		letrec$({
			f: λ('n', $g(plus($n, -1))),
			g: λ('n', $f(plus($n, -1))),
		}, letrec$({
			a: λ('n', plus($f($n), $b($n))),
			b: λ('n', plus($g($n), $a($n))),
		}, $a))
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let (x :: Int) = 123 in x', () => {
	expect(inferProgram(std,
		letrec$({
			x: [builtinTypes.int, 123],
		}, $x)
	)).toEqual(builtinTypes.int)
})

test('let (x :: [Int]) = [] in x', () => {
	expect(inferProgram(std,
		letrec$({
			x: [listType(builtinTypes.int), $nil],
		}, $x)
	)).toEqual(listType(builtinTypes.int))
})

test('let (x :: [Int]) = []; (y :: String) = [] in y', () => {
	expect(inferProgram(std,
		letrec$({
			x: [listType(builtinTypes.int), $nil],
			y: [builtinTypes.string, $nil],
		}, $y)
	)).toEqual(builtinTypes.string)
})

test('let (x :: a) = 123 in x', () => {
	expect(() => inferProgram(std,
		letrec$({
			x: [T, 123],
		}, $x)
	)).toThrow('too general')
})

test('let (x :: char) = 123 in x', () => {
	expect(() => inferProgram(std,
		letrec$({
			x: [builtinTypes.char, 123],
		}, $x)
	)).toThrow('mismatch')
})

test('let x :: [[Int]] -> Int; x n = 42 in x "foo"', () => {
	expect(() => inferProgram(std,
		letrec$({
			x: [listType(listType(builtinTypes.int)), 42],
		}, $x('foo'))
	)).toThrow('mismatch')
})

test('let (f :: a -> a) = \\x -> x + 1 in f', () => {
	expect(() => inferProgram(std,
		letrec$({
			f: [functionType(T, T), λ('x', plus($x, 1))],
		}, $f)
	)).toThrow('too general')
})

test('let a x = [b x]; b y = let foo = c \'c\' in y; c z = "foo" ++ a z in a', () => {
	expect(inferProgram(std,
		letrec$({
			a: λ('x', $cons($b($x), $nil)),
			b: λ('y', let$('foo', $c('c'), $y)),
			c: λ('z', concat('foo', $a($z)))
		}, $a)
	)).toEqual(functionType(builtinTypes.char, builtinTypes.string))
})

test('let a x = [b x]; b :: t -> t; b y = let foo = c \'c\' in y; c z = "foo" ++ a z in b', () => {
	// Here, a :: Char -> String because a and c belong to the same binding group.
	// If the group is broken into smaller pieces, a would have type t -> [t].
	const a = someTypeVariable()
	expect(inferProgram(std,
		letrec$({
			a: λ('x', $cons($b($x), $nil)),
			b: [functionType(T, T), λ('y', let$('foo', $c('c'), $y))],
			c: λ('z', concat('foo', $a($z)))
		}, $b)
	)).toEqual(functionType(a, a))
})

test('let a x = b x; b :: Int -> Int; b x = a (a x) in a', () => {
	expect(inferProgram(std,
		letrec$({
			a: λ('x', $b($x)),
			b: [functionType(builtinTypes.int, builtinTypes.int), λ('x', $a($a($x)))],
		}, $a)
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let a x = b x; b :: t -> t; b x = let foo = a 123 in a x in a', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		letrec$({
			a: λ('x', $b($x)),
			b: [functionType(T, T), λ('x', let$('foo', $a(123), $a($x)))],
		}, $a)
	)).toEqual(functionType(a, a))
})

test('let f :: a -> b; f x = x in f', () => {
	expect(() => inferProgram(std,
		letrec$({
			f: [functionType(T, T2), λ('x', $x)],
		}, $f)
	)).toThrow('too general')
})

test('let f :: a -> a; f x = f x in f', () => {
	const a = someTypeVariable()
	expect(inferProgram(std,
		letrec$({
			f: [functionType(T, T), λ('x', $f($x))],
		}, $f)
	)).toEqual(functionType(a, a))
})

test('let (a :: []) = \\x -> 123 in a', () => {
	expect(() => inferProgram(std,
		letrec$({
			a: [builtinTypes.list, λ('x', 123)],
		}, $a)
	)).toThrow('incomplete type')
})

test('let (a :: [] []) = nil in a', () => {
	expect(() => inferProgram(std,
		letrec$({
			a: [listType(builtinTypes.list), $nil],
		}, $a)
	)).toThrow('wrong kind')
})

test('data SomeKind a = SomeKind (a Int)\nlet unwrap :: a b -> b; unwrap = unwrap; getVal :: SomeKind t -> t Int; getVal = getVal in let foo y = (getVal y, unwrap y) in foo', () => {
	const t: Type = { tag: 'typeVariable', name: 't', kind: [, ,], level: Infinity }
	expect(() => inferProgram(std,
		letrec$({
			unwrap: [functionType({ tag: 'reify', generic: t, argument: T }, T), $('unwrap')],
			getVal: [functionType(
				{ tag: 'reify', generic: { tag: 'namedType', name: 'SomeKind', kind: [[, ,], ,] }, argument: t },
				{ tag: 'reify', generic: t, argument: builtinTypes.int },
			), $('getVal')],
		}, let$('foo', λ('y', call(',', $('getVal')($y), $('unwrap')($y))), $foo))
	)).toThrow('kind mismatch')
})
