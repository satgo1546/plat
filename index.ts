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
	| { type: 'variable', name: Token }
	| { type: 'assign', name: Token, value: Expression }
	| { type: 'call', callee: Expression, paren: Token, arguments: Expression[] }
	| { type: 'get', object: Expression, name: Token }
	| { type: 'set', object: Expression, name: Token, value: Expression }

type Statement =
	| { type: 'block', statements: Statement[] }
	| { type: 'expression', expression: Expression }
	| { type: 'print', expression: Expression }
	| { type: 'var', name: Token, initializer?: Expression }
	| { type: 'function', name: Token, params: Token[], body: Statement[] }
	| { type: 'class', name: Token, methods: (Statement & { type: 'function' })[] }
	| { type: 'if', condition: Expression, thenBranch: Statement, elseBranch?: Statement }
	| { type: 'while', condition: Expression, body: Statement }
	| { type: 'return', keyword: Token, value?: Expression }

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
		case 'variable':
			return expr.name.lexeme
		case 'assign':
			return parenthesize(expr.name.lexeme + '=', expr.value)
		case 'call':
			return parenthesize('call', expr.callee, ...expr.arguments)
		case 'get':
			return parenthesize('.' + expr.name, expr.object)
		case 'set':
			return parenthesize('.' + expr.name + '=', expr.object)
	}
}

export function parse(tokens: Token[]): Statement[] {
	let current = 0
	function parseError(message: string): never {
		error(tokens[current].line, `${message} at \`${tokens[current].lexeme}\``)
		throw parseError
	}
	const match = (...types: Token['type'][]) => types.includes(tokens[current]?.type) && !!++current
	const consume = (type: Token['type'], message: string) => {
		if (tokens[current]?.type !== type) parseError(message)
		return tokens[current++]
	}
	const assignment = (): Expression => {
		const expr = or()
		if (match('=')) {
			const equals = tokens[current - 1]
			const value = assignment()
			if (expr.type === 'variable') {
				return { type: 'assign', name: expr.name, value }
			} else if (expr.type === 'get') {
				return { type: 'set', object: expr.object, name: expr.name, value }
			}
			error(equals.line, 'invalid assignment target')
		}
		return expr
	}
	const or = (): Expression => {
		let expr = and()
		while (match('or')) {
			expr = { type: 'binary', left: expr, operator: tokens[current - 1], right: and() }
		}
		return expr
	}
	const and = (): Expression => {
		let expr = equality()
		while (match('and')) {
			expr = { type: 'binary', left: expr, operator: tokens[current - 1], right: equality() }
		}
		return expr
	}
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
		return call()
	}
	const call = (): Expression => {
		let expr = primary()
		for (; ;) {
			if (match('(')) {
				const args: Expression[] = []
				do {
					if (tokens[current].type === ')') break
					if (args.length >= 255) {
						error(tokens[current].line, 'too many arguments')
					}
					args.push(expression())
				} while (match(','))
				expr = {
					type: 'call',
					callee: expr,
					paren: consume(')', '`)` expected after arguments'),
					arguments: args,
				}
			} else if (match('.')) {
				expr = {
					type: 'get',
					object: expr,
					name: consume('identifier', 'property name expected after `.`'),
				}
			} else {
				break
			}
		}
		return expr
	}
	const primary = (): Expression => {
		if (match('false')) return { type: 'literal', value: false }
		if (match('true')) return { type: 'literal', value: true }
		if (match('nil')) return { type: 'literal', value: undefined }
		if (match('number', 'string')) return {
			type: 'literal',
			value: tokens[current - 1].literal,
		}
		if (match('identifier')) return {
			type: 'variable',
			name: tokens[current - 1],
		}
		if (match('(')) {
			const expr = expression()
			consume(')', '`)` expected after expression')
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
	const expression = assignment
	const block = (): Statement[] => {
		const statements: Statement[] = []
		while (current < tokens.length && tokens[current].type !== '}') {
			const stmt = declaration()
			if (stmt) statements.push(stmt)
		}
		consume('}', '`}` expected after block')
		return statements
	}
	const statement = (): Statement => {
		if (match('print')) {
			const value = expression()
			consume(';', '`;` expected after value to be printed')
			return { type: 'print', expression: value }
		}
		if (match('{')) {
			return { type: 'block', statements: block() }
		}
		if (match('if')) {
			consume('(', '`(` expected after `if`')
			const condition = expression()
			consume(')', '`)` expected after if condition')
			return {
				type: 'if',
				condition,
				thenBranch: statement(),
				elseBranch: match('else') ? statement() : undefined,
			}
		}
		if (match('for')) {
			consume('(', '`(` expected after `for`')
			let initializer: Statement | undefined
			if (match('var')) {
				initializer = varDeclaration()
			} else if (!match(';')) {
				initializer = { type: 'expression', expression: expression() }
				consume(';', '`;` expected after for initializer')
			}
			let condition: Expression = { type: 'literal', value: true }
			if (!match(';')) {
				condition = expression()
				consume(';', '`;` expected after for condition')
			}
			let increment: Expression | undefined
			if (!match(')')) {
				increment = expression()
				consume(')', '`)` expected after for clauses')
			}
			let body = statement()
			if (increment) body = {
				type: 'block',
				statements: [
					body,
					{ type: 'expression', expression: increment },
				],
			}
			body = { type: 'while', condition, body }
			if (initializer) body = {
				type: 'block',
				statements: [
					initializer,
					body,
				],
			}
			return body
		}
		if (match('while')) {
			consume('(', '`(` expected after `while`')
			const condition = expression()
			consume(')', '`)` expected after while condition')
			return {
				type: 'while',
				condition,
				body: statement(),
			}
		}
		if (match('return')) {
			const keyword = tokens[current - 1]
			let value: Expression | undefined
			if (!match(';')) {
				value = expression()
				consume(';', '`;` expected after return value')
			}
			return { type: 'return', keyword, value }
		}
		const value = expression()
		consume(';', '`;` expected after expression')
		return { type: 'expression', expression: value }
	}
	const varDeclaration = (): Statement => {
		const name = consume('identifier', 'variable name expected')
		let initializer
		if (match('=')) {
			initializer = expression()
		}
		consume(';', '`;` expected after variable declaration')
		return { type: 'var', name, initializer }
	}
	const funDeclaration = (kind: string): Statement & { type: 'function' } => {
		const name = consume('identifier', `${kind} name expected`)
		consume('(', `\`(\` expected after ${kind} name`)
		const params: Token[] = []
		do {
			if (tokens[current].type === ')') break
			if (params.length >= 255) {
				error(tokens[current].line, 'too many parameters')
			}
			params.push(consume('identifier', 'parameter name expected'))
		} while (match(','))
		consume(')', '`)` expected after parameters')
		consume('{', `\`{\` expected before ${kind} body`)
		return { type: 'function', name, params, body: block() }
	}
	const declaration = (): Statement | undefined => {
		try {
			if (match('var')) return varDeclaration()
			if (match('fun')) return funDeclaration('function')
			if (match('class')) {
				const name = consume('identifier', 'class name expected')
				consume('{', '`{` expected before class body')
				const methods: (Statement & { type: 'function' })[] = []
				while (current < tokens.length && tokens[current].type !== '}') {
					methods.push(funDeclaration('method'))
				}
				consume('}', '`}` expected after class body')
				return { type: 'class', name, methods }
			}
			return statement()
		} catch (e) {
			if (e !== parseError) throw e
			synchronize()
		}
	}
	try {
		const statements: Statement[] = []
		while (tokens[current]?.type) {
			const stmt = declaration()
			if (stmt) statements.push(stmt)
		}
		return statements
	} catch (e) {
		if (e !== parseError) throw e
		return []
	}
}

function resolve(statements: Statement[]) {
	const scopes: Record<string, boolean>[] = []
	let currentFunction: undefined | 'function'
	const beginScope = () => void scopes.push(Object.create(null))
	const endScope = () => void scopes.pop()
	const declare = (name: Token) => {
		if (!scopes.length) return
		if (name.lexeme in scopes[scopes.length - 1]) {
			error(name.line, 'variable already declared')
		}
		scopes[scopes.length - 1][name.lexeme] = false
	}
	const define = (name: Token) => {
		if (!scopes.length) return
		scopes[scopes.length - 1][name.lexeme] = true
	}
	const resolveLocal = (expr: Expression, name: Token) => {
		for (let i = scopes.length - 1; i >= 0; i--) {
			if (name.lexeme in scopes[i]) {
				locals.set(expr, scopes.length - 1 - i)
				return
			}
		}
	}
	const resolveFunction = (func: Statement & { type: 'function' }, type: typeof currentFunction) => {
		const enclosingFunction = currentFunction
		currentFunction = type
		beginScope()
		for (const param of func.params) {
			declare(param)
			define(param)
		}
		func.body.forEach(resolve)
		endScope()
		currentFunction = enclosingFunction
	}
	function resolve(x: Statement | Expression) {
		switch (x.type) {
			case 'block':
				beginScope()
				x.statements.forEach(resolve)
				endScope()
				break
			case 'var':
				declare(x.name)
				x.initializer && resolve(x.initializer)
				define(x.name)
				break
			case 'variable':
				if (scopes[scopes.length - 1]?.[x.name.lexeme] === false) {
					error(x.name.line, 'local variable initializer accesses itself')
				}
				resolveLocal(x, x.name)
				break
			case 'assign':
				resolve(x.value)
				resolveLocal(x, x.name)
				break
			case 'function':
				declare(x.name)
				define(x.name)
				resolveFunction(x, 'function')
				break
			case 'class':
				declare(x.name)
				define(x.name)
				break
			case 'expression':
			case 'print':
			case 'grouping':
				resolve(x.expression)
				break
			case 'if':
				resolve(x.condition)
				resolve(x.thenBranch)
				x.elseBranch && resolve(x.elseBranch)
				break
			case 'while':
				resolve(x.condition)
				resolve(x.body)
				break
			case 'return':
				if (!currentFunction) {
					error(x.keyword.line, 'nowhere to return')
				}
				x.value && resolve(x.value)
				break
			case 'literal':
				break
			case 'unary':
				resolve(x.right)
				break
			case 'binary':
				resolve(x.left)
				resolve(x.right)
				break
			case 'call':
				resolve(x.callee)
				x.arguments.forEach(resolve)
				break
			case 'get':
				resolve(x.object)
				break
			case 'set':
				resolve(x.object)
				resolve(x.value)
				break
			default:
				x satisfies never
		}
	}
	statements.forEach(resolve)
}

type LoxObject = undefined | number | string | boolean | LoxCallable | LoxInstance

interface LoxCallable {
	arity(): number
	call(args: LoxObject[]): LoxObject
	toString(): string
}

class LoxFunction implements LoxCallable {
	constructor(
		public declaration: Statement & { type: 'function' },
		public closure: Environment,
	) {
	}
	arity() {
		return this.declaration.params.length
	}
	call(args: LoxObject[]): LoxObject {
		const env: Environment = Object.create(this.closure)
		for (let i = 0; i < this.declaration.params.length; i++) {
			env[this.declaration.params[i].lexeme] = args[i]
		}
		try {
			executeBlock(this.declaration.body, env)
		} catch (e) {
			if (!(e instanceof Return)) throw e
			return e.value
		}
	}
	toString() {
		return `<fn ${this.declaration.name.lexeme}>`
	}
}

class LoxClass implements LoxCallable {
	constructor(
		public name: string,
	) {
	}
	arity() {
		return 0
	}
	call(args: LoxObject[]): LoxObject {
		return new LoxInstance(this)
	}
	toString() {
		return this.name
	}
}

class LoxInstance {
	fields: Record<string, LoxObject> = Object.create(null)
	constructor(readonly klass: LoxClass) {
	}
	get(name: Token) {
		if (name.lexeme in this.fields) {
			return this.fields[name.lexeme]
		}
		throw new RuntimeError(name, 'undefined property')
	}
	set(name: Token, value: LoxObject) {
		this.fields[name.lexeme] = value
	}
	toString() {
		return `${this.klass.name} instance`
	}
}

type Environment = Record<string, LoxObject>
let environment: Environment = {
	// @ts-ignore
	__proto__: null,
	clock: {
		arity: () => 0,
		call: () => +new Date / 1000,
		toString: () => '<native function>',
	},
}
const locals = new WeakMap<Expression, number>

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
			switch (expr.operator.type) {
				case 'or':
					return isTruthy(left) ? left : evaluate(expr.right)
				case 'and':
					return isTruthy(left) ? evaluate(expr.right) : left
			}
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
						return left < right
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
		case 'variable': {
			let env = environment
			for (let i = locals.get(expr) ?? Infinity; i && Object.getPrototypeOf(env); i--) {
				env = Object.getPrototypeOf(env)
			}
			if (!Object.hasOwn(env, expr.name.lexeme)) {
				// possible for global variables only
				throw new RuntimeError(expr.name, 'undefined variable')
			}
			return env[expr.name.lexeme]
		}
		case 'assign': {
			let env = environment
			for (let i = locals.get(expr) ?? Infinity; i && Object.getPrototypeOf(env); i--) {
				env = Object.getPrototypeOf(env)
			}
			if (!Object.hasOwn(env, expr.name.lexeme)) {
				// possible for global variables only
				throw new RuntimeError(expr.name, 'undefined variable')
			}
			return env[expr.name.lexeme] = evaluate(expr.value)
		}
		case 'call':
			const callee = evaluate(expr.callee)
			if (!(typeof callee === 'object' && 'call' in callee)) {
				throw new RuntimeError(expr.paren, 'bad callee type')
			}
			const args = expr.arguments.map(evaluate)
			if (args.length !== callee.arity()) {
				throw new RuntimeError(expr.paren, `${callee.arity()}`)
			}
			return callee.call(args)
		case 'get': {
			const object = evaluate(expr.object)
			if (!(object instanceof LoxInstance)) {
				throw new RuntimeError(expr.name, 'only instances have properties')
			}
			return object.get(expr.name)
		}
		case 'set': {
			const object = evaluate(expr.object)
			if (!(object instanceof LoxInstance)) {
				throw new RuntimeError(expr.name, 'only instances have properties')
			}
			const value = evaluate(expr.value)
			object.set(expr.name, value)
			return value
		}
		default:
			expr satisfies never
	}
}

function executeBlock(statements: Statement[], env: Environment) {
	const previous = environment
	environment = env
	try {
		for (const s of statements) {
			execute(s)
		}
	} finally {
		environment = previous
	}
}

class Return {
	constructor(public value: LoxObject) {
	}
}

export function execute(stmt: Statement) {
	switch (stmt.type) {
		case 'block':
			executeBlock(stmt.statements, Object.create(environment))
			break
		case 'expression':
			evaluate(stmt.expression)
			break
		case 'print':
			console.log(stringify(evaluate(stmt.expression)))
			break
		case 'var':
			environment[stmt.name.lexeme] = stmt.initializer && evaluate(stmt.initializer)
			break
		case 'function':
			environment[stmt.name.lexeme] = new LoxFunction(stmt, environment)
			break
		case 'class':
			environment[stmt.name.lexeme] = new LoxClass(stmt.name.lexeme)
			break
		case 'if':
			if (isTruthy(evaluate(stmt.condition))) {
				execute(stmt.thenBranch)
			} else if (stmt.elseBranch) {
				execute(stmt.elseBranch)
			}
			break
		case 'while':
			while (isTruthy(evaluate(stmt.condition))) {
				execute(stmt.body)
			}
			break
		case 'return':
			throw new Return(stmt.value && evaluate(stmt.value))
		default:
			stmt satisfies never
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
	const statements = parse(tokenize(source))
	if (hadError) return
	resolve(statements)
	if (hadError) return
	try {
		for (const statement of statements) {
			execute(statement)
		}
	} catch (e) {
		if (!(e instanceof RuntimeError)) throw e
		console.error(`Runtime error: ${e.message} at \`${e.token.lexeme}\` (line ${e.token.line})`)
		hadRuntimeError = true
	}
}
