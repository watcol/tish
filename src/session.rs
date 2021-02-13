use crate::job::CurPid;
use crate::cmd::Cmd;
use crate::parse::AstCmd;

pub struct Session<T> {
    reader: T,
    cur_pid: CurPid,
}

pub trait Reader: Sized {
    fn init(cur_pid: &CurPid) -> anyhow::Result<Self>;
    fn next_line(&mut self) -> anyhow::Result<String>;
}

impl<T: Reader> Session<T> {
    pub fn new() -> anyhow::Result<Self> {
        let cur_pid = CurPid::new();
        Ok(Self {
            reader: T::init(&cur_pid)?,
            cur_pid,
        })
    }

    pub fn next(&mut self) -> anyhow::Result<bool> {
        let line = match self.reader.next_line() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Readline Error: {}", e);
                return Ok(true);
            }
        };

        let cmd = match AstCmd::parse(line) {
            Ok(Some(res)) => res,
            Ok(None) => return Ok(true),
            Err(e) => {
                eprintln!("Parse Error: {}", e);
                return Ok(true);
            }
        };

        let id = match cmd.name.as_str() {
            "exit" => return Ok(false),
            s => match Cmd::new(s).exec(cmd.args) {
                Ok(Some(id)) => id,
                Ok(None) => return Ok(true),
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(true);
                }
            },
        };

        self.cur_pid.store(id)?;
        id.wait()?;
        self.cur_pid.reset()?;

        Ok(true)
    }

    pub fn all(&mut self) -> anyhow::Result<()> {
        loop {
            if !self.next()? {
                break;
            }
        }

        Ok(())
    }
}