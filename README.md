This is a minimal (ish) init system written in Rust, though it looks a lot like C. It's based on [this tiny init](http://ewontfix.com/14/), and only relies on the libc crate.

It does two things:
1. Spawn a thread that reaps processes.
2. Presumably run a RC script.

## Using it
To run it you need a kernel willing to load it as its init. Here is one way to do that using Qemu, borrowed from [Mustafa Akin](https://medium.com/@mustafaakin/writing-my-own-init-with-go-part-1-22e81495a246):


Set up cross compiling:
```
$ rustup target add x86_64-unknown-linux-gnu
```
