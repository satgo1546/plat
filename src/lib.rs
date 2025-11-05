use std::fmt::{Debug, Write};

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

    impl Scanner<'_> {
        pub fn new(source: &str) -> Scanner<'_> {
            Scanner {
                source,
                start: 0,
                current: 0,
                line: 1,
            }
        }

        fn make_token(&self, token_type: TokenType) -> Token<'_> {
            Token {
                token_type,
                lexeme: &self.source[self.start..self.current],
                line: self.line,
            }
        }

        fn error_token<'a>(&self, message: &'a str) -> Token<'a> {
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

        pub fn next(&mut self) -> Token<'_> {
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
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
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

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1) as u8
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

pub struct VM {}

#[derive(Debug)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
}
pub type InterpretResult = Result<(), InterpretError>;

impl VM {
    pub fn new() -> VM {
        VM {}
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        self.compile(source);
        Ok(())
    }

    fn compile(&self, source: &str) {
        let mut scanner = scanner::Scanner::new(source);
        let mut line = -1;
        loop {
            let token = scanner.next();
            if token.line == line {
                print!("   | ");
            } else {
                line = token.line;
                print!("{:4} ", line);
            }
            println!("{:?} '{}'", token.token_type, token.lexeme);
            if let crate::scanner::TokenType::EOF = token.token_type {
                break;
            }
        }
    }

    fn binary_op(stack: &mut Vec<Value>, op: fn(f64, f64) -> f64) -> InterpretResult {
        let b = stack.pop().unwrap();
        let a = stack.pop().unwrap();
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                stack.push(Value::Number(op(a, b)));
                Ok(())
            }
            #[allow(unreachable_patterns)]
            _ => Err(InterpretError::RuntimeError),
        }
    }

    pub fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        let mut ip = 0;
        let mut stack = Vec::<Value>::new();
        loop {
            match chunk.code[ip] {
                Instruction::Constant(constant) => {
                    let constant = chunk.constants[constant as usize].clone();
                    println!("{:?}", constant);
                    stack.push(constant);
                }
                Instruction::Add => {
                    Self::binary_op(&mut stack, std::ops::Add::add)?;
                }
                Instruction::Subtract => {
                    Self::binary_op(&mut stack, std::ops::Sub::sub)?;
                }
                Instruction::Multiply => {
                    Self::binary_op(&mut stack, std::ops::Mul::mul)?;
                }
                Instruction::Divide => {
                    Self::binary_op(&mut stack, std::ops::Div::div)?;
                }
                Instruction::Negate => {
                    let value = stack.last_mut().unwrap();
                    match value {
                        Value::Number(x) => *x = -*x,
                    }
                }
                Instruction::Return => {
                    let value = stack.pop().unwrap();
                    println!("ret {:?}", value);
                    return Ok(());
                }
            }
            ip += 1;
        }
    }
}
