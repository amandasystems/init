#![no_std]
#![no_main]

use core::arch::asm;

// From the libc crate
#[repr(C)]
struct SigSet {
    #[cfg(target_pointer_width = "32")]
    set: [u32; 32],
    #[cfg(target_pointer_width = "64")]
    set: [u64; 16],
}

impl SigSet {
    fn all_signals() -> Self {
        // FIXME: this breaks for 32-bit architectures!
        let mut set = [0; 16];
        set[0] = 0xfffffffc7fffffff;
        set[1] = 0xffffffffffffffff;
        SigSet { set }
        // https://git.musl-libc.org/cgit/musl/tree/src/signal/sigfillset.c
        // I think this means we should have 0xfffffffffffffffful, in whatever is the best order.
        // Do whatever is the default here:
        // let mut set: sigset_t = MaybeUninit::zeroed().assume_init();
        // libc::sigfillset(&mut set);
    }

    // SIG_BLOCK is 0x0, SIG_UNBLOCK is 0x01
    // unsafe { libc::sigprocmask(0x0, &self.0, ptr::null_mut()) };
    fn block(&self) {}

    fn unblock(&self) {}
}

const STDERR_FILENO: i32 = 2;
const EXIT_FAILURE: i32 = 1;
const SYS_LINUX_GETPID: usize = 39;

struct Signals;

impl Signals {
    fn block_all() -> SignalBlocker {
        let set = SigSet::all_signals();
        set.block();
        SignalBlocker(set)
    }
}

struct SignalBlocker(SigSet);

impl Drop for SignalBlocker {
    fn drop(&mut self) {
        self.0.unblock();
    }
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
pub fn _start(_stack_top: *const u8) -> ! {
    if my_pid() != 1 {
        print("E: Not PID 1; exiting!\n\0");
        exit(EXIT_FAILURE);
    }

    {
        let _ = Signals::block_all();
        spawn_thread(reap_processes);
    }
    new_process_group();
    print("This is where I would run my init script, if I had one!\0");
    exit(EXIT_FAILURE);
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
