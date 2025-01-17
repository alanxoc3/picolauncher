use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::anyhow;
use embedded_hal::i2c::{I2c, Operation as I2cOperation};
use event::CreateKind;
use linux_embedded_hal::I2cdev;
use log::{debug, error, info, warn};
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use notify_debouncer_full::{new_debouncer, notify::*, DebounceEventResult};
use tokio::process::{Child, Command};

use crate::{
    consts::*,
    p8util::{format_label, screenshot2cart},
};

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

pub fn kill_pico8_process(pico8_process: &Child) -> anyhow::Result<()> {
    let pico8_pid = Pid::from_raw(
        pico8_process
            .id()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "PID not found"))?
            as i32,
    );

    kill(pico8_pid, Signal::SIGKILL)?;
    Ok(())
}

pub fn stop_pico8_process(pico8_process: &Child) -> anyhow::Result<()> {
    let pico8_pid = Pid::from_raw(
        pico8_process
            .id()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "PID not found"))?
            as i32,
    );
    kill(pico8_pid, Signal::SIGSTOP)?;
    Ok(())
}

pub fn resume_pico8_process(pico8_process: &Child) -> anyhow::Result<()> {
    let pico8_pid = Pid::from_raw(
        pico8_process
            .id()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "PID not found"))?
            as i32,
    );
    kill(pico8_pid, Signal::SIGCONT)?;
    Ok(())
}

// Watch screenshot directory for new screenshots and then convert to a cartridge + downscale
pub fn screenshot_watcher() {
    let mut debouncer = new_debouncer(Duration::from_secs(2), None, |res: DebounceEventResult| {
        match res {
            Ok(events) => {
                for event in events.iter() {
                    if event.event.kind == EventKind::Create(CreateKind::File) {
                        debug!("{event:?}");

                        // TODO should do this for each path?
                        let screenshot_fullpath = event.event.paths.get(0).unwrap();

                        // TODO don't use unwrap
                        let cart_name = screenshot_fullpath.file_stem().unwrap().to_string_lossy();

                        // preprocess newly created screenshot and downscale to png
                        let mut cart_128 = screenshot2cart(screenshot_fullpath).unwrap();
                        let cart_32 = format_label(&mut cart_128, 32).unwrap();

                        let mut out_128_file = File::create(PathBuf::from(format!(
                            "{SCREENSHOT_PATH}/{cart_name}.128.p8"
                        )))
                        .unwrap();
                        let mut out_32_file = File::create(PathBuf::from(format!(
                            "{SCREENSHOT_PATH}/{cart_name}.32.p8"
                        )))
                        .unwrap();

                        cart_128.write(&mut out_128_file);
                        cart_32.write(&mut out_32_file);
                    }
                }
            },
            Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
        }
    })
    .unwrap();

    debouncer
        .watcher()
        .watch(Path::new("drive/screenshots"), RecursiveMode::Recursive)
        .unwrap();

    info!("screenshot watcher registered");

    loop {} // TODO this might consume a lot of cpu?
}

/// Suspend the pico8 process until child process exits
pub async fn pico8_to_bg(pico8_process: &Child, mut child: Child) {
    // suspend current pico8 process and swap with newly spawned process
    stop_pico8_process(pico8_process);

    // unsuspend when child finishes
    child.wait().await.unwrap();
    resume_pico8_process(pico8_process);
}

/// Attempts to spawn pico8 binary by trying multiple potential binary names depending on the
/// platform
pub fn launch_pico8_binary(bin_names: &Vec<String>, args: Vec<&str>) -> anyhow::Result<Child> {
    for bin_name in bin_names {
        let pico8_process = Command::new(bin_name.clone())
            .args(args.clone())
            // .stdout(Stdio::piped())
            .spawn();

        match pico8_process {
            Ok(process) => return Ok(process),
            Err(e) => warn!("failed launching {bin_name}: {e}"),
        }
    }
    Err(anyhow!("failed to launch pico8"))
}

/// Use the pico8 binary to export games from *.p8.png to *.p8
pub async fn pico8_export(
    bin_names: &Vec<String>,
    in_file: &Path,
    out_file: &Path,
) -> anyhow::Result<()> {
    let mut pico8_process = launch_pico8_binary(
        bin_names,
        vec![
            "-x",
            in_file.to_str().unwrap(),
            "-export",
            out_file.to_str().unwrap(),
        ],
    )?;
    pico8_process.wait().await?;
    Ok(())
}

const CTRL_REG1_GYRO: u8 = 0x10; // Gyroscope control register
const OUT_X_L_GYRO: u8 = 0x18; // Gyroscope output X low register
const CTRL_REG6_ACCEL: u8 = 0x20; // Accelerometer control register
const OUT_X_L_ACCEL: u8 = 0x28; // Accelerometer X low byte

const WHO_AM_I: u8 = 0x0F; // WHO_AM_I register
const WHO_AM_I_RESP: u8 = 0x68; // the value we expect to get back from whoami request
const GYRO_SCALE: f64 = 0.07; // sensitivity of gyroscope
const ACCEL_SCALE: f64 = 0.000061; // sensitivity of accel

pub struct LSM9DS1 {
    dev: I2cdev,
    accel_gyro_addr: u8,
    calib: (f32, f32, f32),
}

impl LSM9DS1 {
    /// sdom_high: if the SDOM pin is pulled high or low
    /// sdoag_high: if the SDOAG pin is pulled high or low
    pub fn new(i2cdev: &str, sdoag_high: bool) -> anyhow::Result<Self> {
        let accel_gyro_addr = if sdoag_high { 0x6B } else { 0x6A };

        // create i2c device
        let mut dev = I2cdev::new(i2cdev)?;

        // run a heartbeat command to see if the device is connected
        // for gyro
        let mut buf = [0];
        dev.write_read(accel_gyro_addr, &[WHO_AM_I], &mut buf)?;
        if buf[0] != WHO_AM_I_RESP {
            return Err(anyhow::anyhow!(format!(
                "unexpected gyroscope address, expected: {}, got: {}",
                accel_gyro_addr, buf[0]
            )));
        }

        // enable the gyro
        dev.write(accel_gyro_addr, &[CTRL_REG1_GYRO, 0x60])?;

        // enable the accel
        dev.write(accel_gyro_addr, &[CTRL_REG6_ACCEL, 0x60])?;

        Ok(LSM9DS1 {
            dev,
            accel_gyro_addr,
            calib: (0., 0., 0.),
        })
    }

    // calibrate the gyro by taking sample values
    // TODO maybe don't need this if we implement complimentary filter
    pub fn calibrate(&mut self) -> Result<()> {
        const CALIB_STEPS: u32 = 1000;

        let mut calib = (0., 0., 0.);
        for _ in 0..CALIB_STEPS {
            // TODO technically can skip and allow dropping a couple of calib steps
            let data = self.read_gyro()?;
            calib += data;
        }
        calib /= CALIB_STEPS;

        Ok(())
    }

    // spawn a task that periodically queries the gyro and updates internal tilt
    // TODO

    fn read_register(&mut self, addr: u8, len: usize) -> anyhow::Result<Vec<u8>> {
        let mut buf = vec![0; len];
        self.dev
            .write_read(self.gyro_addr, &[addr | 0x80], &mut buf)?; // 0x80 for auto-increment
        Ok(buf)
    }

    pub fn read_gyro(&mut self) -> anyhow::Result<(f64, f64, f64)> {
        let data = self.read_register(OUT_X_L_GYRO, 6)?;

        let x = i16::from_le_bytes([data[0], data[1]]) as f64 * GYRO_SCALE;
        let y = i16::from_le_bytes([data[2], data[3]]) as f64 * GYRO_SCALE;
        let z = i16::from_le_bytes([data[4], data[5]]) as f64 * GYRO_SCALE;

        Ok((x, y, z))
    }

    pub fn read_accel(&mut self) -> anyhow::Result<(f64, f64, f64)> {
        let data = self.read_register(OUT_X_L_ACCEL, 6)?;

        let x = i16::from_le_bytes([data[0], data[1]]) as f64 * ACCEL_SCALE;
        let y = i16::from_le_bytes([data[2], data[3]]) as f64 * ACCEL_SCALE;
        let z = i16::from_le_bytes([data[4], data[5]]) as f64 * ACCEL_SCALE;

        Ok((x, y, z))
    }
}
