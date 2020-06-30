use myutil::{err::*, *};
use nix::{
    mount::{self, umount2, MntFlags, MsFlags},
    sched::{clone, unshare, CloneFlags},
    unistd::{chdir, pivot_root},
};
use std::process::Command;

fn main() {
    // TODO clap args
    pnk!(run("/data/baseimage", "/lib/systemd/systemd"));
}

fn run(root_path: &str, cmd_path: &str) -> Result<i32> {
    let ops = || -> isize {
        let res = mount_make_rprivate()
            .c(d!())
            .and_then(|_| do_pivot_root(root_path).c(d!()))
            .and_then(|_| mount_dynfs_proc().c(d!()))
            .and_then(|_| unshare(CloneFlags::CLONE_NEWUSER).c(d!()))
            .and_then(|_| start_systemd(cmd_path));
        pnk!(res);
        0
    };

    let mut stack = vec![0; 1024 * 1024];
    let mut flags = CloneFlags::empty();
    flags.insert(CloneFlags::CLONE_NEWNS);
    flags.insert(CloneFlags::CLONE_NEWPID);

    let pid = clone(
        Box::new(ops),
        stack.as_mut_slice(),
        flags,
        Some(libc::SIGCHLD),
    )
    .c(d!())?
    .as_raw();

    Ok(pid)
}

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

fn start_systemd(cmd_path: &str) -> Result<()> {
    Command::new(cmd_path)
        .status()
        .c(d!())
        .and_then(|s| if s.success() { Ok(()) } else { Err(eg!()) })
}

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
