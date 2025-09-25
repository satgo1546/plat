import { test, expect } from 'vitest'

import {
	inferProgram,
	builtinTypes,
	functionType,
	type Expression,
	type Type,
	type Scope,
	type Pattern,
	type InterfaceDefinitions,
	type Kind,
} from './index.ts'

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

function newTypeParameter(kindOrBound?: Kind | string, ...bounds: string[]): Type & { tag: 'typeVariable' } {
	if (typeof kindOrBound === 'string') {
		bounds.unshift(kindOrBound)
		kindOrBound = undefined
	}
	return { tag: 'typeVariable', name: crypto.randomUUID(), kind: kindOrBound, bounds, level: Infinity }
}

const T: Type = newTypeParameter()
const T2: Type = newTypeParameter()
const TEq: Type = newTypeParameter('Eq')
const TOrd: Type = newTypeParameter('Ord')
const std = {
	true: builtinTypes.bool,
	false: builtinTypes.bool,
	'+': functionType(builtinTypes.int, functionType(builtinTypes.int, builtinTypes.int)),
	'++': functionType(builtinTypes.string, functionType(builtinTypes.string, builtinTypes.string)),
	cons: functionType(T, functionType(listType(T), listType(T))),
	nil: listType(T),
	',': functionType(T, functionType(T2, pairType(T, T2))),
	'==': functionType(TEq, functionType(TEq, builtinTypes.bool)),
	'<': functionType(TOrd, functionType(TOrd, builtinTypes.bool)),
	'||': functionType(builtinTypes.bool, functionType(builtinTypes.bool, builtinTypes.bool)),
	'read': functionType(builtinTypes.string, newTypeParameter('Read')),
	'show': functionType(newTypeParameter('Show'), builtinTypes.string),
} satisfies Scope

const stdi: InterfaceDefinitions = {
	Eq: {
		parents: [],
		implementations: [
			builtinTypes.null,
			builtinTypes.bool,
			builtinTypes.int,
			builtinTypes.char,
			builtinTypes.float,
			listType(newTypeParameter('Eq')),
			pairType(newTypeParameter('Eq'), newTypeParameter('Eq')),
		],
	},
	Ord: {
		parents: ['Eq'],
		implementations: [
			builtinTypes.null,
			builtinTypes.bool,
			builtinTypes.int,
			builtinTypes.char,
			builtinTypes.float,
			listType(newTypeParameter('Ord')),
			pairType(newTypeParameter('Ord'), newTypeParameter('Ord')),
		],
	},
	Read: {
		parents: [],
		implementations: [
			builtinTypes.null,
			builtinTypes.bool,
			builtinTypes.int,
			builtinTypes.char,
			builtinTypes.float,
			listType(newTypeParameter('Read')),
			pairType(newTypeParameter('Read'), newTypeParameter('Read')),
		],
	},
	Show: {
		parents: [],
		implementations: [
			builtinTypes.null,
			builtinTypes.bool,
			builtinTypes.int,
			builtinTypes.char,
			builtinTypes.float,
			listType(newTypeParameter('Show')),
			pairType(newTypeParameter('Show'), newTypeParameter('Show')),
		],
	},
	Functor: {
		parents: [],
		implementations: [
			// instance Functor []
			builtinTypes.list,
			// instance Functor ((,) a)
			{ tag: 'reify', generic: builtinTypes.pair, argument: newTypeParameter() },
		],
	},
	Applicative: {
		parents: ['Functor'],
		implementations: [
			builtinTypes.list,
			{ tag: 'reify', generic: builtinTypes.pair, argument: newTypeParameter() },
		],
	},
	Monad: {
		parents: ['Applicative'],
		implementations: [
			builtinTypes.list,
			{ tag: 'reify', generic: builtinTypes.pair, argument: newTypeParameter() },
		],
	},
}

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

function makeCallable<T extends Expression>(x: T): T & CallableExpression {
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

function $(name: string): CallableExpression & { tag: 'variable' } {
	return makeCallable({ tag: 'variable', name })
}

const $a = $('a')
const $b = $('b')
const $c = $('c')
const $f = $('f')
const $g = $('g')
const $k = $('k')
const $m = $('m')
const $n = $('n')
const $s = $('s')
const $x = $('x')
const $xs = $('xs')
const $y = $('y')
const $ys = $('ys')
const $z = $('z')
const $foo = $('foo')
const $bar = $('bar')
const $true = $('true')
const $false = $('false')
const $nil = $('nil')
const $cons = $('cons')
const $identity = $('identity')
const $read = $('read')
const $show = $('show')

function λ(parameterName: string, body: string | number | Expression): CallableExpression & { tag: 'function' } {
	return makeCallable({
		tag: 'function',
		parameterName,
		body: expr(body),
	})
}

function let$(variableName: string, variableValue: string | number | Expression, body: string | number | Expression): Expression {
	return {
		tag: 'let',
		variables: { [variableName]: { clauses: [{ parameters: [], value: expr(variableValue) }] } },
		body: expr(body),
	}
}

function letrec$(variables: Record<string,
	| string | number | Expression
	| [Type | undefined, string | number | Expression]
	| [Type | undefined, ...[...PatternLike[], string | number | Expression][]]
>, body: string | number | Expression): Expression {
	const result: Expression & { tag: 'let' } = {
		tag: 'let',
		variables: {},
		body: expr(body),
	}
	for (const name in variables) {
		if (Array.isArray(variables[name])) {
			if (Array.isArray(variables[name][1])) {
				result.variables[name] = {
					type: variables[name][0],
					clauses: variables[name].slice(1).map((x: any) => ({
						parameters: x.slice(0, -1).map(pattern),
						value: expr(x.at(-1)),
					})),
				}
			} else {
				result.variables[name] = {
					type: variables[name][0],
					clauses: [{ parameters: [], value: expr(variables[name][1]) }],
				}
			}
		} else {
			result.variables[name] = {
				clauses: [{ parameters: [], value: expr(variables[name]) }],
			}
		}
	}
	return result
}

type PatternLike = string | number | { tag: 'variable', name: string } | [Type, ...PatternLike[]] | Pattern
function pattern(x: PatternLike): Pattern {
	if (typeof x === 'string' || typeof x === 'number') {
		return expr(x) as Pattern
	} else if (Array.isArray(x)) {
		const [constructor, ...fields] = x as [Type, ...PatternLike[]]
		return { tag: 'structure', constructor, fields: fields.map(pattern) }
	} else if (x.tag === 'variable') {
		return { tag: 'as', variableName: x.name, pattern: { tag: 'wildcard' } }
	} else {
		return x
	}
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
	bounds: string = ''
	asymmetricMatch(other: any) {
		if (other?.tag !== 'typeVariable') return false
		return (this.name ??= other.name) === other.name
			&& other.kind === undefined
			&& this.bounds === JSON.stringify(other.bounds.toSorted())
	}
	toString() {
		return `SomeTypeVariable(${this.name ?? '?'}${this.bounds.length ? ': ' + this.bounds : ''})`
	}
	toAsymmetricMatcher() {
		return this.toString()
	}
}
const someTypeVariable = (...bounds: string[]): any => {
	const y = new SomeTypeVariable
	y.bounds = JSON.stringify(bounds.sort())
	return y
}

test('someUndefinedVariable', () => {
	expect(() => inferProgram(std, stdi,
		$('someUndefinedVariable')
	)).toThrow('undefined')
})

test('let x = 123 in x', () => {
	expect(inferProgram(std, stdi,
		let$('x', 123, $x)
	)).toEqual(builtinTypes.int)
})

test('let a = 10 in [a, a]', () => {
	expect(inferProgram(std, stdi,
		let$('a', 10, $cons($a, $cons($a, $nil)))
	)).toEqual(listType(builtinTypes.int))
})

test('let a = "foo" in let a = 10 in a + a', () => {
	expect(inferProgram(std, stdi,
		let$('a', 'foo', let$('a', 10, plus($a, $a)))
	)).toEqual(builtinTypes.int)
})

test("(\\x -> 'q')", () => {
	expect(inferProgram(std, stdi,
		λ('x', 'q')
	)).toEqual(functionType(someTypeVariable(), builtinTypes.char))
})

test('(\\x -> x)', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', $x)
	)).toEqual(functionType(a, a))
})

test('(\\x -> x + x)', () => {
	expect(inferProgram(std, stdi,
		λ('x', plus($x, $x))
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('(\\x -> (x 123) + x)', () => {
	expect(() => inferProgram(std, stdi,
		λ('x', plus($x(123), $x))
	)).toThrow('match')
})

test('(\\foo -> foo 123)', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('foo', $foo(123))
	)).toEqual(functionType(functionType(builtinTypes.int, a), a))
})

test('(\\foo -> foo 1 + foo 2)', () => {
	expect(inferProgram(std, stdi,
		λ('foo', plus($foo(1), $foo(2)))
	)).toEqual(functionType(
		functionType(builtinTypes.int, builtinTypes.int),
		builtinTypes.int,
	))
})

test('(\\a -> let b = a in b)', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('a', let$('b', $a, $b))
	)).toEqual(functionType(a, a))
})

test('(\\a -> let b = a in b + b)', () => {
	expect(inferProgram(std, stdi,
		λ('a', let$('b', $a, plus($b, $b)))
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let a = 123 in (\\x -> a)', () => {
	expect(inferProgram(std, stdi,
		let$('a', 123, λ('x', $a))
	)).toEqual(functionType(someTypeVariable(), builtinTypes.int))
})

test('123 "foo"', () => {
	expect(() => inferProgram(std, stdi,
		call(expr(123), 'foo')
	)).toThrow('match')
})

test('(\\s -> "s is: " ++ s) 99', () => {
	expect(() => inferProgram(std, stdi,
		λ('s', concat(expr('s is: '), $s))(99)
	)).toThrow('match')
})

test('let identity = (\\x -> x) in identity "foo"', () => {
	expect(inferProgram(std, stdi,
		let$('identity', λ('x', $x), $identity('foo'))
	)).toEqual(builtinTypes.string)
})

test('let identity x = x in identity "foo"', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			identity: [, [$x, $x]],
		}, $identity('foo'))
	)).toEqual(builtinTypes.string)
})

test('(\\s -> let identity = (\\x -> x) in "foo" ++ (identity s))', () => {
	expect(inferProgram(std, stdi,
		λ('s', let$('identity', λ('x', $x), concat('foo', $identity($s))))
	)).toEqual(functionType(builtinTypes.string, builtinTypes.string))
})

test('(\\a -> let identity = (\\x -> x) in identity a)', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('a', let$('identity', λ('x', $x), $identity($a)))
	)).toEqual(functionType(a, a))
})

test('(\\f -> f f)', () => {
	expect(() => inferProgram(std, stdi,
		λ('f', $f($f))
	)).toThrow('infinite')
})

test('let identity = (\\x -> x) in identity identity', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		let$('identity', λ('x', $x), $identity($identity))
	)).toEqual(functionType(a, a))
})

test('let identity = (\\x -> x) in (identity 42, identity "foo")', () => {
	expect(inferProgram(std, stdi,
		let$('identity', λ('x', $x), call(',', $identity(42), $identity('foo')))
	)).toEqual(pairType(builtinTypes.int, builtinTypes.string))
})

test('\\x -> let y = x in (y + 1, y ++ [])', () => {
	expect(() => inferProgram(std, stdi,
		λ('x', let$('y', $x, call(',', plus($y, 1), concat($y, $nil))))
	)).toThrow('match')
})

test('\\x -> let y = (\\z -> z) in y', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', let$('y', λ('z', $z), $y))
	)).toEqual(functionType(someTypeVariable(), a, a))
})

test('\\x -> let y = x in y', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', let$('y', $x, $y))
	)).toEqual(functionType(a, a))
})

test('\\x -> let y = (\\z -> x) in y', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', let$('y', λ('z', $x), $y))
	)).toEqual(functionType(a, someTypeVariable(), a))
})

test('\\x -> \\y -> x y', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', λ('y', $x($y)))
	)).toEqual(functionType(functionType(a, b), a, b))
})

test('let c = (\\x -> \\y -> x y) in c', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std, stdi,
		let$('c', λ('x', λ('y', $x($y))), $c)
	)).toEqual(functionType(functionType(a, b), a, b))
})

test('let y = (\\z -> z) in y', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		let$('y', λ('z', $z), $y)
	)).toEqual(functionType(a, a))
})

test('\\x -> let y = (\\z -> z) in y', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', let$('y', λ('z', $z), $y))
	)).toEqual(functionType(a, functionType(b, b)))
})

test('\\x -> let y = (\\z -> z) in y x', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', let$('y', λ('z', $z), $y($x)))
	)).toEqual(functionType(a, a))
})

test('\\x -> x x', () => {
	expect(() => inferProgram(std, stdi,
		λ('x', $x($x))
	)).toThrow('infinite')
})

test('\\y -> y (\\z -> y z)', () => {
	expect(() => inferProgram(std, stdi,
		λ('y', $y(λ('z', $y($z))))
	)).toThrow('infinite')
})

test('\\x -> \\y -> \\k -> k (k x y)', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', λ('y', λ('k', $k($k($x, $y)))))
	)).toEqual(functionType(a, b, functionType(a, b, a), b, a))
})

test('\\x -> \\y -> \\k -> k (k x y) (k y x)', () => {
	// This involves unifying two type variables with the same name.
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', λ('y', λ('k', $k($k($x, $y), $k($y, $x)))))
	)).toEqual(functionType(a, a, functionType(a, a, a), a))
})

test('let identity = (\\x -> x) in identity identity', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		let$('identity', λ('x', $x), $identity($identity))
	)).toEqual(functionType(a, a))
})

test('let x = (\\a -> \\b -> a b) in let y = let z = x (\\a -> a) in z in y', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		let$('x', λ('a', λ('b', $a($b))),
			let$('y', let$('z', $x(λ('a', $a)), $z), $y))
	)).toEqual(functionType(a, a))
})

test('\\x -> \\y -> let z = x y in \\x -> y x', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	const c = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', λ('y', let$('z', $x($y), λ('x', $y($x)))))
	)).toEqual(functionType(functionType(functionType(a, b), c), functionType(functionType(a, b), functionType(a, b))))
})

test('\\x -> let y = (\\z -> x z) in y', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', let$('y', λ('z', $x($z)), $y))
	)).toEqual(functionType(functionType(a, b), a, b))
})

test('\\x -> \\y -> let z = x y in z y', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', λ('y', let$('z', $x($y), $z($y))))
	)).toEqual(functionType(functionType(a, a, b), a, b))
})

test('\\x -> let y = x in y y', () => {
	expect(() => inferProgram(std, stdi,
		λ('x', let$('y', $x, $y($y)))
	)).toThrowError('infinite')
})

test('\\x -> let y = let z = x (\\x -> x) in z in y', () => {
	const a = someTypeVariable()
	const b = someTypeVariable()
	expect(inferProgram(std, stdi,
		λ('x', let$('y', let$('z', $x(λ('x', $x)), $z), $y))
	)).toEqual(functionType(functionType(functionType(a, a), b), b))
})

test("let foo = (\\a -> foo a) in foo 'x'", () => {
	expect(inferProgram(std, stdi,
		let$('foo', λ('a', $foo($a)), $foo('x'))
	)).toEqual(someTypeVariable())
})

test('let bar = bar in bar', () => {
	expect(inferProgram(std, stdi,
		let$('bar', $bar, $bar)
	)).toEqual(someTypeVariable())
})

test('let f x = 2 + f (x + 1) in f', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: [, [$x, plus(2, $f(plus($x, 1)))]],
		}, $f)
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let f x = 2 + g x; g x = f (x + 1) in f', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: [, [$x, plus(2, $g($x))]],
			g: [, [$x, $f(plus($x, 1))]],
		}, $f)
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let f x = 2 + g x; g x = f (x + 1) in g', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: [, [$x, plus(2, $g($x))]],
			g: [, [$x, $f(plus($x, 1))]],
		}, $g)
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let x = 123; y = 456 in [x, y]', () => {
	// x and y are not mutually recursive but it's okay.
	expect(inferProgram(std, stdi,
		letrec$({
			x: 123,
			y: 456,
		}, $cons($x, $cons($y, $nil)))
	)).toEqual(listType(builtinTypes.int))
})

test('let f x = 123; g y = 456 in [f, g]', () => {
	// f and g are not mutually recursive but it's okay.
	expect(inferProgram(std, stdi,
		letrec$({
			f: [, [$x, 123]],
			g: [, [$y, 456]],
		}, $cons($f, $cons($g, $nil)))
	)).toEqual(listType(functionType(someTypeVariable(), builtinTypes.int)))
})

test('let f x = 2 + g x; g x = f (x + 1) in [f, g]', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: [, [$x, plus(2, $g($x))]],
			g: [, [$x, $f(plus($x, 1))]],
		}, $cons($f, $cons($g, $nil)))
	)).toEqual(listType(functionType(builtinTypes.int, builtinTypes.int)))
})

test('let identity x = x in let foo n = identity identity n in foo identity', () => {
	// This would not type check:
	//   let identity x = x; foo n = identity identity n in foo identity
	// for identity and foo are not mutually recursive.
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		letrec$({ identity: [, [$x, $x]] },
			letrec$({ foo: [, [$n, $identity($identity, $n)]] },
				$foo($identity)))
	)).toEqual(functionType(a, a))
})

test('let f = \\n -> g (n - 1); g = \\n -> f (n - 1) in f', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: λ('n', $g(plus($n, -1))),
			g: λ('n', $f(plus($n, -1))),
		}, $f)
	)).toEqual(functionType(builtinTypes.int, someTypeVariable()))
})

test('let f = \\n -> g (n - 1); g = \\n -> f (n - 1) + 1 in f', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: λ('n', $g(plus($n, -1))),
			g: λ('n', plus($f(plus($n, -1)), 1)),
		}, $f)
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let f = \\n -> g (n - 1); g = \\n -> f (n - 1) in let a = \\n -> f n in a', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: λ('n', $g(plus($n, -1))),
			g: λ('n', $f(plus($n, -1))),
		}, let$('a', λ('n', $f($n)), $a))
	)).toEqual(functionType(builtinTypes.int, someTypeVariable()))
})

test('let f = \\n -> g (n - 1); g = \\n -> f (n - 1) in let a = \\n -> f n + b n; b = \\n -> g n + a n in a', () => {
	expect(inferProgram(std, stdi,
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
	expect(inferProgram(std, stdi,
		letrec$({
			x: [builtinTypes.int, 123],
		}, $x)
	)).toEqual(builtinTypes.int)
})

test('let (x :: [Int]) = [] in x', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			x: [listType(builtinTypes.int), $nil],
		}, $x)
	)).toEqual(listType(builtinTypes.int))
})

test('let (x :: [Int]) = []; (y :: String) = [] in y', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			x: [listType(builtinTypes.int), $nil],
			y: [builtinTypes.string, $nil],
		}, $y)
	)).toEqual(builtinTypes.string)
})

test('let (x :: a) = 123 in x', () => {
	expect(() => inferProgram(std, stdi,
		letrec$({
			x: [T, 123],
		}, $x)
	)).toThrow('too general')
})

test('let (x :: char) = 123 in x', () => {
	expect(() => inferProgram(std, stdi,
		letrec$({
			x: [builtinTypes.char, 123],
		}, $x)
	)).toThrow('mismatch')
})

test('let x :: [[Int]] -> Int; x n = 42 in x "foo"', () => {
	expect(() => inferProgram(std, stdi,
		letrec$({
			x: [listType(listType(builtinTypes.int)), 42],
		}, $x('foo'))
	)).toThrow('mismatch')
})

test('let (f :: a -> a) = \\x -> x + 1 in f', () => {
	expect(() => inferProgram(std, stdi,
		letrec$({
			f: [functionType(T, T), λ('x', plus($x, 1))],
		}, $f)
	)).toThrow('too general')
})

test('let a x = [b x]; b y = let foo = c \'c\' in y; c z = "foo" ++ a z in a', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			a: [, [$x, $cons($b($x), $nil)]],
			b: [, [$y, let$('foo', $c('c'), $y)]],
			c: [, [$z, concat('foo', $a($z))]],
		}, $a)
	)).toEqual(functionType(builtinTypes.char, builtinTypes.string))
})

test('let a x = [b x]; b :: t -> t; b y = let foo = c \'c\' in y; c z = "foo" ++ a z in b', () => {
	// Here, a :: Char -> String because a and c belong to the same binding group.
	// If the group is broken into smaller pieces, a would have type t -> [t].
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		letrec$({
			a: [, [$x, $cons($b($x), $nil)]],
			b: [functionType(T, T), [$y, let$('foo', $c('c'), $y)]],
			c: [, [$z, concat('foo', $a($z))]],
		}, $b)
	)).toEqual(functionType(a, a))
})

test('let a x = b x; b :: Int -> Int; b x = a (a x) in a', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			a: [, [$x, $b($x)]],
			b: [functionType(builtinTypes.int, builtinTypes.int), λ('x', $a($a($x)))],
		}, $a)
	)).toEqual(functionType(builtinTypes.int, builtinTypes.int))
})

test('let a x = b x; b :: t -> t; b x = let foo = a 123 in a x in a', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		letrec$({
			a: [, [$x, $b($x)]],
			b: [functionType(T, T), λ('x', let$('foo', $a(123), $a($x)))],
		}, $a)
	)).toEqual(functionType(a, a))
})

test('let f :: a -> b; f x = x in f', () => {
	expect(() => inferProgram(std, stdi,
		letrec$({
			f: [functionType(T, T2), λ('x', $x)],
		}, $f)
	)).toThrow('too general')
})

test('let f :: a -> a; f x = f x in f', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		letrec$({
			f: [functionType(T, T), λ('x', $f($x))],
		}, $f)
	)).toEqual(functionType(a, a))
})

test('let (a :: []) = \\x -> 123 in a', () => {
	expect(() => inferProgram(std, stdi,
		letrec$({
			a: [builtinTypes.list, λ('x', 123)],
		}, $a)
	)).toThrow('incomplete type')
})

test('let (a :: [] []) = nil in a', () => {
	expect(() => inferProgram(std, stdi,
		letrec$({
			a: [listType(builtinTypes.list), $nil],
		}, $a)
	)).toThrow('wrong kind')
})

test('data SomeKind a = SomeKind (a Int)\nlet unwrap :: a b -> b; unwrap = unwrap; getVal :: SomeKind t -> t Int; getVal = getVal in let foo y = (getVal y, unwrap y) in foo', () => {
	const t = newTypeParameter([, ,])
	expect(() => inferProgram(std, stdi,
		letrec$({
			unwrap: [functionType({ tag: 'reify', generic: t, argument: T }, T), $('unwrap')],
			getVal: [functionType(
				{ tag: 'reify', generic: { tag: 'namedType', name: 'SomeKind', kind: [[, ,], ,] }, argument: t },
				{ tag: 'reify', generic: t, argument: builtinTypes.int },
			), $('getVal')],
		}, let$('foo', λ('y', call(',', $('getVal')($y), $('unwrap')($y))), $foo))
	)).toThrow('kind mismatch')
})

test('let a 0 n = n + 1; a m 0 = a (m - 1) 1; a m n = a (m - 1) (a m (n - 1)) in a 3 4', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			a: [,
				[0, $n, plus($n, 1)],
				[$m, 0, $a(plus($m, -1), 1)],
				[$m, $n, $a(plus($m, -1), $a($m, plus($n, -1)))],
			],
		}, $a(3, 4))
	)).toEqual(builtinTypes.int)
})

test('let a :: Int -> Int -> Int; a 0 n = n + 1; a m 0 = a (m - 1) 1; a m n = a (m - 1) (a m (n - 1)) in a 3 4', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			a: [
				functionType(builtinTypes.int, builtinTypes.int, builtinTypes.int),
				[0, $n, plus($n, 1)],
				[$m, 0, $a(plus($m, -1), 1)],
				[$m, $n, $a(plus($m, -1), $a($m, plus($n, -1)))],
			],
		}, $a(3, 4))
	)).toEqual(builtinTypes.int)
})

test('let a :: t -> t; a 0 n = n + 1; a m 0 = a (m - 1) 1; a m n = a (m - 1) (a m (n - 1)) in a 3 4', () => {
	expect(() => inferProgram(std, stdi,
		letrec$({
			a: [
				functionType(T, T),
				[0, $n, plus($n, 1)],
				[$m, 0, $a(plus($m, -1), 1)],
				[$m, $n, $a(plus($m, -1), $a($m, plus($n, -1)))],
			],
		}, $a(3, 4))
	)).toThrow('match')
})

test('let f "" = 0; f (x:xs) = f xs + 1 in f', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: [,
				['', 0],
				[[std.cons, $x, $xs], plus($f($xs), 1)],
			],
		}, $f)
	)).toEqual(functionType(builtinTypes.string, builtinTypes.int))
})

test('let f [] = 0; f (x:xs) = f xs + 1 in f', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: [,
				[[std.nil], 0],
				[[std.cons, $x, $xs], plus($f($xs), 1)],
			],
		}, $f)
	)).toEqual(functionType(listType(someTypeVariable()), builtinTypes.int))
})

test('let f :: String -> Int; f [] = 0; f (x:xs) = f xs + 1 in f', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: [
				functionType(builtinTypes.string, builtinTypes.int),
				[[std.nil], 0],
				[[std.cons, $x, $xs], plus($f($xs), 1)],
			],
		}, $f)
	)).toEqual(functionType(builtinTypes.string, builtinTypes.int))
})

test('let g [] [] = 0; g [] (y:ys) = g [] ys - 1; g (x:xs) y = g xs y + 1 in g', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			g: [,
				[[std.nil], [std.nil], 0],
				[[std.nil], [std.cons, $y, $ys], plus($g($nil, $ys), -1)],
				[[std.cons, $x, $xs], $y, plus($g($xs, $y), 1)],
			],
		}, $g)
	)).toEqual(functionType(listType(someTypeVariable()), listType(someTypeVariable()), builtinTypes.int))
})

test('let g [] 0 = "foo" in g', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			g: [,
				[[std.nil], 0, 'foo'],
			],
		}, $g)
	)).toEqual(functionType(listType(someTypeVariable()), builtinTypes.int, builtinTypes.string))
})

test('let f x = 0; f = 1 in f', () => {
	expect(() => inferProgram(std, stdi,
		letrec$({
			f: [,
				[$x, 0],
				[1],
			],
		}, $f)
	)).toThrow('match')
})

test('let head x:xs = x in head', () => {
	const a = someTypeVariable()
	expect(inferProgram(std, stdi,
		letrec$({
			head: [,
				[[std.cons, $x, $xs], $x],
			],
		}, $('head'))
	)).toEqual(functionType(listType(a), a))
})

test('let f x y = x == y in f', () => {
	const a = someTypeVariable('Eq')
	expect(inferProgram(std, stdi,
		letrec$({
			f: [,
				[$x, $y, call('==', $x, $y)],
			],
		}, $f)
	)).toEqual(functionType(a, a, builtinTypes.bool))
})

test('\\x -> \\y -> x < y', () => {
	const a = someTypeVariable('Ord')
	expect(inferProgram(std, stdi,
		λ('x', λ('y', call('<', $x, $y)))
	)).toEqual(functionType(a, a, builtinTypes.bool))
})

test('let f x y = x == y || x < y || y == x || y < x in f', () => {
	const a = someTypeVariable('Ord')
	expect(inferProgram(std, stdi,
		letrec$({
			f: [,
				[$x, $y, call('||', call('||', call('||', call('==', $x, $y), call('<', $x, $y)), call('==', $y, $x)), call('<', $y, $x))],
			],
		}, $f)
	)).toEqual(functionType(a, a, builtinTypes.bool))
})

test('let f x y = x == y || x < y in f', () => {
	const a = someTypeVariable('Ord')
	expect(inferProgram(std, stdi,
		letrec$({
			f: [,
				[$x, $y, call('||', call('==', $x, $y), call('<', $x, $y))],
			],
		}, $f)
	)).toEqual(functionType(a, a, builtinTypes.bool))
})

test('1 == 1', () => {
	expect(inferProgram(std, stdi,
		call('==', 1, 1)
	)).toEqual(builtinTypes.bool)
})

test('123 == "123"', () => {
	expect(() => inferProgram(std, stdi,
		call('==', 123, '123')
	)).toThrow('match')
})

test('(\\x -> x) == (\\y -> y)', () => {
	expect(() => inferProgram(std, stdi,
		call('==', λ('x', $x), λ('y', $y))
	)).toThrow('implement')
})

test('("foo", "bar") < ("foo", "baz")', () => {
	expect(inferProgram(std, stdi,
		call('<', call(',', 'foo', 'bar'), call(',', 'foo', 'baz'))
	)).toEqual(builtinTypes.bool)
})

test('\\x -> \\y -> \\z -> x == (y, z)', () => {
	const a = someTypeVariable('Eq')
	const b = someTypeVariable('Eq')
	expect(inferProgram(std, stdi,
		λ('x', λ('y', λ('z', call('==', $x, call(',', $y, $z)))))
	)).toEqual(functionType(pairType(a, b), a, b, builtinTypes.bool))
})

test('let f x y z = x == (y, z) in f ("foo", 123)', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: [,
				[$x, $y, $z, call('==', $x, call(',', $y, $z))],
			],
		}, $f(call(',', 'foo', 123)))
	)).toEqual(functionType(builtinTypes.string, builtinTypes.int, builtinTypes.bool))
})

test('("foo", \\x -> x) == ("foo", \\y -> y)', () => {
	expect(() => inferProgram(std, stdi,
		call('==', call(',', 1, λ('x', $x)), call(',', 1, λ('y', $y)))
	)).toThrow('implement')
})

test('let f :: Functor a => a Int -> [a Int]; f = f in f []', () => {
	const a = newTypeParameter([, ,], 'Functor')
	expect(inferProgram(std, stdi,
		letrec$({
			f: [
				functionType(
					{ tag: 'reify', generic: a, argument: builtinTypes.int },
					listType({ tag: 'reify', generic: a, argument: builtinTypes.int }),
				),
				[$f],
			],
		}, $f($nil))
	)).toEqual(listType(listType(builtinTypes.int)))
})

test('let f :: a Int -> Bool; f x = True in let g x = x == x || f x in g', () => {
	// should be Eq (a Int) => a Int -> Bool
	expect(() => inferProgram(std, stdi,
		letrec$({
			f: [
				functionType(
					{ tag: 'reify', generic: newTypeParameter([, ,]), argument: builtinTypes.int },
					builtinTypes.bool,
				),
				[$x, $true],
			],
		}, letrec$({
			g: [,
				[$x, call('||', call('==', $x, $x), $f($x))],
			],
		}, $g))
	)).toThrow('unsupported')
})

test('let f (a:b:c) = a < b in f', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: [,
				[[std.cons, $a, [std.cons, $b, $c]], call('<', $a, $b)],
			],
		}, $f)
	)).toEqual(functionType(listType(someTypeVariable('Ord')), builtinTypes.bool))
})

test('let areTheSame x y = [x, y] == [y, x] in areTheSame', () => {
	const a = someTypeVariable('Eq')
	expect(inferProgram(std, stdi,
		letrec$({
			areTheSame: [,
				[$x, $y, call('==', $cons($x, $cons($y, $nil)), $cons($y, $cons($x, $nil)))],
			],
		}, $('areTheSame'))
	)).toEqual(functionType(a, a, builtinTypes.bool))
})

test('let f x = x < 1 in f', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			f: [,
				[$x, call('<', $x, 1)],
			],
		}, $f)
	)).toEqual(functionType(builtinTypes.int, builtinTypes.bool))
})

test('let foo x = let bar s = x < 1 || s < 1 in bar 0', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			foo: [,
				[$x, letrec$({
					bar: [,
						[$s, call('||', call('<', $x, 1), call('<', $s, 1))],
					],
				}, $bar(0))],
			],
		}, $foo)
	)).toEqual(functionType(builtinTypes.int, builtinTypes.bool))
})

test('let foo x = let bar s = show x ++ s ++ show x in (bar, bar ", ") in foo', () => {
	expect(inferProgram(std, stdi,
		letrec$({
			foo: [,
				[$x, letrec$({
					bar: [,
						[$s, concat(concat($show($x), $s), $show($x))],
					],
				}, call(',', $bar, $bar(', ')))],
			],
		}, $foo)
	)).toEqual(functionType(someTypeVariable('Show'), pairType(
		functionType(builtinTypes.string, builtinTypes.string),
		builtinTypes.string,
	)))
})

test('\\x -> show (read x)', () => {
	expect(() => inferProgram(std, stdi,
		λ('x', $show($read($x)))
	)).toThrow('ambi')
})

test('let f x = show (read x) in f', () => {
	expect(() => inferProgram(std, stdi,
		letrec$({
			f: [,
				[$x, $show($read($x))],
			],
		}, $f)
	)).toThrow('ambi')
})

test('let f x y = show (read x) < show (read y)', () => {
	expect(() => inferProgram(std, stdi,
		letrec$({
			f: [,
				[$x, $y, call('<', $show($read($x)), $show($read($y)))],
			],
		}, $f)
	)).toThrow('ambi')
})

test('let f x = read x == []', () => {
	expect(() => inferProgram(std, stdi,
		letrec$({
			f: [,
				[$x, call('==', $read($x), $nil)],
			],
		}, $f)
	)).toThrow('ambi')
})

test('show []', () => {
	expect(() => inferProgram(std, stdi,
		$show($nil)
	)).toThrow('ambi')
})

test('\\x -> read x', () => {
	expect(inferProgram(std, stdi,
		λ('x', $read($x))
	)).toEqual(functionType(builtinTypes.string, someTypeVariable('Read')))
})
