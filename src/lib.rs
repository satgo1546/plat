use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display, Write},
    iter::zip,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

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

#[derive(Debug, Default, Clone, Copy)]
pub enum Instruction {
    #[default]
    NOP,
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
    DefineGlobal(u8),
    GetGlobal(u8),
    SetGlobal(u8),
    GetLocal(u8),
    SetLocal(u8),
    GetUpvalue(u8),
    SetUpvalue(u8),
    Jump(i16),
    JumpIfFalse(i16),
    Call(u8),
    Closure(u8),      // may be followed by LocalUpvalue and NonlocalUpvalue instructions
    LocalUpvalue(u8), // never in isolation
    NonlocalUpvalue(u8), // never in isolation
    CloseUpvalue,
    Class(u8),
    GetProperty(u8),
    SetProperty(u8),
    Method(u8),
    Inherit,
    GetSuper(u8),
}

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Closure(Rc<ObjFunction>, Vec<Rc<RefCell<ObjUpvalue>>>),
    Native(fn(Vec<Value>) -> Value),
    Class(Rc<RefCell<ObjClass>>),
    Instance(Rc<RefCell<ObjInstance>>),
    BoundMethod(Rc<ObjBoundMethod>),
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Closure(a_function, a_upvalues), Self::Closure(b_function, b_upvalues)) => {
                Rc::ptr_eq(a_function, b_function)
                    && zip(a_upvalues, b_upvalues).all(|(a, b)| Rc::ptr_eq(a, b))
            }
            (Self::Native(a), Self::Native(b)) => std::ptr::fn_addr_eq(*a, *b),
            (Self::Class(a), Self::Class(b)) => Rc::ptr_eq(a, b),
            (Self::Instance(a), Self::Instance(b)) => Rc::ptr_eq(a, b),
            (Self::BoundMethod(a), Self::BoundMethod(b)) => {
                Rc::ptr_eq(&a.receiver, &b.receiver)
                    && Rc::ptr_eq(&a.class, &b.class)
                    && a.method_name == b.method_name
            }
            _ => false,
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
            Self::Closure(x, _) => write!(f, "<fn {}>", x.name),
            Self::Native(_) => f.write_str("<native fn>"),
            Self::Class(x) => f.write_str(&x.borrow().name),
            Self::Instance(x) => write!(f, "{} instance", x.borrow().class.borrow().name),
            Self::BoundMethod(x) => write!(f, "<bound fn {}>", x.method_name),
        }
    }
}

#[derive(Default, Clone)]
pub struct ObjFunction {
    arity: i32,
    upvalue_count: i32,
    chunk: Chunk,
    name: String,
}

impl Debug for ObjFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "== {}/{} ==\n{:?}", self.name, self.arity, self.chunk)
    }
}

#[derive(Debug, Clone)]
pub enum ObjUpvalue {
    Open(usize, usize), // frame index, stack index
    Closed(Value),
}

#[derive(Debug, Clone)]
pub struct ObjClass {
    name: String,
    methods: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct ObjInstance {
    class: Rc<RefCell<ObjClass>>,
    fields: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct ObjBoundMethod {
    receiver: Rc<RefCell<ObjInstance>>,
    class: Rc<RefCell<ObjClass>>,
    method_name: String,
}

#[derive(Default, Clone)]
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
            match instruction {
                Instruction::Constant(constant)
                | Instruction::DefineGlobal(constant)
                | Instruction::GetGlobal(constant)
                | Instruction::SetGlobal(constant)
                | Instruction::Closure(constant)
                | Instruction::Class(constant)
                | Instruction::GetProperty(constant)
                | Instruction::SetProperty(constant)
                | Instruction::Method(constant)
                | Instruction::GetSuper(constant) => {
                    write!(f, " = {:?}", self.constants[*constant as usize])?
                }
                _ => {}
            }
        }
        Ok(())
    }
}

mod compiler {
    use std::rc::Rc;

    use crate::{
        Chunk, Instruction, ObjFunction, Value,
        scanner::{Scanner, Token, TokenType},
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
    struct Precedence(u8);

    impl Precedence {
        pub const NONE: Self = Self(0);
        pub const ASSIGNMENT: Self = Self(1); // =
        pub const OR: Self = Self(2); // or
        pub const AND: Self = Self(3); // and
        pub const EQUALITY: Self = Self(4); // == !=
        pub const COMPARISON: Self = Self(5); // < <= > >=
        pub const TERM: Self = Self(6); // + -
        pub const FACTOR: Self = Self(7); // * /
        pub const UNARY: Self = Self(8); // ! -
        pub const CALL: Self = Self(9); // . ()
        #[allow(unused)]
        pub const PRIMARY: Self = Self(10);
    }

    struct Local<'a> {
        name: Token<'a>,
        depth: i32,
        is_captured: bool,
    }

    #[derive(Debug, Clone, Copy)]
    enum FunctionType {
        Function,
        Method,
        Initializer,
        Script,
    }

    #[derive(Debug, Clone, Copy)]
    struct Upvalue {
        index: u8,
        is_local: bool,
    }

    pub struct Compiler<'a> {
        scanner: Scanner<'a>,
        current: Token<'a>,
        previous: Token<'a>,
        had_error: bool,
        panic_mode: bool,
        frames: Vec<CompilerFrame<'a>>,
        enclosing_classes: Vec<bool>, // struct ClassCompiler.hasSuperclass
    }

    pub struct CompilerFrame<'a> {
        function: ObjFunction,
        function_type: FunctionType,
        locals: Vec<Local<'a>>,
        upvalues: Vec<Upvalue>,
        scope_depth: i32,
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
                frames: vec![CompilerFrame {
                    function: ObjFunction {
                        arity: 0,
                        upvalue_count: 0,
                        chunk: Chunk::new(),
                        name: String::new(),
                    },
                    function_type: FunctionType::Script,
                    locals: Vec::new(),
                    upvalues: Vec::new(),
                    scope_depth: 0,
                }],
                enclosing_classes: Vec::new(),
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

        fn parse_variable(&mut self, error_message: &str) -> u8 {
            self.consume(TokenType::Identifier, error_message);
            self.declare_variable();
            let frame = self.frames.last().unwrap();
            if frame.scope_depth > 0 {
                0
            } else {
                self.make_constant(Value::String(self.previous.lexeme.to_string()))
            }
        }

        fn declare_variable(&mut self) {
            let frame = self.frames.last().unwrap();
            if frame.scope_depth > 0 {
                let name = self.previous;
                let mut error = false;
                for local in frame.locals.iter().rev() {
                    if local.depth != -1 && local.depth < frame.scope_depth {
                        break;
                    }
                    if local.name.lexeme == name.lexeme {
                        error = true;
                        break;
                    }
                }
                if error {
                    self.error("variable already defined");
                } else {
                    self.add_local(name);
                }
            }
        }

        fn add_local(&mut self, name: Token<'a>) {
            let frame = self.frames.last_mut().unwrap();
            if frame.locals.len() > u8::MAX as usize {
                self.error("too many local variables");
                return;
            }
            frame.locals.push(Local {
                name,
                depth: -1,
                is_captured: false,
            });
        }

        fn define_variable(&mut self, global: u8) {
            if self.frames.last().unwrap().scope_depth > 0 {
                self.make_initialized();
                return;
            }
            self.emit_instruction(Instruction::DefineGlobal(global));
        }

        fn make_initialized(&mut self) {
            let frame = self.frames.last_mut().unwrap();
            if frame.scope_depth == 0 {
                return;
            }
            frame.locals.last_mut().unwrap().depth = frame.scope_depth;
        }

        fn resolve_local(&mut self, frame_index: usize, name: Token) -> Option<u8> {
            let frame = &mut self.frames[frame_index];
            if let Some((i, local)) = frame
                .locals
                .iter()
                .enumerate()
                .rev()
                .find(|&(_, local)| local.name.lexeme == name.lexeme)
            {
                if local.depth == -1 {
                    self.error("self-initializer");
                }
                Some(i as u8)
            } else {
                None
            }
        }

        fn add_upvalue(&mut self, frame_index: usize, index: u8, is_local: bool) -> u8 {
            if self.frames[frame_index].upvalues.len() > u8::MAX as usize {
                self.error("too many upvalues");
            }
            self.frames[frame_index].function.upvalue_count += 1;
            self.frames[frame_index]
                .upvalues
                .push(Upvalue { index, is_local });
            (self.frames[frame_index].upvalues.len() - 1) as u8
        }

        fn resolve_upvalue(&mut self, frame_index: usize, name: Token) -> Option<u8> {
            if frame_index == 0 {
                return None;
            }
            if let Some(local) = self.resolve_local(frame_index - 1, name) {
                self.frames[frame_index - 1].locals[local as usize].is_captured = true;
                return Some(self.add_upvalue(frame_index, local, true));
            }
            if let Some(upvalue) = self.resolve_upvalue(frame_index - 1, name) {
                return Some(self.add_upvalue(frame_index, upvalue, false));
            }
            None
        }

        fn named_variable(&mut self, name: Token, can_assign: bool) {
            let (get_op, set_op, arg): (fn(u8) -> Instruction, fn(u8) -> Instruction, u8) =
                if let Some(i) = self.resolve_local(self.frames.len() - 1, name) {
                    (Instruction::GetLocal, Instruction::SetLocal, i)
                } else if let Some(arg) = self.resolve_upvalue(self.frames.len() - 1, name) {
                    (Instruction::GetUpvalue, Instruction::SetUpvalue, arg)
                } else {
                    let arg = self.make_constant(Value::String(name.lexeme.to_string()));
                    (Instruction::GetGlobal, Instruction::SetGlobal, arg)
                };
            if can_assign && self.matches(TokenType::Equal) {
                self.expression();
                self.emit_instruction(set_op(arg));
            } else {
                self.emit_instruction(get_op(arg));
            }
        }

        fn argument_list(&mut self) -> u8 {
            let mut count = 0u8;
            loop {
                if self.check(TokenType::RightParen) {
                    break;
                }
                self.expression();
                count = count.checked_add(1).unwrap_or_else(|| {
                    self.error("too many arguments");
                    0
                });
                if !self.matches(TokenType::Comma) {
                    break;
                }
            }
            self.consume(TokenType::RightParen, "`)` expected after arguments");
            count
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
                TokenType::And => Precedence::AND,
                TokenType::Or => Precedence::OR,
                TokenType::LeftParen | TokenType::Dot => Precedence::CALL,
                _ => Precedence::NONE,
            }
        }

        fn parse_precedence(&mut self, base_precedence: Precedence) {
            let can_assign = base_precedence <= Precedence::ASSIGNMENT;
            self.advance();
            match self.previous.token_type {
                TokenType::LeftParen => {
                    self.expression();
                    self.consume(TokenType::RightParen, "`)` expected after expression");
                }
                TokenType::Bang => {
                    self.parse_precedence(Precedence::UNARY);
                    self.emit_instruction(Instruction::Not);
                }
                TokenType::Minus => {
                    self.parse_precedence(Precedence::UNARY);
                    self.emit_instruction(Instruction::Negate);
                }
                TokenType::Nil => {
                    self.emit_instruction(Instruction::Nil);
                }
                TokenType::True => {
                    self.emit_instruction(Instruction::True);
                }
                TokenType::False => {
                    self.emit_instruction(Instruction::False);
                }
                TokenType::Number => {
                    let value = self.previous.lexeme.parse();
                    let value = value.expect("tokenizer slip through?");
                    self.emit_constant(Value::Number(value));
                }
                TokenType::String => {
                    let value = &self.previous.lexeme[1..self.previous.lexeme.len() - 1];
                    self.emit_constant(Value::String(value.to_string()));
                }
                TokenType::Identifier => {
                    self.named_variable(self.previous, can_assign);
                }
                TokenType::This => {
                    if self.enclosing_classes.is_empty() {
                        self.error("stray `this`");
                    }
                    self.named_variable(self.previous, can_assign);
                }
                TokenType::Super => {
                    if let Some(has_super_class) = self.enclosing_classes.last() {
                        if !*has_super_class {
                            self.error("`super` in a root class");
                        }
                    } else {
                        self.error("stray `super`");
                    }
                    self.consume(TokenType::Dot, "`.` expected after `super`");
                    self.consume(TokenType::Identifier, "method name expected after `.`");
                    let name = self.make_constant(Value::String(self.previous.lexeme.to_string()));
                    self.named_variable(
                        Token {
                            token_type: TokenType::Identifier,
                            lexeme: "this",
                            line: 0,
                        },
                        false,
                    );
                    self.named_variable(
                        Token {
                            token_type: TokenType::Identifier,
                            lexeme: "super",
                            line: 0,
                        },
                        false,
                    );
                    self.emit_instruction(Instruction::GetSuper(name));
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
                        self.parse_precedence(Precedence(precedence.0 + 1));
                        match token_type {
                            TokenType::Plus => self.emit_instruction(Instruction::Add),
                            TokenType::Minus => self.emit_instruction(Instruction::Subtract),
                            TokenType::Star => self.emit_instruction(Instruction::Multiply),
                            TokenType::Slash => self.emit_instruction(Instruction::Divide),
                            TokenType::EqualEqual => self.emit_instruction(Instruction::Equal),
                            TokenType::BangEqual => {
                                self.emit_instruction(Instruction::Equal);
                                self.emit_instruction(Instruction::Not);
                            }
                            TokenType::Less => self.emit_instruction(Instruction::Less),
                            TokenType::LessEqual => {
                                self.emit_instruction(Instruction::Greater);
                                self.emit_instruction(Instruction::Not);
                            }
                            TokenType::Greater => self.emit_instruction(Instruction::Greater),
                            TokenType::GreaterEqual => {
                                self.emit_instruction(Instruction::Less);
                                self.emit_instruction(Instruction::Not);
                            }
                            _ => unreachable!("match statements mismatch"),
                        }
                    }
                    TokenType::And => {
                        let end_jump = self.emit_jump();
                        self.emit_instruction(Instruction::Pop);
                        self.parse_precedence(Precedence::AND);
                        self.patch_jump(end_jump, Instruction::JumpIfFalse);
                    }
                    TokenType::Or => {
                        let else_jump = self.emit_jump();
                        let end_jump = self.emit_jump();
                        self.patch_jump(else_jump, Instruction::JumpIfFalse);
                        self.emit_instruction(Instruction::Pop);
                        self.parse_precedence(Precedence::OR);
                        self.patch_jump(end_jump, Instruction::Jump);
                    }
                    TokenType::LeftParen => {
                        let arg_count = self.argument_list();
                        self.emit_instruction(Instruction::Call(arg_count));
                    }
                    TokenType::Dot => {
                        self.consume(TokenType::Identifier, "property name expected");
                        let name =
                            self.make_constant(Value::String(self.previous.lexeme.to_string()));
                        if can_assign && self.matches(TokenType::Equal) {
                            self.expression();
                            self.emit_instruction(Instruction::SetProperty(name));
                        } else {
                            self.emit_instruction(Instruction::GetProperty(name));
                        }
                    }
                    _ => unreachable!("unhandled TokenType having precedence other than None"),
                }
            }
            if can_assign && self.matches(TokenType::Equal) {
                self.error("invalid assignment target");
            }
        }

        fn expression(&mut self) {
            self.parse_precedence(Precedence::ASSIGNMENT);
        }

        fn begin_scope(&mut self) {
            let frame = self.frames.last_mut().unwrap();
            frame.scope_depth += 1;
        }

        fn end_scope(&mut self) {
            let frame = self.frames.last_mut().unwrap();
            frame.scope_depth -= 1;
            while let Some(Local { is_captured, .. }) =
                frame.locals.pop_if(|x| x.depth > frame.scope_depth)
            {
                // Inline self.emit_instruction() to make the borrow checker happy 😾
                frame.function.chunk.write(
                    if is_captured {
                        Instruction::CloseUpvalue
                    } else {
                        Instruction::Pop
                    },
                    self.previous.line,
                );
            }
        }

        fn block(&mut self) {
            self.begin_scope();
            while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
                self.declaration();
            }
            self.consume(TokenType::RightBrace, "`}` expected after block");
            self.end_scope();
        }

        fn statement(&mut self) {
            if self.matches(TokenType::Print) {
                self.expression();
                self.consume(TokenType::Semicolon, "`;` expected after value");
                self.emit_instruction(Instruction::Print);
            } else if self.matches(TokenType::LeftBrace) {
                self.block();
            } else if self.matches(TokenType::If) {
                self.consume(TokenType::LeftParen, "`(` expected after `if`");
                self.expression();
                self.consume(TokenType::RightParen, "`)` expected after condition");
                let then_jump = self.emit_jump();
                self.emit_instruction(Instruction::Pop);
                self.statement();
                let else_jump = self.emit_jump();
                self.patch_jump(then_jump, Instruction::JumpIfFalse);
                self.emit_instruction(Instruction::Pop);
                if self.matches(TokenType::Else) {
                    self.statement();
                }
                self.patch_jump(else_jump, Instruction::Jump);
            } else if self.matches(TokenType::For) {
                self.begin_scope();
                self.consume(TokenType::LeftParen, "`(` expected after `for`");
                if self.matches(TokenType::Var) {
                    self.var_declaration();
                } else if !self.matches(TokenType::Semicolon) {
                    self.expression();
                    self.consume(TokenType::Semicolon, "`;` expected after initializer");
                }
                let mut loop_start = self.chunk_len();
                let exit_jump = if !self.matches(TokenType::Semicolon) {
                    self.expression();
                    self.consume(TokenType::Semicolon, "`;` expected after condition");
                    let exit_jump = self.emit_jump();
                    self.emit_instruction(Instruction::Pop);
                    Some(exit_jump)
                } else {
                    None
                };
                if !self.matches(TokenType::RightParen) {
                    let body_jump = self.emit_jump();
                    let increment_start = self.chunk_len();
                    self.expression();
                    self.emit_instruction(Instruction::Pop);
                    self.consume(TokenType::RightParen, "`)` expected after increment");
                    self.emit_loop(loop_start, Instruction::Jump);
                    loop_start = increment_start;
                    self.patch_jump(body_jump, Instruction::Jump);
                }
                self.statement();
                self.emit_loop(loop_start, Instruction::Jump);
                if let Some(jump) = exit_jump {
                    self.patch_jump(jump, Instruction::JumpIfFalse);
                    self.emit_instruction(Instruction::Pop);
                }
                self.end_scope();
            } else if self.matches(TokenType::While) {
                let loop_start = self.chunk_len();
                self.consume(TokenType::LeftParen, "`(` expected after `while`");
                self.expression();
                self.consume(TokenType::RightParen, "`)` expected after condition");
                let exit_jump = self.emit_jump();
                self.emit_instruction(Instruction::Pop);
                self.statement();
                self.emit_loop(loop_start, Instruction::Jump);
                self.patch_jump(exit_jump, Instruction::JumpIfFalse);
                self.emit_instruction(Instruction::Pop);
            } else if self.matches(TokenType::Return) {
                if let FunctionType::Script = self.frames.last().unwrap().function_type {
                    self.error("stray return");
                }
                if self.matches(TokenType::Semicolon) {
                    self.emit_instruction(Instruction::Nil);
                } else {
                    if let FunctionType::Initializer = self.frames.last().unwrap().function_type {
                        self.error("invalid return value for initializer");
                    }
                    self.expression();
                    self.consume(TokenType::Semicolon, "`;` expected after return value");
                }
                if let FunctionType::Initializer = self.frames.last().unwrap().function_type {
                    self.emit_instruction(Instruction::GetLocal(0));
                } else {
                    self.emit_instruction(Instruction::Return);
                }
            } else {
                self.expression();
                self.consume(TokenType::Semicolon, "`;` expected after expression");
                self.emit_instruction(Instruction::Pop);
            }
        }

        fn var_declaration(&mut self) {
            let global = self.parse_variable("variable name expected");
            if self.matches(TokenType::Equal) {
                self.expression();
            } else {
                self.emit_instruction(Instruction::Nil);
            }
            self.consume(
                TokenType::Semicolon,
                "`;` expected after variable declaration",
            );
            self.define_variable(global);
        }

        fn function(&mut self, function_type: FunctionType) {
            self.frames.push(CompilerFrame {
                function: ObjFunction {
                    arity: 0,
                    upvalue_count: 0,
                    chunk: Chunk::new(),
                    name: self.previous.lexeme.to_string(),
                },
                function_type,
                locals: vec![Local {
                    name: Token {
                        token_type: TokenType::EOF,
                        lexeme: match function_type {
                            FunctionType::Function | FunctionType::Script => "",
                            FunctionType::Method | FunctionType::Initializer => "this",
                        },
                        line: 0,
                    },
                    depth: 0,
                    is_captured: false,
                }],
                upvalues: Vec::new(),
                scope_depth: 0,
            });
            self.begin_scope();
            self.consume(TokenType::LeftParen, "`(` expected after function name");
            loop {
                if self.check(TokenType::RightParen) {
                    break;
                }
                let frame = self.frames.last_mut().unwrap();
                frame.function.arity += 1;
                if frame.function.arity > 255 {
                    self.error("too many parameters");
                }
                let constant = self.parse_variable("parameter name expected");
                self.define_variable(constant);
                if !self.matches(TokenType::Comma) {
                    break;
                }
            }
            self.consume(TokenType::RightParen, "`)` expected after parameters");
            self.consume(TokenType::LeftBrace, "`{` expected before function body");
            self.block();
            if let FunctionType::Initializer = function_type {
                self.emit_instruction(Instruction::GetLocal(0));
            } else {
                self.emit_instruction(Instruction::Nil);
            }
            self.emit_instruction(Instruction::Return);
            let CompilerFrame {
                function, upvalues, ..
            } = self.frames.pop().unwrap();
            let constant = self.make_constant(Value::Closure(Rc::new(function), Vec::new()));
            self.emit_instruction(Instruction::Closure(constant));
            for upvalue in upvalues {
                if upvalue.is_local {
                    self.emit_instruction(Instruction::LocalUpvalue(upvalue.index));
                } else {
                    self.emit_instruction(Instruction::NonlocalUpvalue(upvalue.index));
                }
            }
        }

        fn fun_declaration(&mut self) {
            let global = self.parse_variable("function name expected");
            self.make_initialized();
            self.function(FunctionType::Function);
            self.define_variable(global);
        }

        fn class_declaration(&mut self) {
            let global = self.parse_variable("class name expected");
            self.make_initialized();
            let class_name = self.previous;
            let class_name_constant =
                self.make_constant(Value::String(class_name.lexeme.to_string()));
            self.emit_instruction(Instruction::Class(class_name_constant));
            self.define_variable(global);
            let has_superclass = self.matches(TokenType::Less);
            self.enclosing_classes.push(has_superclass);
            if has_superclass {
                self.consume(TokenType::Identifier, "superclass name expected");
                self.named_variable(self.previous, false);
                if self.previous.lexeme == class_name.lexeme {
                    self.error("self inheritance");
                }

                self.begin_scope();
                self.add_local(Token {
                    token_type: TokenType::Identifier,
                    lexeme: "super",
                    line: 0,
                });
                self.define_variable(0);

                self.named_variable(class_name, false);
                self.emit_instruction(Instruction::Inherit);
            }
            self.named_variable(class_name, false);
            self.consume(TokenType::LeftBrace, "`{` expected before class body");
            while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
                self.consume(TokenType::Identifier, "method name expected");
                let constant = self.make_constant(Value::String(self.previous.lexeme.to_string()));
                self.function(if self.previous.lexeme == "init" {
                    FunctionType::Initializer
                } else {
                    FunctionType::Method
                });
                self.emit_instruction(Instruction::Method(constant));
            }
            self.consume(TokenType::RightBrace, "`}` expected after class body");
            self.emit_instruction(Instruction::Pop);
            if has_superclass {
                self.end_scope();
            }
            self.enclosing_classes.pop().unwrap();
        }

        fn declaration(&mut self) {
            if self.matches(TokenType::Var) {
                self.var_declaration();
            } else if self.matches(TokenType::Fun) {
                self.fun_declaration();
            } else if self.matches(TokenType::Class) {
                self.class_declaration();
            } else {
                self.statement();
            }
            if self.panic_mode {
                self.synchronize();
            }
        }

        pub fn compile(&mut self) -> Result<ObjFunction, ()> {
            self.advance();
            while !self.matches(TokenType::EOF) {
                self.declaration();
            }
            self.consume(TokenType::EOF, "end of expression expected");
            if self.had_error {
                return Err(());
            }
            self.emit_instruction(Instruction::Nil);
            self.emit_instruction(Instruction::Return);
            let frame = &mut self.frames[0];
            if !self.had_error {
                println!("{:?}", frame.function);
            }
            Ok(std::mem::take(&mut frame.function))
        }

        fn emit_instruction(&mut self, instruction: Instruction) {
            let frame = self.frames.last_mut().unwrap();
            frame.function.chunk.write(instruction, self.previous.line);
        }

        fn make_constant(&mut self, value: Value) -> u8 {
            let frame = self.frames.last_mut().unwrap();
            if let Some(constant) = frame.function.chunk.add_constant(value) {
                constant
            } else {
                self.error("too many constants in one chunk");
                0
            }
        }

        fn emit_constant(&mut self, value: Value) {
            let constant = self.make_constant(value);
            self.emit_instruction(Instruction::Constant(constant));
        }

        fn chunk_len(&self) -> usize {
            let frame = self.frames.last().unwrap();
            frame.function.chunk.code.len()
        }

        fn emit_jump(&mut self) -> usize {
            self.emit_instruction(Instruction::NOP);
            self.chunk_len() - 1
        }

        fn patch_jump(&mut self, jump: usize, instruction: fn(i16) -> Instruction) {
            let Ok(offset) = (self.chunk_len() - jump).try_into() else {
                self.error("jump offset too large");
                return;
            };
            let frame = self.frames.last_mut().unwrap();
            frame.function.chunk.code[jump] = instruction(offset);
        }

        fn emit_loop(&mut self, loop_start: usize, instruction: fn(i16) -> Instruction) {
            let Ok(offset) = (loop_start as isize - self.chunk_len() as isize).try_into() else {
                self.error("loop body too large");
                return;
            };
            self.emit_instruction(instruction(offset));
        }
    }
}

pub struct CallFrame {
    function: Rc<ObjFunction>,
    upvalues: Vec<Rc<RefCell<ObjUpvalue>>>, // = frame->closure->upvalues
    open_upvalues: HashMap<usize, Rc<RefCell<ObjUpvalue>>>,
    ip: usize,
    stack: Vec<Value>,
}

pub struct VM {
    frames: Vec<CallFrame>,
    globals: HashMap<String, Value>,
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
            frames: Vec::new(),
            globals: HashMap::from([(
                "clock".to_string(),
                Value::Native(|_| {
                    Value::Number(match SystemTime::now().duration_since(UNIX_EPOCH) {
                        Ok(x) => x.as_secs_f64(),
                        Err(x) => -x.duration().as_secs_f64(),
                    })
                }),
            )]),
        }
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut compiler = compiler::Compiler::new(source);
        match compiler.compile() {
            Ok(f) => self.run(Rc::new(f)),
            Err(_) => Err(InterpretError::CompileError),
        }
    }

    fn runtime_error(&mut self, message: &str) -> InterpretResult {
        eprintln!("Runtime error: {}", message);
        for frame in self.frames.iter().rev() {
            eprintln!(
                "\tat {} (line {})",
                if frame.function.name.is_empty() {
                    "(main)"
                } else {
                    &frame.function.name
                },
                frame.function.chunk.lines[frame.ip]
            );
        }
        self.frames.clear();
        Err(InterpretError::RuntimeError)
    }

    fn binary_op(&mut self, op: fn(f64, f64) -> Value) -> InterpretResult {
        let frame = self.frames.last_mut().unwrap();
        let b = frame.stack.pop().unwrap();
        let a = frame.stack.pop().unwrap();
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                frame.stack.push(op(a, b));
                Ok(())
            }
            _ => self.runtime_error("operands must be numbers"),
        }
    }

    fn call(&mut self, closure_value: Value, stack: Vec<Value>) -> InterpretResult {
        let Value::Closure(function, upvalues) = closure_value else {
            panic!()
        };
        if stack.len() - 1 != function.arity as usize {
            return self.runtime_error(&format!(
                "expected {} arguments but got {}",
                function.arity,
                stack.len() - 1
            ));
        }
        self.frames.push(CallFrame {
            function,
            upvalues,
            open_upvalues: HashMap::new(),
            ip: 0,
            stack,
        });
        Ok(())
    }

    fn run(&mut self, function: Rc<ObjFunction>) -> InterpretResult {
        self.frames.clear();
        self.frames.push(CallFrame {
            function,
            upvalues: Vec::new(),
            open_upvalues: HashMap::new(),
            ip: 0,
            stack: Vec::new(),
        });
        loop {
            let frame = self.frames.last_mut().unwrap();
            match frame.function.chunk.code[frame.ip] {
                Instruction::NOP => {}
                Instruction::Constant(constant) => {
                    let constant = frame.function.chunk.constants[constant as usize].clone();
                    frame.stack.push(constant);
                }
                Instruction::Nil => {
                    frame.stack.push(Value::Nil);
                }
                Instruction::True => {
                    frame.stack.push(Value::Boolean(true));
                }
                Instruction::False => {
                    frame.stack.push(Value::Boolean(false));
                }
                Instruction::Add => {
                    let b = frame.stack.pop().unwrap();
                    let a = frame.stack.pop().unwrap();
                    frame.stack.push(match (a, b) {
                        (a @ Value::String(_), b) | (a, b @ Value::String(_)) => {
                            Value::String(format!("{}{}", a, b))
                        }
                        (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                        _ => {
                            return self.runtime_error(
                                "operands must consist of two numbers or at least one string",
                            );
                        }
                    })
                }
                Instruction::Subtract => {
                    self.binary_op(|a, b| Value::Number(a - b))?;
                }
                Instruction::Multiply => {
                    self.binary_op(|a, b| Value::Number(a * b))?;
                }
                Instruction::Divide => {
                    self.binary_op(|a, b| Value::Number(a / b))?;
                }
                Instruction::Equal => {
                    let b = frame.stack.pop().unwrap();
                    let a = frame.stack.pop().unwrap();
                    frame.stack.push(Value::Boolean(a == b));
                }
                Instruction::Less => {
                    self.binary_op(|a, b| Value::Boolean(a < b))?;
                }
                Instruction::Greater => {
                    self.binary_op(|a, b| Value::Boolean(a > b))?;
                }
                Instruction::Not => {
                    let value = frame.stack.last_mut().unwrap();
                    *value = Value::Boolean(!value.is_truthy());
                }
                Instruction::Negate => {
                    let value = frame.stack.last_mut().unwrap();
                    match value {
                        Value::Number(x) => *x = -*x,
                        _ => return self.runtime_error("operand must be number"),
                    }
                }
                Instruction::Pop => {
                    frame.stack.pop().unwrap();
                }
                Instruction::Print => {
                    let value = frame.stack.pop().unwrap();
                    println!("{}", value);
                }
                Instruction::Return => {
                    let result = frame.stack.pop().unwrap();
                    for upvalue in frame.open_upvalues.values() {
                        upvalue.replace_with(|upvalue| match upvalue {
                            ObjUpvalue::Open(_, index) => {
                                ObjUpvalue::Closed(frame.stack[*index].clone())
                            }
                            ObjUpvalue::Closed(value) => ObjUpvalue::Closed(value.clone()),
                        });
                    }
                    self.frames.pop();
                    match self.frames.last_mut() {
                        Some(frame) => frame.stack.push(result),
                        None => return Ok(()),
                    }
                }
                Instruction::DefineGlobal(name) => {
                    let Value::String(name) = &frame.function.chunk.constants[name as usize] else {
                        panic!("bad operand of DefineGlobal")
                    };
                    self.globals
                        .insert(name.clone(), frame.stack.pop().unwrap());
                }
                Instruction::GetGlobal(name) => {
                    let Value::String(name) = &frame.function.chunk.constants[name as usize] else {
                        panic!("bad operand of GetGlobal")
                    };
                    let Some(value) = self.globals.get(name) else {
                        return self.runtime_error("undefined variable");
                    };
                    frame.stack.push(value.clone());
                }
                Instruction::SetGlobal(name) => {
                    let Value::String(name) = &frame.function.chunk.constants[name as usize] else {
                        panic!("bad operand of SetGlobal")
                    };
                    if let None = self
                        .globals
                        .insert(name.clone(), frame.stack.last().unwrap().clone())
                    {
                        self.globals.remove(name);
                        return self.runtime_error("undefined variable");
                    }
                }
                Instruction::GetLocal(slot) => {
                    frame.stack.push(frame.stack[slot as usize].clone());
                }
                Instruction::SetLocal(slot) => {
                    frame.stack[slot as usize] = frame.stack.last().unwrap().clone();
                }
                Instruction::GetUpvalue(slot) => {
                    let value = match (*self.frames.last().unwrap().upvalues[slot as usize]
                        .as_ref()
                        .borrow())
                    .clone()
                    {
                        ObjUpvalue::Open(frame_index, index) => {
                            self.frames[frame_index].stack[index].clone()
                        }
                        ObjUpvalue::Closed(value) => value.clone(),
                    };
                    self.frames.last_mut().unwrap().stack.push(value);
                }
                Instruction::SetUpvalue(slot) => {
                    let value = frame.stack.last().unwrap().clone();
                    match {
                        let mut upvalue = self.frames.last().unwrap().upvalues[slot as usize]
                            .as_ref()
                            .borrow_mut();
                        match *upvalue {
                            ObjUpvalue::Open(frame_index, index) => Some((frame_index, index)),
                            ObjUpvalue::Closed(_) => {
                                *upvalue = ObjUpvalue::Closed(value.clone());
                                None
                            }
                        }
                    } {
                        Some((frame_index, index)) => self.frames[frame_index].stack[index] = value,
                        None => {}
                    }
                }
                Instruction::Jump(offset) => {
                    frame.ip = frame.ip.wrapping_add_signed(offset as isize);
                    continue;
                }
                Instruction::JumpIfFalse(offset) => {
                    if !frame.stack.last().unwrap().is_truthy() {
                        frame.ip = frame.ip.wrapping_add_signed(offset as isize);
                        continue;
                    }
                }
                Instruction::Call(arg_count) => {
                    let mut stack = frame
                        .stack
                        .split_off(frame.stack.len() - arg_count as usize - 1);
                    match &stack[0] {
                        closure_value @ Value::Closure(_, _) => {
                            self.call(closure_value.clone(), stack)?;
                            continue;
                        }
                        Value::Native(_) => {
                            let Value::Native(native) = stack.remove(0) else {
                                panic!()
                            };
                            frame.stack.push(native(stack));
                        }
                        Value::Class(class) => {
                            let class = Rc::clone(class);
                            let instance = Value::Instance(Rc::new(RefCell::new(ObjInstance {
                                class: Rc::clone(&class),
                                fields: HashMap::new(),
                            })));
                            stack[0] = instance.clone();
                            frame.stack.push(instance);
                            if let Some(initializer) = class.borrow().methods.get("init") {
                                self.call(initializer.clone(), stack)?;
                                continue;
                            } else if arg_count != 0 {
                                return self
                                    .runtime_error("too many arguments for default constructor");
                            }
                        }
                        Value::BoundMethod(bound_method) => {
                            let class = bound_method.class.borrow();
                            let closure_value = class
                                .methods
                                .get(&bound_method.method_name)
                                .unwrap()
                                .clone();
                            drop(class);
                            stack[0] = Value::Instance(Rc::clone(&bound_method.receiver));
                            self.call(closure_value, stack)?;
                            continue;
                        }
                        _ => return self.runtime_error("bad callee"),
                    }
                }
                Instruction::Closure(constant) => {
                    let current_frame_index = self.frames.len() - 1;
                    let frame = &mut self.frames[current_frame_index];
                    let Value::Closure(function, _) =
                        frame.function.chunk.constants[constant as usize].clone()
                    else {
                        panic!()
                    };
                    let upvalues = (0..function.upvalue_count)
                        .map(|_| {
                            frame.ip += 1;
                            match frame.function.chunk.code[frame.ip] {
                                Instruction::LocalUpvalue(index) => Rc::clone(
                                    frame.open_upvalues.entry(index as usize).or_insert_with(
                                        || {
                                            Rc::new(RefCell::new(ObjUpvalue::Open(
                                                current_frame_index,
                                                index as usize,
                                            )))
                                        },
                                    ),
                                ),
                                Instruction::NonlocalUpvalue(index) => {
                                    Rc::clone(&frame.upvalues[index as usize])
                                }
                                _ => panic!("not enough upvalue instructions"),
                            }
                        })
                        .collect();
                    frame.stack.push(Value::Closure(function, upvalues));
                }
                Instruction::LocalUpvalue(_) | Instruction::NonlocalUpvalue(_) => {
                    unreachable!("upvalue instructions should not be executed themselves")
                }
                Instruction::CloseUpvalue => {
                    let current_frame_index = self.frames.len() - 1;
                    let frame = &mut self.frames[current_frame_index];
                    let value = frame.stack.pop().unwrap();
                    let upvalue = frame.open_upvalues.get(&frame.stack.len()).unwrap();
                    *upvalue.borrow_mut() = ObjUpvalue::Closed(value);
                }
                Instruction::Class(constant) => {
                    let Value::String(name) =
                        frame.function.chunk.constants[constant as usize].clone()
                    else {
                        panic!()
                    };
                    frame
                        .stack
                        .push(Value::Class(Rc::new(RefCell::new(ObjClass {
                            name,
                            methods: HashMap::new(),
                        }))));
                }
                Instruction::GetProperty(constant) => {
                    let Value::Instance(instance) = frame.stack.pop().unwrap() else {
                        return self.runtime_error("not an instance");
                    };
                    let instance_borrowed = instance.borrow();
                    let Value::String(name) = &frame.function.chunk.constants[constant as usize]
                    else {
                        panic!()
                    };
                    if let Some(value) = instance_borrowed.fields.get(name) {
                        frame.stack.push(value.clone());
                    } else if instance_borrowed.class.borrow().methods.contains_key(name) {
                        frame.stack.push(Value::BoundMethod(Rc::new(ObjBoundMethod {
                            receiver: Rc::clone(&instance),
                            class: Rc::clone(&instance_borrowed.class),
                            method_name: name.clone(),
                        })));
                    } else {
                        return self.runtime_error("undefined property");
                    }
                }
                Instruction::SetProperty(constant) => {
                    let value = frame.stack.pop().unwrap();
                    let Value::Instance(instance) = frame.stack.pop().unwrap() else {
                        return self.runtime_error("not an instance");
                    };
                    let Value::String(name) = &frame.function.chunk.constants[constant as usize]
                    else {
                        panic!()
                    };
                    instance
                        .borrow_mut()
                        .fields
                        .insert(name.clone(), value.clone());
                    frame.stack.push(value);
                }
                Instruction::Method(constant) => {
                    let Value::String(name) =
                        frame.function.chunk.constants[constant as usize].clone()
                    else {
                        panic!()
                    };
                    let method = frame.stack.pop().unwrap();
                    let Some(Value::Class(class)) = frame.stack.last() else {
                        panic!()
                    };
                    class.borrow_mut().methods.insert(name, method);
                }
                Instruction::Inherit => {
                    let Some(Value::Class(subclass)) = frame.stack.pop() else {
                        panic!()
                    };
                    let Some(Value::Class(superclass)) = frame.stack.last().cloned() else {
                        return self.runtime_error("invalid superclass");
                    };
                    subclass
                        .borrow_mut()
                        .methods
                        .extend(superclass.borrow().methods.clone());
                }
                Instruction::GetSuper(constant) => {
                    let Value::String(name) = &frame.function.chunk.constants[constant as usize]
                    else {
                        panic!()
                    };
                    let Some(Value::Class(superclass)) = frame.stack.pop() else {
                        panic!()
                    };
                    let Value::Instance(instance) = frame.stack.pop().unwrap() else {
                        return self.runtime_error("not an instance");
                    };
                    if superclass.borrow().methods.contains_key(name) {
                        frame.stack.push(Value::BoundMethod(Rc::new(ObjBoundMethod {
                            receiver: Rc::clone(&instance),
                            class: Rc::clone(&superclass),
                            method_name: name.clone(),
                        })));
                    } else {
                        return self.runtime_error("undefined property");
                    }
                }
            }
            // Re-borrow to make the borrow checker happy 😾
            let frame = self.frames.last_mut().unwrap();
            frame.ip += 1;
        }
    }
}
