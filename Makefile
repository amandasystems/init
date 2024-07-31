myinit: hello-world.c
	gcc -static -O0 hello-world.c -o myinit

disk.img: myinit
	mkdir sysroot
	cp myinit sysroot
	sudo virt-make-fs --format=raw --type=ext2 sysroot disk.img
	${RM} -r sysroot

clean:
	${RM} myinit disk.img

.PHONY: boot
boot: disk.img
	qemu-system-x86_64 -kernel vmlinuz \
	-drive format=raw,file=disk.img \
	-nographic \
	-append "console=ttyS0 root=/dev/sda init=/myinit selinux=0"