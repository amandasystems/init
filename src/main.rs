#![no_std]
#![no_main]

use core::{mem::MaybeUninit, ptr};

extern crate libc;

struct Signals;

impl Signals {
    fn block_all() -> SignalBlocker {
        let set = unsafe {
            let mut set: libc::sigset_t = MaybeUninit::zeroed().assume_init();

            libc::sigfillset(&mut set);
            libc::sigprocmask(libc::SIG_BLOCK, &set, ptr::null_mut());
            set
        };
        SignalBlocker(set)
    }
}

struct SignalBlocker(libc::sigset_t);

impl Drop for SignalBlocker {
    fn drop(&mut self) {
        unsafe { libc::sigprocmask(libc::SIG_UNBLOCK, &self.0, ptr::null_mut()) };
    }
}

fn print(message: &str) {
    unsafe {
        libc::printf(message.as_ptr() as *const _);
    }
}

fn my_pid() -> i32 {
    unsafe { libc::getpid() }
}

fn reap_processes() -> ! {
    let mut status = 0 as libc::c_int;
    loop {
        unsafe {
            libc::wait(&mut status);
        }
    }
}

fn spawn_thread(f: fn() -> !) {
    const PID_IF_PARENT: i32 = 0;

    let pid_or_zero = unsafe { libc::fork() };
    if pid_or_zero == PID_IF_PARENT {
        return;
    } else {
        f()
    }
}

fn new_process_group() {
    const FOR_CURRENT_PROCESS: i32 = 0;
    unsafe {
        libc::setsid();
        libc::setpgid(FOR_CURRENT_PROCESS, 0);
    }
}

#[no_mangle]
pub extern "C" fn main() -> libc::c_int {
    if my_pid() != 1 {
        print("E: Not PID 1; exiting!\n");
        return libc::EXIT_FAILURE;
    }

    {
        let _ = Signals::block_all();
        spawn_thread(reap_processes);
    }
    new_process_group();
    print("This is where I would run my init script, if I had one!\n");
    return libc::EXIT_SUCCESS;
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
