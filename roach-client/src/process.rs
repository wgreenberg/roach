use tokio::process::{Command, Child, ChildStdin, ChildStdout};
use tokio::io::{BufReader, AsyncBufReadExt, Lines, AsyncWriteExt};
use std::process::Stdio;

pub struct Process {
    stdin: ChildStdin,
    output: Lines<BufReader<ChildStdout>>,
}

impl Process {
    pub fn new(cmd_str: &str) -> Process {
        let mut cmd = Command::new(cmd_str);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        let mut child = cmd.spawn().expect("failed to spawn command");
        let stdout = child.stdout.take().expect("child did not have stdout");
        let stdin = child.stdin.take().expect("child did not have stdin");
        let mut output = BufReader::new(stdout).lines();
        tokio::spawn(async move {
            let status = child.await
                .expect("child process encountered an error");
            println!("child status was {}", status);
        });
        Process { stdin, output }
    }

    pub async fn send(&mut self, input: &str, stop_on_ok: bool) -> String {
        let mut input_bytes: Vec<u8> = input.as_bytes().into();
        input_bytes.push(b'\n');
        let n = self.stdin.write(&input_bytes).await.expect("couldn't write to process");
        let mut lines = Vec::new();
        while let Some(line) = self.output.next_line().await.expect("couldn't read line") {
            lines.push(line.clone());
            if stop_on_ok {
                if line == "ok" {
                    break
                }
            } else {
                break;
            }
        }
        lines.join("\n")
    }
}
