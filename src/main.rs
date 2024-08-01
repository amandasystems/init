#![no_std]
#![no_main]

use core::arch::asm;

struct SignalMask(u64);

const STDERR_FILENO: i32 = 2;
const EXIT_FAILURE: i32 = 1;
const SYS_LINUX_GETPID: usize = 39;
const SYS_RT_SIGPROCMASK: usize = 14;
const ALL_SIGNALS: SignalMask = SignalMask(0xffffffff);
const SET_SIZE: usize = 0x8;
const NULL_PTR_AS_USIZE: usize = 0x0;

impl SignalMask {
    // From: https://stackoverflow.com/questions/34671312/sigprocmask-returns-22-in-assembly
    fn block(&self) {
        const SIG_BLOCK: usize = 0x0;
        unsafe {
            syscall4(
                SYS_RT_SIGPROCMASK,
                SIG_BLOCK,
                (&self.0 as *const u64) as usize,
                NULL_PTR_AS_USIZE,
                SET_SIZE,
            );
        }
    }
    fn unblock(&self) {
        const SIG_UNBLOCK: usize = 0x1;
        unsafe {
            syscall4(
                SYS_RT_SIGPROCMASK,
                SIG_UNBLOCK,
                (&self.0 as *const u64) as usize,
                NULL_PTR_AS_USIZE,
                SET_SIZE,
            );
        }
    }
}

struct Signals;

impl Signals {
    fn block_all() -> SignalBlocker {
        ALL_SIGNALS.block();
        SignalBlocker
    }
}

struct SignalBlocker;

impl Drop for SignalBlocker {
    fn drop(&mut self) {
        print("\nCleanup: unblocking all signals\n");
        ALL_SIGNALS.unblock();
    }
}

// From the linux-syscall crate
pub unsafe fn syscall4(sysno: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let ret;
    asm!(
        "syscall",
        inlateout("rax") sysno => ret,
        in("rdi") arg0,
        in("rsi") arg1,
        in("rdx") arg2,
        in("r10") arg3,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack, preserves_flags, readonly)
    );
    assert!(ret == 0, "Error running syscall!");
    ret
}

// CONFIRMED WORKING!!!
fn my_pid() -> usize {
    let ret;
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") SYS_LINUX_GETPID => ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack, preserves_flags, readonly)
        );
    }
    ret
}

fn wait(status: &mut i32) {
    // FIXME: implement this!
}

enum ForkResult {
    Parent,
    Child(i32),
}

fn fork() -> ForkResult {
    // FIXME: implement the fork syscall!
    ForkResult::Parent
}

fn reap_processes() -> ! {
    let mut status = 0;
    loop {
        wait(&mut status);
    }
}

fn spawn_thread(f: fn() -> !) {
    match fork() {
        ForkResult::Parent => (),
        ForkResult::Child(_) => f(),
    }
}

fn new_process_group() {
    // FIXME: setsid()
    // FIXME: setpgid(0, 0), where the first zero means for this process
}

fn print(message: &str) {
    write(STDERR_FILENO, message.as_ptr() as *const _, message.len());
}

fn exit(status: i32) -> ! {
    unsafe {
        asm!(
            "syscall",
            in("rax") 60,
            in("rdi") status,
            options(noreturn)
        );
    }
}

fn write(fd: i32, buf: *const u8, count: usize) -> isize {
    unsafe {
        let r0;
        asm!(
            "syscall",
            inlateout("rax") 1isize => r0,
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") count,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack, preserves_flags)
        );
        r0
    }
}

// The name `_start` is magic; Linux uses it as the initial
// label of execution. I figured I might as well use that one.
// See so: https://stackoverflow.com/a/67919410.
// Other names would require linker directives etc etc, and that's
// just a bother.
#[no_mangle]
#[inline(never)]
pub extern "C" fn _start(_stack_top: *const u8) -> ! {
    if my_pid() != 1 {
        print("E: Not PID 1; exiting!\n");
        exit(EXIT_FAILURE);
    }

    {
        // This needs a name or we get an early drop!?
        let _signal_guard = Signals::block_all();
        spawn_thread(reap_processes);
    }
    new_process_group();
    print("This is where I would run my init script, if I had one!\n");
    exit(EXIT_FAILURE);
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
