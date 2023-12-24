//  A line comment starts with a double slash and ends at the first newline
//  A block comment starts with slash star and ends at the first star slash
//  A double slash within a string literal does not start a comment
//  Slash star within a string literal does not start a comment
//  A string literal starts with doublequote and ends at the first double quote
//  A string literal does not end at a double quote preceded by a backslash
//  
#[derive(Debug, PartialEq)]
pub enum Token {
    Char(char),
    Star,
    NewLine,
    FrontSlash,
    DoubleQuote,
}

#[derive(Debug, PartialEq)]
pub enum El {
    SingleLineComment(String),
    BlockComment(String),
    Code(String),
}
pub enum State {
    SingleLineComment,
    BlockComment,
    BlockCommentEnd,
    Comment,
    Code,
    StrLiteral,
}
fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for c in input.chars() {
        match c {
            '*' => tokens.push(Token::Star),
            '\n' => tokens.push(Token::NewLine),
            '/' => tokens.push(Token::FrontSlash),
            '"' => tokens.push(Token::DoubleQuote),
            _ => tokens.push(Token::Char(c)),
        }
    }
    tokens
}

fn parse(tokens: Vec<Token>) -> Vec<El> {
    let mut state = State::Code;
    let mut built = String::new();
    let mut parsed :Vec<El>= vec![];

    for t in tokens {
        state = match state {
        State::Code => {
            match t {
                Token::FrontSlash => {
                    // push built to parsed
                    if built.len() > 0 {
                        parsed.push(El::Code(built.clone()));
                        built.clear();
                    }
                    State::Comment
                },
                Token::Char(c)=> {
                    built.push(c);
                    State::Code
                }
                Token::Star => {
                    built.push('*');
                    State::Code
                },
                Token::NewLine => {
                    parsed.push(El::Code(built.clone()));
                    built.clear();
                    State::Code
                },
                Token::DoubleQuote => {
                    if built.len() > 0 {
                        parsed.push(El::Code(built.clone()));
                        built.clear();
                    }
                    State::StrLiteral
                },
            }
        }
        State::Comment => {
            match t {
                Token::FrontSlash =>
                    State::SingleLineComment,
                Token::Star => 
                    State::BlockComment
                ,
                t => {
                    panic!("Unexpected token: {:?}", t);
                }
            }
        },
        State::SingleLineComment => {
            match t {
                Token::NewLine => {
                    parsed.push(El::SingleLineComment(built.clone()));
                    built.clear();
                    State::Code
                },
                Token::Char(c) => {
                    built.push(c);
                    State::SingleLineComment
                },
                Token::Star => {
                    built.push('*');
                    State::SingleLineComment
                },
                Token::FrontSlash => {
                    built.push('/');
                    State::SingleLineComment
                },
                Token::DoubleQuote => {
                    built.push('"');
                    State::SingleLineComment
                },
            }
        },
        State::BlockComment => {
            match t {
                Token::Star => {
                    State::BlockCommentEnd
                },
                Token::Char(c) => {
                    built.push(c);
                    State::BlockComment
                },
                Token::FrontSlash => {
                    built.push('/');
                    State::BlockComment
                },
                Token::DoubleQuote => {
                    State::BlockComment
                },
                Token::NewLine => {
                    built.push('\n');
                    State::BlockComment
                }
            }
        }
        State::StrLiteral => {
            // TODO: collect the string literal
            // TODO: Handle escaped chars
            match t {
                Token::DoubleQuote => {
                    parsed.push(El::Code(built.clone()));
                    built.clear();
                    State::Code
                }
                Token::FrontSlash => {
                    built.push('/');
                    State::StrLiteral
                }
                Token::Star => {
                    built.push('*');
                    State::StrLiteral
                }
                Token::NewLine => {
                    built.push('\n');
                    State::StrLiteral
                }
                Token::Char(c) => {
                    built.push(c);
                    State::StrLiteral
                }
            }
        }
        State::BlockCommentEnd => {
            match t {
                Token::FrontSlash => {
                    parsed.push(El::BlockComment(built.clone()));
                    built.clear();
                    State::Code
                },
                Token::Star => {
                    built.push('*');
                    built.push('*');
                    State::BlockComment
                },
                Token::DoubleQuote => {
                    built.push('*');
                    built.push('"');
                    State::BlockComment
                },
                Token::Char(c) => {
                    built.push('*');
                    built.push(c);
                    State::BlockComment
                },
                Token::NewLine => {
                    built.push('*');
                    built.push('\n');
                    State::BlockComment
                },
                }
            }
        } 
    }
    parsed
}
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide a filepath as a command line argument.");
        return;
    }
    let filepath = &args[1];
    // read the file
    let contents = std::fs::read_to_string(filepath)
        .expect("Something went wrong reading the file");
    let tokens = lex(&contents);
    let parsed = parse(tokens);
    println!("{:#?}", parsed);
}

#[cfg(test)]
mod tests {
    use crate::{lex, parse, El};

    #[test]
    fn single_line_comment() {
        let input = "//single line comment\n";
        let output = lex(input);
        let parsed = parse(output);
        assert_eq!(parsed, vec![
            El::SingleLineComment("single line comment".into())
        ]);

    }
    #[test]
    fn block_comment() {
        let input = "/*block comment*/";
        let output = lex(input);
        let parsed = parse(output);
        assert_eq!(parsed, vec![
            El::BlockComment("block comment".into())
        ]);
    }

    #[test]
    fn code() {
        let input = "let x = 1;\n";
        let output = lex(input);
        let parsed = parse(output);
        assert_eq!(parsed, vec![
            El::Code("let x = 1;".into())
        ]);
    }

    #[test]
    fn multiline_comment() {
        let input = "/*\n*multi line block comment\n*/";
        let output = lex(input);
        let parsed = parse(output);
        assert_eq!(parsed, vec![
            El::BlockComment("\n*multi line block comment\n".into())
        ]);
    }

    #[test]
    fn single_line_comment_w_fake_block_comment() {
        let input = "//single line comment /* fake block comment */\n";
        let output = lex(input);
        let parsed = parse(output);
        assert_eq!(parsed, vec![
            El::SingleLineComment("single line comment /* fake block comment */".into())
        ]);
    }

    #[test]
    fn block_comment_w_fake_single_line_comment() {
        let input = "/*block comment // fake single line comment */";
        let output = lex(input);
        let parsed = parse(output);
        assert_eq!(parsed, vec![
            El::BlockComment("block comment // fake single line comment ".into())
        ]);
    }

    #[test]
    fn string_w_slash() {
        let input = "\"/string w/ slash /\"";
        let output = lex(input);
        let parsed = parse(output);
        assert_eq!(parsed, vec![
            El::Code("/string w/ slash /".into())
        ]);
    }
    #[test]
    fn string_w_star() {
        let input = "\"*string w/ star *\"";
        let output = lex(input);
        let parsed = parse(output);
        assert_eq!(parsed, vec![
            El::Code("*string w/ star *".into())
        ]);
    }
    
    #[test]
    fn single_line_comment_from_slashes() {
        let input = "//////\n";
        let output = lex(input);
        let parsed = parse(output);
        assert_eq!(parsed, vec![
            El::SingleLineComment("////".into())
        ]);
    }
}
