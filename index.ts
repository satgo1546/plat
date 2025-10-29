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

export function run(source: string) {
	hadError = false
	console.log(tokenize(source))
}
