# RadianOS Filesystem Structure

| Directory   | Description                                                                                              | Typical Subdirectories / Notes                                     |
|-------------|----------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------|
| `/binary`   | Minimal global binaries or essential tools accessible early in boot or recovery.                          | Low-level utilities, no full userland implied.                    |
| `/boot`     | Bootloader configs, kernel images, and platform-specific startup files.                                  | Architecture-specific folders (e.g., `/boot/x86_64`, `/boot/aarch64`) |
| `/hw`  | Logical device namespace managed by the kernel’s driver model, replacing UNIX `/dev` with structured nodes.| e.g., `/hw/display0/`, `/hw/disk0/`, `/hw/input/`    |
| `/mount`    | Dynamic mount points for filesystems, network shares, or system services exposed as mounts.              | e.g., `/mount/fs-root/`, `/mount/net/`, `/mount/logs/`             |
| `/mutable`  | Writable runtime state (logs, caches, spool, runtime files). Equivalent to UNIX `/var` but explicit.       | `/mutable/logs/`, `/mutable/cache/`, `/mutable/spool/`, `/mutable/runtime/` |
| `/system`   | Core OS internals and base system components.                                                           | `/system/core/`, `/system/include/`, `/system/lib/`, `/system/run/`, `/system/opt/` |
| `/temp`     | Temporary scratch space cleared on reboot. Used by system and users.                                      | Analogous to UNIX `/tmp`.                                          |
| `/user`     | User-specific files, home directories, and optional userland binaries.                                   | `/user/binary/`, `/user/home/`                                     |
| `/misc`     | Miscellaneous or auxiliary data not fitting other categories.                                            | Non-critical files.                                                |
| `/opt`      | Optional packages, third-party applications, or extensions modularly added.                              | Non-core system software.                                          |

