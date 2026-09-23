// SPDX-License-Identifier: RPL-1.5
//! A child pins its process-group identity until every member has been signalled.
//! Peek with WNOWAIT: reaping first would permit PID/PGID reuse before killpg.
use nix::{
    sys::{
        signal::{Signal, killpg},
        wait::{Id, WaitPidFlag, WaitStatus, waitid},
    },
    unistd::Pid,
};
use std::io;

pub(crate) fn exited(pid: u32) -> io::Result<bool> {
    match waitid(
        Id::Pid(Pid::from_raw(pid as i32)),
        WaitPidFlag::WEXITED | WaitPidFlag::WNOHANG | WaitPidFlag::WNOWAIT,
    ) {
        Ok(WaitStatus::StillAlive) => Ok(false),
        Ok(_) => Ok(true),
        Err(error) => Err(error.into()),
    }
}
pub(crate) fn terminate(pid: u32) {
    let _ = killpg(Pid::from_raw(pid as i32), Signal::SIGKILL);
}
