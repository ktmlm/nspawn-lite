use clap::App;
use myutil::{err::*, *};
use nix::{
    mount::{self, umount2, MntFlags, MsFlags},
    sched::{clone, CloneFlags},
    unistd::{chdir, execv, pivot_root},
};
use std::{ffi::CString, process};

macro_rules! err {
    () => {{
        eprintln!("\n\x1b[31;01mInvalid arguments, please run `nspawn-lite --help`.\x1b[00m");
        process::exit(1);
    }};
}

fn main() {
    let matches = App::new("nspawn-lite")
        .version("0.1")
        .author("FanHui. <hui.fan@mail.ru>")
        .about("A mininal container engine.")
        .args_from_usage(
            "-r --root-path=[PATH] 'The new rootfs path **before** chroot.'",
        )
        .args_from_usage(
            "-c --cmd-path=[PATH] 'The command path **after** chroot.'",
        )
        .args_from_usage(
            "-n --exec-name=[NAME] 'The name of 'inner systemd' process, gotten by `ps` command.'",
        )
        .get_matches();

    match (
        matches.value_of("root-path"),
        matches.value_of("cmd-path"),
        matches.value_of("exec-name"),
    ) {
        (Some(rp), Some(cp), exec_name) => {
            pnk!(run(rp, cp, exec_name.unwrap_or("systemd-embed")));
        }
        _ => {
            err!();
        }
    }
}

// Return the PID of the-inner-systemd
fn run(root_path: &str, cmd_path: &str, exec_name: &str) -> Result<i32> {
    // 临时栈空间, 执行`execv`后就会被丢弃
    const STACK_SIZ: usize = 1024 * 1024;

    let mut stack = Vec::with_capacity(STACK_SIZ);
    unsafe {
        stack.set_len(STACK_SIZ);
    }

    let mut flags = CloneFlags::empty();
    flags.insert(CloneFlags::CLONE_NEWNS);
    flags.insert(CloneFlags::CLONE_NEWPID);
    flags.insert(CloneFlags::CLONE_NEWUTS);
    flags.insert(CloneFlags::CLONE_NEWIPC);

    // Create a minimal container
    let ops = || -> isize {
        pnk!(mount_make_rprivate()
            .c(d!())
            .and_then(|_| do_pivot_root(root_path).c(d!()))
            .and_then(|_| mount_dynfs_proc().c(d!()))
            .and_then(|_| start_systemd(cmd_path, exec_name)));
        0
    };

    Ok(clone(
        Box::new(ops),
        stack.as_mut_slice(),
        flags,
        Some(libc::SIGCHLD),
    )
    .c(d!())?
    .as_raw())
}

// SEE: `man 2 pivot_root`
fn mount_make_rprivate() -> Result<()> {
    mountx(
        None,
        "/",
        None,
        pnk!(MsFlags::from_bits(
            MsFlags::MS_REC.bits() | MsFlags::MS_PRIVATE.bits()
        )),
        None,
    )
    .c(d!())
}

// As the PID-1 process
fn start_systemd(cmd_path: &str, exec_name: &str) -> Result<()> {
    execv(
        &CString::new(cmd_path).unwrap(),
        &[
            &CString::new(exec_name).unwrap(),
            &CString::new("--system").unwrap(),
        ],
    )
    .map(|_| ())
    .c(d!())
}

// SEE: `man 2 pivot_root`
fn do_pivot_root(root_path: &str) -> Result<()> {
    mountx(Some(root_path), root_path, None, MsFlags::MS_BIND, None)
        .c(d!())
        .and_then(|_| pivot_root(root_path, root_path).c(d!()))
        .and_then(|_| umount2("/", MntFlags::MNT_DETACH).c(d!()))
        .and_then(|_| chdir("/").c(d!()))
}

fn mount_dynfs_proc() -> Result<()> {
    let mut flags = MsFlags::empty();
    flags.insert(MsFlags::MS_NODEV);
    flags.insert(MsFlags::MS_NOEXEC);
    flags.insert(MsFlags::MS_NOSUID);
    flags.insert(MsFlags::MS_RELATIME);

    mountx(Some("proc"), "/proc", Some("proc"), flags, None).c(d!())
}

#[inline(always)]
fn mountx(
    from: Option<&str>,
    to: &str,
    fstype: Option<&str>,
    flags: MsFlags,
    data: Option<&str>,
) -> Result<()> {
    mount::mount(from, to, fstype, flags, data).c(d!())
}
