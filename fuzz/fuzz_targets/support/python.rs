use std::cell::RefCell;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

struct Oracle {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Oracle {
    fn start() -> Self {
        let python = std::env::var_os("ASTRAL_INI_PYTHON").unwrap_or_else(|| "python3".into());
        let mut child = Command::new(python)
            .arg("-u")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("python_oracle.py"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("start Python ConfigParser oracle");
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        Self {
            child,
            input,
            output,
        }
    }

    fn inspect(&mut self, request: &Value) -> Value {
        serde_json::to_writer(&mut self.input, request).unwrap();
        writeln!(self.input).unwrap();
        self.input.flush().unwrap();
        let mut response = String::new();
        assert_ne!(
            self.output.read_line(&mut response).unwrap(),
            0,
            "Python oracle exited"
        );
        serde_json::from_str(&response).expect("Python oracle returned JSON")
    }
}

impl Drop for Oracle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Compare all dialects through one persistent Python process per fuzz worker.
pub(crate) fn compare(input: &str, lookups: &[(&str, &str)]) {
    thread_local! {
        static ORACLE: RefCell<Oracle> = RefCell::new(Oracle::start());
    }

    ORACLE.with(|oracle| {
        let mut oracle = oracle.borrow_mut();
        for (case_sensitive, delimiters, options) in super::support::profiles() {
            let request = json!({
                "input": input,
                "case_sensitive": case_sensitive,
                "delimiters": delimiters,
                "lookups": lookups,
            });
            assert_eq!(
                super::support::snapshot(input, options, lookups),
                oracle.inspect(&request),
                "{request}"
            );
        }
    });
}
