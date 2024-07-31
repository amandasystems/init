
.PHONY: disk.img
disk.img:
	cargo build --target x86_64-unknown-none
	mkdir sysroot
	cp target/x86_64-unknown-none/debug/init sysroot
	sudo virt-make-fs --format=raw --type=ext2 sysroot disk.img
	${RM} -r sysroot

clean:
	cargo clean
	${RM} disk.img

.PHONY: boot
boot: disk.img
	qemu-system-x86_64 -kernel vmlinuz \
	-drive format=raw,file=disk.img \
	-nographic \
	-append "console=ttyS0 root=/dev/sda init=/init selinux=0"