use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use futures::prelude::*;
use tokio::io::{read_to_end, ReadToEnd};
use tokio_process::{Child, ChildStderr, ChildStdout, CommandExt};
use tokio_timer::{sleep, Delay};

/// Runs a command with a timeout.
///
/// # Examples
///
/// ```
/// # use futures::Future;
/// # use gameserver::util::{TimeoutCommand, TimeoutCommandError};
/// # use std::time::Duration;
/// let cmd = TimeoutCommand::new("sleep", &["60"], Duration::from_secs(1)).unwrap();
/// tokio::run(cmd.then(|r| match r {
///     Err(TimeoutCommandError::TimedOut) => Ok(()),
///     r => panic!("r = {:?}", r),
/// }));
/// ```
///
/// ```
/// # use futures::Future;
/// # use gameserver::util::{TimeoutCommand, TimeoutCommandError};
/// # use std::time::Duration;
/// let cmd = TimeoutCommand::new("echo", &["foo", "bar"], Duration::from_secs(1)).unwrap();
/// tokio::run(cmd.then(|r| match r {
///     Ok(bs) => {
///         assert_eq!(bs, b"foo bar");
///         Ok(())
///     },
///     r => panic!("r = {:?}", r),
/// }));
/// ```
pub struct TimeoutCommand {
    child: Mutex<Child>,
    done: bool,
    timer: Delay,
    logd: PathBuf,
    read_stdout: bool,
    read_stderr: bool,
    stdout: ReadToEnd<ChildStdout>,
    // stderr: ReadToEnd<ChildStderr>,
}

impl TimeoutCommand {
    pub fn new<I, P1, P2, S1, S2>(
        command: S1,
        wd: P1,
        logd: P2,
        args: I,
        timeout: Duration,
    ) -> io::Result<TimeoutCommand>
    where
        I: IntoIterator<Item = S2> + Debug,
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        S1: AsRef<OsStr>,
        S2: AsRef<OsStr>,
    {
        let mut cmd = Command::new(command.as_ref());
        cmd.args(args)
            .current_dir(wd)
            .stdin(Stdio::null())
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped());
        debug!("{:?}", cmd);
        cmd.spawn_async().map(|mut child| {
            let stdout = child
                .stdout()
                .take()
                .expect("for some reason no stdout was present");
            // let stderr = child
            //     .stderr()
            //     .take()
            //     .expect("for some reason no stderr was present");
            TimeoutCommand {
                child: Mutex::new(child),
                done: false,
                read_stdout: false,
                read_stderr: false,
                logd: logd.as_ref().to_path_buf(),
                timer: sleep(timeout),
                stdout: read_to_end(stdout, Vec::new()),
                // stderr: read_to_end(stderr, Vec::new()),
            }
        })
    }
}

impl Future for TimeoutCommand {
    type Item = Vec<u8>;
    type Error = TimeoutCommandError;

    fn poll(&mut self) -> Result<Async<Vec<u8>>, TimeoutCommandError> {
        if self.done {
            match self.stdout.poll() {
                Ok(Async::Ready((_, buf))) => Ok(Async::Ready(buf)),
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(err) => Err(TimeoutCommandError::Io(err)),
            }
        // TODO: not the most ideal but works
        // if !self.read_stderr {
        //     match self.stderr.poll() {
        //         Ok(Async::Ready((_, buf))) => {
        //             File::create(self.logd.join("stderr.log"))
        //                 .and_then(|mut f| f.write_all(&buf));
        //             self.read_stderr = true;
        //             Ok(Async::NotReady)
        //         }
        //         Ok(Async::NotReady) => Ok(Async::NotReady),
        //         Err(err) => Err(TimeoutCommandError::Io(err)),
        //     }
        // } else {
        //     match self.stdout.poll() {
        //         Ok(Async::Ready((_, buf))) => {
        //             File::create(self.logd.join("stdout.log"))
        //                 .and_then(|mut f| f.write_all(&buf));
        //             Ok(Async::Ready(buf))
        //         }
        //         Ok(Async::NotReady) => Ok(Async::NotReady),
        //         Err(err) => Err(TimeoutCommandError::Io(err)),
        //     }
        // }
        } else {
            match self.timer.poll() {
                Ok(Async::Ready(())) => {
                    // We've timed out.
                    let mut child = self.child.lock().unwrap();
                    child.kill().map_err(TimeoutCommandError::Io)?;
                    Err(TimeoutCommandError::TimedOut)
                }
                Ok(Async::NotReady) => {
                    let status = self.child.lock().unwrap().poll();
                    match status {
                        Ok(Async::Ready(status)) => {
                            // We're done.
                            if status.success() {
                                self.done = true;
                                self.poll()
                            } else {
                                Err(TimeoutCommandError::Exit(status))
                            }
                        }
                        Ok(Async::NotReady) => Ok(Async::NotReady),
                        Err(err) => Err(TimeoutCommandError::Io(err)),
                    }
                }
                Err(err) => Err(TimeoutCommandError::Timer(err)),
            }
        }
    }
}

#[derive(Debug, Display)]
pub enum TimeoutCommandError {
    Exit(ExitStatus),
    Io(std::io::Error),
    TimedOut,
    Timer(tokio_timer::Error),
}

impl Error for TimeoutCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TimeoutCommandError::Exit(_) => None,
            TimeoutCommandError::Io(err) => Some(err),
            TimeoutCommandError::TimedOut => None,
            TimeoutCommandError::Timer(err) => Some(err),
        }
    }
}
