/// book contains a Rust implementation of the source code from the book which
/// attemps to maintain the spirit of the original c source.
pub mod book {
    /// checks if `regexp` matches `text`, using the limited syntax:
    ///
    /// | character | meaning                                              |
    /// |-----------|------------------------------------------------------|
    /// | c         | matches any literal character `c`                    |
    /// | .         | matches any single character                         |
    /// | ^         | matches the beginning of input                       |
    /// | $         | matches the end of input                             |
    /// | *         | matches zero or more occurences of the previous char |
    ///
    pub fn match_regexp(regexp: &str, mut text: &str) -> bool {
        if regexp.starts_with('^') {
            // then check if the rest of `regexp` matches `text` from the start.
            match_here(&regexp[1..], text)
        } else {
            // otherwise, we start looping.
            loop {
                // if the regexp matches the remaining text we return true
                if match_here(regexp, text) {
                    break true;
                }

                // if text is empty (& that didn't match the regex) we return false
                if text.is_empty() {
                    break false;
                }

                // otherwise we move on to the next character.
                text = &text[1..]
            }
        }
    }

    fn match_here(regexp: &str, text: &str) -> bool {
        if regexp.is_empty() {
            // matches anything.
            true
        } else if regexp.chars().nth(1) == Some('*') {
            // match the first char repeatedly
            match_star(regexp.chars().next().unwrap(), &regexp[2..], text)
        } else if regexp == "$" {
            // only match the end of `text`.
            text.is_empty()
        // If we have text remaining
        } else if !text.is_empty()
        // and the match is either any char, or the first char of the text
        && (regexp.starts_with('.') || regexp.starts_with(text.chars().next().unwrap()))
        {
            // move on to the next match
            match_here(&regexp[1..], &text[1..])
        } else {
            false
        }
    }

    fn match_star(starred: char, regexp: &str, mut text: &str) -> bool {
        loop {
            // check if we can match 0 instances of `starred` on `text`
            if match_here(regexp, text) {
                break true;
            }

            // if `text` is empty, and `starred` is neither `.` nor matches the start
            // of `text`...
            if text.is_empty() || !(starred == '.' || text.starts_with(starred)) {
                // then we know we don't match
                break false;
            }

            // otherwise, move on to the next character
            text = &text[1..]
        }
    }
}

/// rs contains a reimplementation of the source code from the book which has
/// been modernised, but is more-or-less an equivalent algorithm.
pub mod rs {
    pub fn match_regexp(regexp: &str, text: &str) -> bool {
        use parse::Tokenize;

        Matcher::new(regexp.tokenize()).is_match(text)
    }

    #[derive(Clone)]
    struct Matcher<Ts: Iterator<Item = parse::Token>> {
        tokens: std::iter::Peekable<Ts>,
    }

    impl<Ts: Iterator<Item = parse::Token> + Clone> Matcher<Ts> {
        fn new(tokens: Ts) -> Matcher<Ts> {
            Self {
                tokens: tokens.peekable(),
            }
        }

        fn is_match(mut self, text: &str) -> bool {
            match self.tokens.peek() {
                Some(parse::Token::Start) => Matcher::new(self.tokens.skip(1)).match_here(text),
                _ => self.match_rest(text),
            }
        }

        fn match_rest(&mut self, text: &str) -> bool {
            if self.clone().match_here(text) {
                true
            } else if text.is_empty() {
                false
            } else {
                self.match_rest(&text[1..])
            }
        }

        fn match_here(&mut self, text: &str) -> bool {
            match self.tokens.next() {
                None => true,
                Some(parse::Token::ZeroOrMore(c)) => self.match_star(c, text),
                Some(parse::Token::End) => text.is_empty(),
                Some(parse::Token::Single(parse::Single::Any)) if !text.is_empty() => {
                    self.match_here(&text[1..])
                }
                Some(parse::Token::Single(parse::Single::Literal(c))) if text.starts_with(c) => {
                    self.match_here(&text[1..])
                }
                Some(parse::Token::Start) => panic!("$ token in illegal position"),
                _ => false,
            }
        }

        fn match_star(&self, c: parse::Single, text: &str) -> bool {
            if self.clone().match_here(text) {
                true
            } else if text.is_empty() {
                false
            } else {
                self.match_star(c, &text[1..])
            }
        }
    }

    mod parse {
        #[derive(Copy, Clone, Debug)]
        pub enum Token {
            Single(Single),
            Start,
            End,
            ZeroOrMore(Single),
        }

        #[derive(Copy, Clone, Debug)]
        pub enum Single {
            Any,
            Literal(char),
        }

        impl From<char> for Single {
            fn from(c: char) -> Self {
                match c {
                    '.' => Self::Any,
                    _ => Self::Literal(c),
                }
            }
        }

        pub trait Tokenize {
            fn tokenize(&self) -> Tokens<'_>;
        }

        impl Tokenize for str {
            fn tokenize<'src>(&'src self) -> Tokens<'src> {
                Tokens(self.chars().peekable())
            }
        }

        #[derive(Clone)]
        pub struct Tokens<'src>(std::iter::Peekable<std::str::Chars<'src>>);

        impl<'src> Iterator for Tokens<'src> {
            type Item = Token;

            fn next(&mut self) -> Option<Self::Item> {
                self.0.next().map(|nxt| match nxt {
                    '^' => Token::Start,
                    '$' => Token::End,
                    c if self.0.peek().copied() == Some('*') => {
                        self.0.next();
                        Token::ZeroOrMore(c.into())
                    }
                    c => Token::Single(c.into()),
                })
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{book, rs};

    #[test]
    fn same_results() {
        for (regexp, text) in [
            ("^abacus$", "abacus"),
            (".*frog", "aaaaafrog"),
            ("nomatch", "wat"),
        ] {
            let book = book::match_regexp(regexp, text);
            let rs = rs::match_regexp(regexp, text);
            assert_eq!(
                book, rs,
                "example failed: {text} =~ /{regexp}/; book={book}, rs={rs}"
            );
        }
    }
}
