use super::LexError;

pub(crate) struct Cursor<'a> {
    line: usize,
    col: usize,
    index: usize,
    source: &'a str,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            line: 1,
            col: 1,
            index: 0,
            source,
        }
    }

    pub(crate) fn is_end(&self) -> bool {
        self.peek_token().is_none()
    }

    fn get_token(&self) -> std::ops::Range<usize> {
        fn is_separator(c: char) -> bool {
            c.is_whitespace() || c == '\n'
        }
        let next_token_start = self.source[self.index..]
            .find(|c: char| !is_separator(c))
            .map_or(self.source.len(), |i| i + self.index);
        let next_token_end = self.source[next_token_start..]
            .trim_start()
            .find(is_separator)
            .map_or(self.source.len(), |i| i + next_token_start);
        next_token_start..next_token_end
    }

    pub(crate) fn peek_token(&self) -> Option<&'a str> {
        let ret = self.get_token();
        if ret.is_empty() {
            return None;
        }
        Some(&self.source[ret])
    }

    pub(crate) fn next_token(&mut self) -> Option<&'a str> {
        let ret = self.get_token();
        if ret.is_empty() {
            return None;
        }
        let advanced_lines = self.source[self.index..ret.end]
            .chars()
            .filter(|&c| c == '\n')
            .count();
        self.line += advanced_lines;
        if advanced_lines != 0 {
            self.col = 1;
        }
        self.col += self.source[self.index..ret.end]
            .lines()
            .last()
            .unwrap()
            .chars()
            .count();
        self.index = ret.end;
        Some(&self.source[ret])
    }

    pub(crate) fn next_line_remaining(&mut self) -> &'a str {
        let spaces = self.source[self.index..]
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(self.source.len());

        self.col += spaces;
        self.index += spaces;

        let remaining_end = self.source[self.index..]
            .find('\n')
            .unwrap_or(self.source.len());
        let ret = if self
            .source
            .get(self.index + remaining_end - 1..=self.index + remaining_end)
            == Some("\r\n")
        {
            &self.source[self.index..self.index + remaining_end - 1]
        } else {
            &self.source[self.index..self.index + remaining_end]
        };
        self.col += ret.chars().count();
        self.index += remaining_end;
        ret
    }

    pub(crate) fn line(&self) -> usize {
        self.line
    }

    pub(crate) fn col(&self) -> usize {
        self.col
    }

    pub(crate) fn err_expected_token(&self, message: &'static str) -> LexError {
        LexError::ExpectedToken {
            line: self.line(),
            col: self.col(),
            message,
        }
    }
}

#[test]
fn test1() {
    let mut cursor = Cursor::new(
        r"
            hoge
            foo
            bar bar
        ",
    );

    assert_eq!(cursor.line(), 1);
    assert_eq!(cursor.col(), 1);
    assert_eq!(cursor.next_token(), Some("hoge"));
    assert_eq!(cursor.line(), 2);
    assert_eq!(cursor.col(), 17);
    assert_eq!(cursor.next_token(), Some("foo"));
    assert_eq!(cursor.line(), 3);
    assert_eq!(cursor.col(), 16);
    assert_eq!(cursor.next_token(), Some("bar"));
    assert_eq!(cursor.line(), 4);
    assert_eq!(cursor.col(), 16);
    assert_eq!(cursor.next_token(), Some("bar"));
    assert_eq!(cursor.line(), 4);
    assert_eq!(cursor.col(), 20);
}

#[test]
fn test2() {
    const SOURCE: &str = r"
        #TITLE 花たちに希望を [SP ANOTHER]
        #ARTIST Sound piercer feat.DAZBEE
        #BPM 187
    ";

    let mut cursor = Cursor::new(SOURCE);

    assert_eq!(cursor.next_token(), Some("#TITLE"));
    assert_eq!(cursor.next_line_remaining(), "花たちに希望を [SP ANOTHER]");
    assert_eq!(cursor.next_token(), Some("#ARTIST"));
    assert_eq!(cursor.next_line_remaining(), "Sound piercer feat.DAZBEE");
    assert_eq!(cursor.next_token(), Some("#BPM"));
    assert_eq!(cursor.next_line_remaining(), "187");
}
