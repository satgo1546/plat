export let hadError = false
function error(line: number, message: string) {
	console.error(`Error: ${message} (line ${line})`)
	hadError = true
}

type Token = {
	type:
	// single-character tokens
	| '(' | ')' | '{' | '}' | ',' | '.' | '-' | '+' | ';' | '/' | '*'
	// one or two character tokens
	| '!' | '!=' | '=' | '==' | '>' | '>=' | '<' | '<='
	// literals
	| 'identifier' | 'string' | 'number'
	// keywords
	| 'nil' | 'true' | 'false' | 'var' | 'fun' | 'class' | 'this' | 'super'
	| 'and' | 'or' | 'if' | 'else' | 'for' | 'while' | 'return' | 'print'
	// end of file
	| undefined,
	lexeme: string,
	line: number,
	literal?: number | string,
}

const keywords = new Set<Token['type']>([
	'nil', 'true', 'false', 'var', 'fun', 'class', 'this', 'super',
	'and', 'or', 'if', 'else', 'for', 'while', 'return', 'print',
])

const digits = '0123456789'
const letters = '_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'

export function tokenize(source: string): Token[] {
	const tokens: Token[] = []
	let start = 0, current = 0
	const addToken = (type: Token['type'], literal?: any) => {
		const token = { type, lexeme: source.slice(start, current), line, literal }
		tokens.push(token)
		return token
	}
	const match = (expected: string): boolean => source[current] === expected && !!++current
	let line = 1
	while (current < source.length) {
		start = current
		const c = source[current++]
		switch (c) {
			case '(': case ')': case '{': case '}': case ',': case '.':
			case '-': case '+': case ';': case '*':
				addToken(c)
				break
			case '!':
				addToken(match('=') ? '!=' : '!')
				break
			case '=':
				addToken(match('=') ? '==' : '=')
				break
			case '<':
				addToken(match('=') ? '<=' : '<')
				break
			case '>':
				addToken(match('=') ? '>=' : '>')
				break
			case '/':
				if (match('/')) {
					while (current < source.length && source[current] !== '\n') current++
				} else {
					addToken('/')
				}
				break
			case ' ': case '\r': case '\t':
				break
			case '\n':
				line++
				break
			case '"':
				while (current < source.length && source[current] !== '"') current++
				if (current >= source.length) {
					error(line, 'unterminated string')
					break
				}
				addToken('string', source.slice(start + 1, current++))
				break
			default:
				if (digits.includes(c)) {
					while (current < source.length && digits.includes(source[current])) current++
					if (source[current] === '.' && digits.includes(source[current + 1])) {
						current++
						while (current < source.length && digits.includes(source[current])) current++
					}
					addToken('number', parseFloat(source.slice(start, current)))
				} else if (letters.includes(c)) {
					while (current < source.length && (letters + digits).includes(source[current])) current++
					const token = addToken('identifier')
					if (keywords.has(token.lexeme as any)) token.type = token.lexeme as any
				} else {
					error(line, 'unexpected character')
				}
		}
	}
	tokens.push({ type: undefined, lexeme: '', line })
	return tokens
}

type Expression =
	| { type: 'literal', value: number | string | boolean | undefined }
	| { type: 'grouping', expression: Expression }
	| { type: 'unary', operator: Token, right: Expression }
	| { type: 'binary', left: Expression, operator: Token, right: Expression }

export function pprint(expr: Expression): string {
	const parenthesize = (name: string, ...exprs: Expression[]) => `(${[name, ...exprs.map(pprint)].join(' ')})`
	switch (expr.type) {
		case 'literal':
			return String(expr.value ?? 'nil')
		case 'grouping':
			return parenthesize('group', expr.expression)
		case 'unary':
			return parenthesize(expr.operator.lexeme, expr.right)
		case 'binary':
			return parenthesize(expr.operator.lexeme, expr.left, expr.right)
	}
}

export function parse(tokens: Token[]): Expression | undefined {
	let current = 0
	function parseError(message: string): never {
		error(tokens[current].line, `${message} at \`${tokens[current].lexeme}\``)
		throw parseError
	}
	const match = (...types: Token['type'][]) => types.includes(tokens[current]?.type) && !!++current
	const equality = (): Expression => {
		let expr = comparison()
		while (match('==', '!=')) {
			expr = { type: 'binary', left: expr, operator: tokens[current - 1], right: comparison() }
		}
		return expr
	}
	const comparison = (): Expression => {
		let expr = term()
		while (match('<', '<=', '>', '>=')) {
			expr = { type: 'binary', left: expr, operator: tokens[current - 1], right: term() }
		}
		return expr
	}
	const term = (): Expression => {
		let expr = factor()
		while (match('+', '-')) {
			expr = { type: 'binary', left: expr, operator: tokens[current - 1], right: factor() }
		}
		return expr
	}
	const factor = (): Expression => {
		let expr = unary()
		while (match('*', '/')) {
			expr = { type: 'binary', left: expr, operator: tokens[current - 1], right: unary() }
		}
		return expr
	}
	const unary = (): Expression => {
		if (match('!', '-')) {
			return { type: 'unary', operator: tokens[current - 1], right: unary() }
		}
		if (match('+')) {
			error(tokens[current].line, 'unary `+` is not supported')
		}
		return primary()
	}
	const primary = (): Expression => {
		if (match('false')) return { type: 'literal', value: false }
		if (match('true')) return { type: 'literal', value: true }
		if (match('nil')) return { type: 'literal', value: undefined }
		if (match('number', 'string')) return {
			type: 'literal',
			value: tokens[current - 1].literal,
		}
		if (match('(')) {
			const expr = expression()
			if (tokens[current].type !== ')') {
				parseError('`)` expected after expression')
			}
			current++
			return { type: 'grouping', expression: expr }
		}
		parseError('expression expected')
	}
	const synchronize = () => {
		for (current++; ; current++) switch (tokens[current]?.type) {
			case ';':
				current++
				return
			case 'var':
			case 'fun':
			case 'class':
			case 'if':
			case 'for':
			case 'while':
			case 'print':
			case 'return':
			case undefined:
				return
		}
	}
	const expression = equality
	try {
		return expression()
	} catch (e) {
		if (e !== parseError) throw e
	}
}

type LoxObject = undefined | number | string | boolean

function isTruthy(object: LoxObject): boolean {
	return object !== undefined && object !== false
}

function isEqual(a: LoxObject, b: LoxObject): boolean {
	// Note that we have NaN == NaN and +0 != -0 in Lox.
	// https://docs.oracle.com/javase/8/docs/api/java/lang/Double.html#equals-java.lang.Object-
	return Object.is(a, b)
}

function stringify(object: LoxObject): string {
	if (object === undefined) return 'nil'
	return String(object)
}

export function evaluate(expr: Expression): LoxObject {
	switch (expr.type) {
		case 'literal':
			return expr.value
		case 'grouping':
			return evaluate(expr.expression)
		case 'unary': {
			const right = evaluate(expr.right)
			switch (expr.operator.type) {
				case '!':
					return !isTruthy(right)
				case '-':
					if (typeof right === 'number') {
						return -right
					}
					break
			}
			throw new RuntimeError(expr.operator, 'bad operand type')
		}
		case 'binary': {
			const left = evaluate(expr.left)
			const right = evaluate(expr.right)
			switch (expr.operator.type) {
				case '+':
					if (typeof left === 'number' && typeof right === 'number') {
						return left + right
					} else if (typeof left === 'string' || typeof right === 'string') {
						return stringify(left) + stringify(right)
					}
					break
				case '-':
					if (typeof left === 'number' && typeof right === 'number') {
						return left - right
					}
					break
				case '*':
					if (typeof left === 'number' && typeof right === 'number') {
						return left * right
					}
					break
				case '/':
					if (typeof left === 'number' && typeof right === 'number') {
						return left / right
					}
					break
				case '<':
					if (typeof left === 'number' && typeof right === 'number') {
						return left < right
					} else if (typeof left === 'string' && typeof right === 'string') {
						return left <right
					}
					break
				case '<=':
					if (typeof left === 'number' && typeof right === 'number') {
						return left <= right
					} else if (typeof left === 'string' && typeof right === 'string') {
						return left <= right
					}
					break
				case '>':
					if (typeof left === 'number' && typeof right === 'number') {
						return left > right
					} else if (typeof left === 'string' && typeof right === 'string') {
						return left > right
					}
					break
				case '>=':
					if (typeof left === 'number' && typeof right === 'number') {
						return left >= right
					} else if (typeof left === 'string' && typeof right === 'string') {
						return left >= right
					}
					break
				case '==':
					return isEqual(left, right)
				case '!=':
					return !isEqual(left, right)
			}
			throw new RuntimeError(expr.operator, 'bad operand type')
		}
		default:
			expr satisfies never
	}
}

export let hadRuntimeError = false
class RuntimeError extends Error {
	token: Token
	constructor(token: Token, message: string) {
		super(message)
		this.token = token
	}
}
export function run(source: string) {
	hadError = false
	hadRuntimeError = false
	const expr = parse(tokenize(source))
	if (!expr) return
	try {
		console.log(stringify(evaluate(expr)))
	} catch (e) {
		if (!(e instanceof RuntimeError)) throw e
		console.error(`Runtime error: ${e.message} at \`${e.token.lexeme}\` (line ${e.token.line})`)
		hadRuntimeError = true
	}
}
