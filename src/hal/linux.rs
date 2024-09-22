use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf}, process::Child,
};

use nix::{sys::signal::{kill, Signal}, unistd::Pid};

pub const IN_PIPE: &'static str = "in_pipe";
pub const OUT_PIPE: &'static str = "out_pipe";

/// create named pipes if they don't exist
fn create_pipe(pipe: &Path) -> anyhow::Result<()> {
    use nix::{
        sys::{
            signal::{kill, Signal},
            stat::Mode,
        },
        unistd::{mkfifo, Pid},
    };
    if !pipe.exists() {
        mkfifo(pipe, Mode::S_IRUSR | Mode::S_IWUSR).expect("failed to create pipe {pipe}");
    }
    Ok(())
}

pub fn open_in_pipe() -> anyhow::Result<File> {
    create_pipe(&PathBuf::from(IN_PIPE))?;

    let in_pipe = OpenOptions::new().write(true).open(&*IN_PIPE)?;

    Ok(in_pipe)
}

pub fn open_out_pipe() -> anyhow::Result<File> {
    create_pipe(&PathBuf::from(IN_PIPE))?;

    let out_pipe = OpenOptions::new().read(true).open(&*OUT_PIPE)?;

    Ok(out_pipe)
}

pub fn stop_pico8_process(pico8_process: &Child) -> anyhow::Result<()> {
    let pico8_pid = Pid::from_raw(pico8_process.id() as i32);
    kill(pico8_pid, Signal::SIGSTOP)?;
    Ok(())
}
