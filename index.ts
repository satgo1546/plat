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

export function run(source: string) {
	hadError = false
	const expr = parse(tokenize(source))
	if (!expr) return
	console.log(pprint(expr))
}
