//! A singlethreaded QEMU fuzzer that can auto-restart.

use clap::{App, Arg};
use core::time::Duration;
use std::{
    env,
    fs::{self},
    path::PathBuf,
};

use libafl::Error;

use libafl_qemu::{amd64::Amd64Regs, QemuEmulator};

/// The fuzzer main (as `no_mangle` C function)
#[no_mangle]
pub fn libafl_qemu_main() {
    // Registry the metadata types used in this fuzzer
    // Needed only on no_std
    //RegistryBuilder::register::<Tokens>();

    let mut args = vec!["libafl_qemu_fuzzbench".into()];
    let mut args_iter = std::env::args();
    while let Some(arg) = args_iter.next() {
        if arg.starts_with("--libafl") {
            args.push(arg);
            args.push(args_iter.next().unwrap());
        } else if arg.starts_with("-libafl") {
            args.push("-".to_owned() + &arg);
            args.push(args_iter.next().unwrap());
        }
    }

    let res = match App::new("libafl_qemu_fuzzbench")
        .version("0.4.0")
        .author("AFLplusplus team")
        .about("LibAFL-based fuzzer with QEMU for Fuzzbench")
        .arg(
            Arg::new("out")
                .about("The directory to place finds in ('corpus')")
                .long("libafl-out")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("in")
                .about("The directory to read initial inputs from ('seeds')")
                .long("libafl-in")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("tokens")
                .long("libafl-tokens")
                .about("A file to read tokens from, to be used during fuzzing")
                .takes_value(true),
        )
        .arg(
            Arg::new("logfile")
                .long("libafl-logfile")
                .about("Duplicates all output to this file")
                .default_value("libafl.log"),
        )
        .arg(
            Arg::new("timeout")
                .long("libafl-timeout")
                .about("Timeout for each individual execution, in milliseconds")
                .default_value("1000"),
        )
        .try_get_matches_from(args)
    {
        Ok(res) => res,
        Err(err) => {
            println!(
                "Syntax: {}, --libafl-in <input> --libafl-out <output>\n{:?}",
                env::current_exe()
                    .unwrap_or_else(|_| "fuzzer".into())
                    .to_string_lossy(),
                err.info,
            );
            return;
        }
    };

    println!(
        "Workdir: {:?}",
        env::current_dir().unwrap().to_string_lossy().to_string()
    );

    // For fuzzbench, crashes and finds are inside the same `corpus` directory, in the "queue" and "crashes" subdir.
    let mut out_dir = PathBuf::from(res.value_of("out").unwrap().to_string());
    if fs::create_dir(&out_dir).is_err() {
        println!("Out dir at {:?} already exists.", &out_dir);
        if !out_dir.is_dir() {
            println!("Out dir at {:?} is not a valid directory!", &out_dir);
            return;
        }
    }
    let mut crashes = out_dir.clone();
    crashes.push("crashes");
    out_dir.push("queue");

    let in_dir = PathBuf::from(res.value_of("in").unwrap().to_string());
    if !in_dir.is_dir() {
        println!("In dir at {:?} is not a valid directory!", &in_dir);
        return;
    }

    let tokens = res.value_of("tokens").map(PathBuf::from);

    let logfile = PathBuf::from(res.value_of("logfile").unwrap().to_string());

    let timeout = Duration::from_millis(
        res.value_of("timeout")
            .unwrap()
            .to_string()
            .parse()
            .expect("Could not parse timeout in milliseconds"),
    );

    fuzz(out_dir, crashes, in_dir, tokens, logfile, timeout)
        .expect("An error occurred while fuzzing");
}

/// The actual fuzzer
fn fuzz(
    _corpus_dir: PathBuf,
    _objective_dir: PathBuf,
    _seed_dir: PathBuf,
    _tokenfile: Option<PathBuf>,
    _logfile: PathBuf,
    _timeout: Duration,
) -> Result<(), Error> {
    let mut emu = QemuEmulator::new();

    emu.set_breakpoint(0x00401176); // LLVMFuzzerTestOneInput

    println!("Run");

    emu.run();

    println!(
        "Break at {:#x}",
        emu.read_reg::<_, usize>(Amd64Regs::Rip).unwrap()
    );

    emu.write_reg(Amd64Regs::Rsi, 6i32).unwrap();
    emu.run();

    // Never reached
    Ok(())
}
