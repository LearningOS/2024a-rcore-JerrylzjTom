//! Process management syscalls

use crate::mm::{translated_ptr_across_pages};
use crate::task::{current_user_token, get_current_task_info, mmap, munmap};
use crate::timer::{get_time_ms, get_time_us};
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,
    },
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let ts = translated_ptr_across_pages(current_user_token(), _ts);
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    let ti = translated_ptr_across_pages(current_user_token(), _ti);
    let current_time = get_time_ms();
    let (status, syscall_time, time) = get_current_task_info();
    unsafe {
        (*ti).syscall_times = syscall_time;
        (*ti).status = status;
        (*ti).time = current_time - time;
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _prot: usize) -> isize {
    // 检查 prot 是否仅包含低 3 位（读、写、执行权限），其他位必须为 0
    if _prot & !0x7 != 0 {
        return -1; // 其他位不为0，返回错误
    }
    // 检查是否有至少一个有效的权限位（读、写或执行）
    if _prot & 0x7 == 0 {
        return -1; // 没有有效权限，返回错误
    }
    if _start % 4096 != 0 {
        return -1;
    }
    mmap(_start.into(), (_start + _len).into(), _prot.into())
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    if _start % 4096 != 0 || _len % 4096 != 0 {
        return -1;
    }
    munmap(_start.into(), (_start + _len).into())
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
