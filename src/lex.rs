
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Operators
    Assign,
    Plus,
    Minus,
    Multiply,
    Equals,
    Less,
    Modulo,

    // Delimiters
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,

    // Values 
    Identifier(String),

    // Literals
    // TODO: None
    True,
    False,
    IntLiteral(i32),

    // Keywords
    Print,
    If,
    Else,
    While,
    Break,
    Continue,
    Function,
    Return,
    Let,
    Struct,
    
    // Type stuff
    Int, 
    Bool,
    RightArrow,
}


pub fn lex(program: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = program.chars().peekable();
    
    while let Some(&c) = chars.peek() {

        if c.is_whitespace() {
            chars.next();
            continue;
        }


        // Alphanumeric strings: keywords or identifiers
        if c.is_ascii_alphabetic() {
            let mut word = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    word.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }

            let token = match word.as_str() {
                "if" => Token::If,
                "else" => Token::Else,
                "while" => Token::While,
                "break" => Token::Break,
                "continue" => Token::Continue,
                "fun" => Token::Function,
                "return" => Token::Return,
                "print" => Token::Print,
                "let" => Token::Let,
                "int" => Token::Int,
                "bool" => Token::Bool,
                "true" => Token::True,
                "false" => Token::False,
                "struct" => Token::Struct,
                _ => Token::Identifier(word),
            };  
            tokens.push(token); 
        } 
        
        // Numbers
        else if c.is_ascii_digit() {
            let mut num_str = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_digit() {
                    num_str.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            let value = num_str.parse::<i32>().unwrap();
            tokens.push(Token::IntLiteral(value));
        }

        else if c == '=' {
            chars.next();
            if chars.peek() == Some(&'=') {
                chars.next();
                tokens.push(Token::Equals);
            } else {
                tokens.push(Token::Assign);
            }
        }

        else if c == '-' {
            chars.next();
            if chars.peek() == Some(&'>') {
                chars.next();
                tokens.push(Token::RightArrow);
            } else {
                tokens.push(Token::Minus);
            }
        }

        else {
            // Processing single character stuff
            let token = match c {
                '+' => Token::Plus,
                '*' => Token::Multiply,
                ';' => Token::Semicolon,
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                '{' => Token::LeftBrace,
                '}' => Token::RightBrace,
                '<' => Token::Less,
                '%' => Token::Modulo,
                ',' => Token::Comma,
                ':' => Token::Colon,
                _ => {panic!("Unexpected character: {}",c)},
            };
            chars.next();
            tokens.push(token);
        }
        
    } 
    tokens
}


