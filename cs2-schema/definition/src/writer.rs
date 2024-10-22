use std::{
    borrow::Cow,
    fs::File,
    io::{
        self,
        BufWriter,
        Write,
    },
    path::Path,
};

pub trait EmitOutput {
    fn emit_line(&mut self, line: &str) -> io::Result<()>;

    fn push_ident(&mut self);
    fn pop_ident(&mut self);
}

pub struct FileEmitter {
    writer: BufWriter<File>,
    ident_stack: Vec<String>,
}

impl FileEmitter {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            ident_stack: Vec::with_capacity(8),
        })
    }

    fn current_ident(&self) -> &str {
        self.ident_stack.last().map_or("", |v| v.as_str())
    }
}

impl EmitOutput for FileEmitter {
    fn emit_line(&mut self, line: &str) -> io::Result<()> {
        let ident = self.ident_stack.last().map_or("".into(), Cow::from);
        writeln!(&mut self.writer, "{}{}", ident, line)
    }

    fn push_ident(&mut self) {
        self.ident_stack
            .push(format!("{}    ", self.current_ident()));
    }

    fn pop_ident(&mut self) {
        let _ = self.ident_stack.pop();
    }
}
