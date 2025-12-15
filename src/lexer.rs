#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Token {
    // =
    Assign,
    // &
    Ampersand,
    String(String)
}

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

#[derive(Debug)]
enum State {
    Root,
    AccumulateQuotedString,
    AccumulateUnquotedString,
    EscapingChar,
}

pub(crate) struct Lexer<'a> {
    accumulator: String,
    state: State,
    input: &'a str
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            accumulator: Default::default(),
            state: State::Root,
            input
        }
    }
}

impl Lexer<'_> {
    fn peek_char(&mut self) -> Option<char> {
        self.input.chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Some(ch)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ch = self.peek_char();
            match self.state {
                State::Root => {
                    match ch {
                        Some('"') => {
                            self.next_char();
                            self.state = State::AccumulateQuotedString;
                        },
                        Some('&') => {
                            self.next_char();
                            return Some(Token::Ampersand)
                        },
                        Some('=') => {
                            self.next_char();
                            return Some(Token::Assign)
                        },
                        Some(_) => {
                            self.state = State::AccumulateUnquotedString;
                        },
                        None => {
                            self.next_char();
                            return None
                        }
                    }
                },
                State::AccumulateQuotedString => {
                    match ch {
                        Some('"') => {
                            self.next_char();
                            self.state = State::Root;
                            return Some(Token::String(std::mem::take(&mut self.accumulator)));
                        },
                        Some('\\') => {
                            self.next_char();
                            self.state = State::EscapingChar; 
                        },
                        _ => {
                            self.next_char();
                            self.accumulator.push(ch.unwrap());
                        }
                    }
                },
                State::AccumulateUnquotedString => {
                    if ch == Some('&') || ch == Some('=') || ch == None {
                        self.state = State::Root;
                        return Some(Token::String(std::mem::take(&mut self.accumulator)));
                    }
                    
                    self.next_char();
                    self.accumulator.push(ch.unwrap());
                },
                State::EscapingChar => {
                    if let Some(c) = ch {
                        self.next_char();
                        self.state = State::AccumulateQuotedString;
                        self.accumulator.push(c);
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, Token};

    #[test]
    fn test_lexer() {
        let lexer = Lexer::new("arg0=\"arg2\\\"\"&arg3=10");
        let expected = vec![
            Token::from("arg0"),
            Token::Assign,
            Token::from("arg2\""),
            Token::Ampersand,
            Token::from("arg3"),
            Token::Assign,
            Token::from("10")
        ];
        let got = lexer.collect::<Vec<_>>();

        assert_eq!(expected, got);
    }
}