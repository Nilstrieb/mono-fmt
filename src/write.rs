use crate::{Result, Write};

impl Write for String {
    fn write_str(&mut self, str: &str) -> Result {
        self.push_str(str);
        Ok(())
    }

    fn write_char(&mut self, char: char) -> Result {
        self.push(char);
        Ok(())
    }
}

impl<W: Write> Write for &mut W {
    fn write_str(&mut self, str: &str) -> Result {
        <W as Write>::write_str(self, str)
    }

    fn write_char(&mut self, char: char) -> Result {
        <W as Write>::write_char(self, char)
    }
}
