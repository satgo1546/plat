use std::fmt::{Debug, Display, Write};

mod scanner {
    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_alpha(c: char) -> bool {
        c >= 'A' && c <= 'Z' || c >= 'a' && c <= 'z' || c == '_'
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TokenType {
        // Single-character tokens.
        LeftParen,
        RightParen,
        LeftBrace,
        RightBrace,
        Comma,
        Dot,
        Minus,
        Plus,
        Semicolon,
        Slash,
        Star,
        // One or two character tokens.
        Bang,
        BangEqual,
        Equal,
        EqualEqual,
        Greater,
        GreaterEqual,
        Less,
        LessEqual,
        // Literals.
        Identifier,
        String,
        Number,
        // Keywords.
        And,
        Class,
        Else,
        False,
        For,
        Fun,
        If,
        Nil,
        Or,
        Print,
        Return,
        Super,
        This,
        True,
        Var,
        While,

        Error,
        EOF,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Token<'a> {
        pub token_type: TokenType,
        pub lexeme: &'a str,
        pub line: i32,
    }

    pub struct Scanner<'a> {
        source: &'a str,
        start: usize,
        current: usize,
        line: i32,
    }

    impl<'a> Scanner<'a> {
        pub fn new(source: &str) -> Scanner<'_> {
            Scanner {
                source,
                start: 0,
                current: 0,
                line: 1,
            }
        }

        fn make_token(&self, token_type: TokenType) -> Token<'a> {
            Token {
                token_type,
                lexeme: &self.source[self.start..self.current],
                line: self.line,
            }
        }

        fn error_token<'b>(&self, message: &'b str) -> Token<'b> {
            Token {
                token_type: TokenType::Error,
                lexeme: message,
                line: self.line,
            }
        }

        fn is_at_end(&self) -> bool {
            self.current >= self.source.len()
        }

        fn char_at(&self, index: usize) -> char {
            self.source.as_bytes()[index] as char
        }

        fn peek(&self) -> char {
            if self.is_at_end() {
                return '\0';
            }
            self.char_at(self.current)
        }

        fn peek_next(&self) -> char {
            if self.current + 1 >= self.source.len() {
                return '\0';
            }
            self.char_at(self.current + 1)
        }

        fn advance(&mut self) -> char {
            let c = self.peek();
            self.current += 1;
            c
        }

        fn matches(&mut self, expected: char) -> bool {
            if self.is_at_end() {
                return false;
            }
            if self.peek() != expected {
                return false;
            }
            self.advance();
            true
        }

        fn skip_whitespace(&mut self) {
            loop {
                match self.peek() {
                    ' ' | '\r' | '\t' => {
                        self.advance();
                    }
                    '\n' => {
                        self.line += 1;
                        self.advance();
                    }
                    '/' => {
                        if self.peek_next() == '/' {
                            while self.peek() != '\n' && !self.is_at_end() {
                                self.advance();
                            }
                        } else {
                            return;
                        }
                    }
                    _ => return,
                };
            }
        }

        fn check_keyword(&self, start: usize, rest: &str, token_type: TokenType) -> TokenType {
            if self.current - self.start == start + rest.len()
                && rest == &self.source[self.start + start..self.current]
            {
                token_type
            } else {
                TokenType::Identifier
            }
        }

        fn identifier_type(&self) -> TokenType {
            match self.char_at(self.start) {
                'a' => self.check_keyword(1, "nd", TokenType::And),
                'c' => self.check_keyword(1, "lass", TokenType::Class),
                'e' => self.check_keyword(1, "lse", TokenType::Else),
                'f' => {
                    if self.current - self.start > 1 {
                        match self.char_at(self.start + 1) {
                            'a' => self.check_keyword(2, "lse", TokenType::False),
                            'o' => self.check_keyword(2, "r", TokenType::For),
                            'u' => self.check_keyword(2, "n", TokenType::Fun),
                            _ => TokenType::Identifier,
                        }
                    } else {
                        TokenType::Identifier
                    }
                }
                'i' => self.check_keyword(1, "f", TokenType::If),
                'n' => self.check_keyword(1, "il", TokenType::Nil),
                'o' => self.check_keyword(1, "r", TokenType::Or),
                'p' => self.check_keyword(1, "rint", TokenType::Print),
                'r' => self.check_keyword(1, "eturn", TokenType::Return),
                's' => self.check_keyword(1, "uper", TokenType::Super),
                't' => {
                    if self.current - self.start > 1 {
                        match self.char_at(self.start + 1) {
                            'h' => self.check_keyword(2, "is", TokenType::This),
                            'r' => self.check_keyword(2, "ue", TokenType::True),
                            _ => TokenType::Identifier,
                        }
                    } else {
                        TokenType::Identifier
                    }
                }
                'v' => self.check_keyword(1, "ar", TokenType::Var),
                'w' => self.check_keyword(1, "hile", TokenType::While),
                _ => TokenType::Identifier,
            }
        }

        pub fn next(&mut self) -> Token<'a> {
            self.skip_whitespace();
            self.start = self.current;
            if self.is_at_end() {
                return self.make_token(TokenType::EOF);
            }
            let c = self.advance();
            match c {
                '(' => self.make_token(TokenType::LeftParen),
                ')' => self.make_token(TokenType::RightParen),
                '{' => self.make_token(TokenType::LeftBrace),
                '}' => self.make_token(TokenType::RightBrace),
                ';' => self.make_token(TokenType::Semicolon),
                ',' => self.make_token(TokenType::Comma),
                '.' => self.make_token(TokenType::Dot),
                '-' => self.make_token(TokenType::Minus),
                '+' => self.make_token(TokenType::Plus),
                '/' => self.make_token(TokenType::Slash),
                '*' => self.make_token(TokenType::Star),
                '!' => {
                    if self.matches('=') {
                        self.make_token(TokenType::BangEqual)
                    } else {
                        self.make_token(TokenType::Bang)
                    }
                }
                '=' => {
                    if self.matches('=') {
                        self.make_token(TokenType::EqualEqual)
                    } else {
                        self.make_token(TokenType::Equal)
                    }
                }
                '<' => {
                    if self.matches('=') {
                        self.make_token(TokenType::LessEqual)
                    } else {
                        self.make_token(TokenType::Less)
                    }
                }
                '>' => {
                    if self.matches('=') {
                        self.make_token(TokenType::GreaterEqual)
                    } else {
                        self.make_token(TokenType::Greater)
                    }
                }
                '"' => {
                    while self.peek() != '"' && !self.is_at_end() {
                        if self.peek() == '\n' {
                            self.line += 1;
                        }
                        self.advance();
                    }
                    if self.is_at_end() {
                        self.error_token("unterminated string")
                    } else {
                        assert!(self.advance() == '"');
                        self.make_token(TokenType::String)
                    }
                }
                c if is_alpha(c) => {
                    while is_alpha(self.peek()) || is_digit(self.peek()) {
                        self.advance();
                    }
                    self.make_token(self.identifier_type())
                }
                c if is_digit(c) => {
                    while is_digit(self.peek()) {
                        self.advance();
                    }
                    if self.peek() == '.' && is_digit(self.peek_next()) {
                        assert!(self.advance() == '.');
                        while is_digit(self.peek()) {
                            self.advance();
                        }
                    }
                    self.make_token(TokenType::Number)
                }
                _ => self.error_token("unexpected character"),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Constant(u8),
    Nil,
    True,
    False,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    Less,
    Greater,
    Not,
    Negate,
    Pop,
    Print,
    Return,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl Value {
    fn is_truthy(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Boolean(x) => *x,
            _ => true,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => f.write_str("nil"),
            Self::Boolean(x) => write!(f, "{}", x),
            Self::Number(x) => write!(f, "{}", x),
            Self::String(x) => f.write_str(x),
        }
    }
}

pub struct Chunk {
    code: Vec<Instruction>,
    lines: Vec<i32>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn write(&mut self, instruction: Instruction, line: i32) {
        self.code.push(instruction);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> Option<u8> {
        if self.constants.len() <= u8::MAX as usize {
            self.constants.push(value);
            Some((self.constants.len() - 1) as u8)
        } else {
            None
        }
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, instruction) in self.code.iter().enumerate() {
            if i > 0 {
                f.write_char('\n')?;
            }
            write!(f, "{:04} ", i)?;
            if i > 0 && self.lines[i - 1] == self.lines[i] {
                f.write_str("   | ")?;
            } else {
                write!(f, "{:4} ", self.lines[i])?;
            }
            write!(f, "{:?}", instruction)?;
        }
        Ok(())
    }
}

mod compiler {
    use crate::{
        Chunk, Instruction, Value,
        scanner::{Scanner, Token, TokenType},
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
    struct Precedence(u8);

    impl Precedence {
        pub const NONE: Self = Self(0);
        pub const ASSIGNMENT: Self = Self(1); // =
        #[allow(unused)]
        pub const OR: Self = Self(2); // or
        #[allow(unused)]
        pub const AND: Self = Self(3); // and
        pub const EQUALITY: Self = Self(4); // == !=
        pub const COMPARISON: Self = Self(5); // < <= > >=
        pub const TERM: Self = Self(6); // + -
        pub const FACTOR: Self = Self(7); // * /
        pub const UNARY: Self = Self(8); // ! -
        #[allow(unused)]
        pub const CALL: Self = Self(9); // . ()
        #[allow(unused)]
        pub const PRIMARY: Self = Self(10);
    }

    pub struct Compiler<'a> {
        scanner: Scanner<'a>,
        current: Token<'a>,
        previous: Token<'a>,
        had_error: bool,
        panic_mode: bool,
    }

    impl<'a> Compiler<'a> {
        pub fn new<'b>(source: &'b str) -> Compiler<'b> {
            Compiler {
                scanner: Scanner::new(source),
                current: Token {
                    token_type: TokenType::EOF,
                    lexeme: "",
                    line: 0,
                },
                previous: Token {
                    token_type: TokenType::EOF,
                    lexeme: "",
                    line: 0,
                },
                had_error: false,
                panic_mode: false,
            }
        }

        fn error_at(&mut self, token: Token, message: &str) {
            if self.panic_mode {
                return;
            }
            self.panic_mode = true;
            eprint!("Error: {}", message);
            match token.token_type {
                TokenType::EOF => eprint!(" at end"),
                TokenType::Error => {}
                _ => eprint!(" at `{}`", token.lexeme),
            }
            eprintln!(" (line {})", token.line);
            self.had_error = true;
        }

        fn error_at_current(&mut self, message: &str) {
            self.error_at(self.current, message);
        }

        fn error(&mut self, message: &str) {
            self.error_at(self.previous, message);
        }

        fn advance(&mut self) {
            self.previous = self.current;
            loop {
                self.current = self.scanner.next();
                if let TokenType::Error = self.current.token_type {
                    self.error_at_current(self.current.lexeme);
                } else {
                    break;
                }
            }
        }

        fn check(&self, token_type: TokenType) -> bool {
            self.current.token_type == token_type
        }

        fn matches(&mut self, token_type: TokenType) -> bool {
            if self.check(token_type) {
                self.advance();
                return true;
            }
            false
        }

        fn consume(&mut self, token_type: TokenType, message: &str) {
            if !self.matches(token_type) {
                self.error_at_current(message);
            }
        }

        fn synchronize(&mut self) {
            self.panic_mode = false;
            while self.current.token_type != TokenType::EOF {
                if let TokenType::Semicolon = self.previous.token_type {
                    return;
                }
                match self.current.token_type {
                    TokenType::Var
                    | TokenType::Fun
                    | TokenType::Class
                    | TokenType::If
                    | TokenType::For
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return => return,
                    _ => {}
                }
                self.advance();
            }
        }

        fn get_precedence(token_type: TokenType) -> Precedence {
            match token_type {
                TokenType::Plus | TokenType::Minus => Precedence::TERM,
                TokenType::Star | TokenType::Slash => Precedence::FACTOR,
                TokenType::EqualEqual | TokenType::BangEqual => Precedence::EQUALITY,
                TokenType::Less
                | TokenType::LessEqual
                | TokenType::Greater
                | TokenType::GreaterEqual => Precedence::COMPARISON,
                _ => Precedence::NONE,
            }
        }

        fn parse_precedence(&mut self, chunk: &mut Chunk, base_precedence: Precedence) {
            self.advance();
            match self.previous.token_type {
                TokenType::LeftParen => {
                    self.expression(chunk);
                    self.consume(TokenType::RightParen, "`)` expected after expression");
                }
                TokenType::Bang => {
                    self.parse_precedence(chunk, Precedence::UNARY);
                    self.emit_instruction(chunk, Instruction::Not);
                }
                TokenType::Minus => {
                    self.parse_precedence(chunk, Precedence::UNARY);
                    self.emit_instruction(chunk, Instruction::Negate);
                }
                TokenType::Nil => {
                    self.emit_instruction(chunk, Instruction::Nil);
                }
                TokenType::True => {
                    self.emit_instruction(chunk, Instruction::True);
                }
                TokenType::False => {
                    self.emit_instruction(chunk, Instruction::False);
                }
                TokenType::Number => {
                    let value = self.previous.lexeme.parse();
                    let value = value.expect("tokenizer slip through?");
                    self.emit_constant(chunk, Value::Number(value));
                }
                TokenType::String => {
                    let value = &self.previous.lexeme[1..self.previous.lexeme.len() - 1];
                    self.emit_constant(chunk, Value::String(value.to_string()));
                }
                _ => {
                    self.error("expression expected");
                }
            }
            loop {
                let precedence = Self::get_precedence(self.current.token_type);
                if base_precedence > precedence {
                    break;
                }
                self.advance();
                match self.previous.token_type {
                    token_type @ (TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Star
                    | TokenType::Slash
                    | TokenType::EqualEqual
                    | TokenType::BangEqual
                    | TokenType::Less
                    | TokenType::LessEqual
                    | TokenType::Greater
                    | TokenType::GreaterEqual) => {
                        self.parse_precedence(chunk, Precedence(precedence.0 + 1));
                        match token_type {
                            TokenType::Plus => self.emit_instruction(chunk, Instruction::Add),
                            TokenType::Minus => self.emit_instruction(chunk, Instruction::Subtract),
                            TokenType::Star => self.emit_instruction(chunk, Instruction::Multiply),
                            TokenType::Slash => self.emit_instruction(chunk, Instruction::Divide),
                            TokenType::EqualEqual => {
                                self.emit_instruction(chunk, Instruction::Equal)
                            }
                            TokenType::BangEqual => {
                                self.emit_instruction(chunk, Instruction::Equal);
                                self.emit_instruction(chunk, Instruction::Not);
                            }
                            TokenType::Less => self.emit_instruction(chunk, Instruction::Less),
                            TokenType::LessEqual => {
                                self.emit_instruction(chunk, Instruction::Greater);
                                self.emit_instruction(chunk, Instruction::Not);
                            }
                            TokenType::Greater => {
                                self.emit_instruction(chunk, Instruction::Greater)
                            }
                            TokenType::GreaterEqual => {
                                self.emit_instruction(chunk, Instruction::Less);
                                self.emit_instruction(chunk, Instruction::Not);
                            }
                            _ => unreachable!("match statements mismatch"),
                        }
                    }
                    _ => unreachable!("unhandled TokenType having precedence other than None"),
                }
            }
        }

        fn expression(&mut self, chunk: &mut Chunk) {
            self.parse_precedence(chunk, Precedence::ASSIGNMENT);
        }

        fn statement(&mut self, chunk: &mut Chunk) {
            if self.matches(TokenType::Print) {
                self.expression(chunk);
                self.consume(TokenType::Semicolon, "`;` expected after value");
                self.emit_instruction(chunk, Instruction::Print);
            } else {
                self.expression(chunk);
                self.consume(TokenType::Semicolon, "`;` expected after expression");
                self.emit_instruction(chunk, Instruction::Pop);
            }
        }

        fn declaration(&mut self, chunk: &mut Chunk) {
            self.statement(chunk);
            if self.panic_mode {
                self.synchronize();
            }
        }

        pub fn compile(&mut self, chunk: &mut Chunk) -> Result<(), ()> {
            self.advance();
            while !self.matches(TokenType::EOF) {
                self.declaration(chunk);
            }
            self.consume(TokenType::EOF, "end of expression expected");
            if self.had_error {
                return Err(());
            }
            self.emit_instruction(chunk, Instruction::Return);
            if !self.had_error {
                println!("{:?}", chunk);
            }
            Ok(())
        }

        fn emit_instruction(&self, chunk: &mut Chunk, instruction: Instruction) {
            chunk.write(instruction, self.previous.line);
        }

        fn emit_constant(&mut self, chunk: &mut Chunk, value: Value) {
            if let Some(constant) = chunk.add_constant(value) {
                self.emit_instruction(chunk, Instruction::Constant(constant));
            } else {
                self.error("too many constants in one chunk");
            }
        }
    }
}

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
}

#[derive(Debug)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
}
pub type InterpretResult = Result<(), InterpretError>;

impl VM {
    pub fn new() -> VM {
        VM {
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut chunk = Chunk::new();
        let mut compiler = compiler::Compiler::new(source);
        if let Err(_) = compiler.compile(&mut chunk) {
            return Err(InterpretError::CompileError);
        }
        self.run(&chunk)
    }

    fn runtime_error(&mut self, chunk: &Chunk, message: &str) -> InterpretResult {
        eprintln!("Runtime error: {} (line {})", message, chunk.lines[self.ip]);
        self.stack.clear();
        Err(InterpretError::RuntimeError)
    }

    fn binary_op(&mut self, chunk: &Chunk, op: fn(f64, f64) -> Value) -> InterpretResult {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                self.stack.push(op(a, b));
                Ok(())
            }
            _ => self.runtime_error(chunk, "operands must be numbers"),
        }
    }

    fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        self.ip = 0;
        self.stack.clear();
        loop {
            match chunk.code[self.ip] {
                Instruction::Constant(constant) => {
                    let constant = chunk.constants[constant as usize].clone();
                    println!("{:?}", constant);
                    self.stack.push(constant);
                }
                Instruction::Nil => {
                    self.stack.push(Value::Nil);
                }
                Instruction::True => {
                    self.stack.push(Value::Boolean(true));
                }
                Instruction::False => {
                    self.stack.push(Value::Boolean(false));
                }
                Instruction::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(match (a, b) {
                        (a @ Value::String(_), b) | (a, b @ Value::String(_)) => {
                            Value::String(format!("{}{}", a, b))
                        }
                        (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                        _ => {
                            return self.runtime_error(
                                chunk,
                                "operands must consist of two numbers or at least one string",
                            );
                        }
                    })
                }
                Instruction::Subtract => {
                    self.binary_op(chunk, |a, b| Value::Number(a - b))?;
                }
                Instruction::Multiply => {
                    self.binary_op(chunk, |a, b| Value::Number(a * b))?;
                }
                Instruction::Divide => {
                    self.binary_op(chunk, |a, b| Value::Number(a / b))?;
                }
                Instruction::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(a == b));
                }
                Instruction::Less => {
                    self.binary_op(chunk, |a, b| Value::Boolean(a < b))?;
                }
                Instruction::Greater => {
                    self.binary_op(chunk, |a, b| Value::Boolean(a > b))?;
                }
                Instruction::Not => {
                    let value = self.stack.last_mut().unwrap();
                    *value = Value::Boolean(!value.is_truthy());
                }
                Instruction::Negate => {
                    let value = self.stack.last_mut().unwrap();
                    match value {
                        Value::Number(x) => *x = -*x,
                        _ => return self.runtime_error(chunk, "operand must be number"),
                    }
                }
                Instruction::Pop => {
                    self.stack.pop().unwrap();
                }
                Instruction::Print => {
                    let value = self.stack.pop().unwrap();
                    println!("{}", value);
                }
                Instruction::Return => {
                    return Ok(());
                }
            }
            self.ip += 1;
        }
    }
}
