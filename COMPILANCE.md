# Compliance Specification

**RadianOS** is not a traditional OS or a Linux distribution. It is a modular, architecture-agnostic operating system foundation and toolkit. This document outlines the requirements and design rules for a system to be considered **Radian-compliant**.

---

## What Makes a System "Radian"

A **Radian system** is an OS that conforms to a shared API, structure, and philosophy—allowing different systems to interoperate, share drivers, and be built using the `oskit` toolkit.

Radian promotes flexibility in implementation, while enforcing a consistent **external contract**.

---

## Compliance Rules

### 1. Required Directory Structure

At runtime, the system **must expose or mount** the following directories with their intended semantics. These paths may be virtual, overlaid, or minimal—but they must be accessible and consistent.

| Directory   | Description                                                                                              | Typical Subdirectories / Notes                                     |
|-------------|----------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------|
| `/binary`   | Minimal global binaries or essential tools accessible early in boot or recovery.                          | Low-level utilities, no full userland implied.                    |
| `/boot`     | Bootloader configs, kernel images, and platform-specific startup files.                                  | Architecture-specific folders (e.g., `/boot/x86_64`, `/boot/aarch64`) |
| `/devices`  | Logical device namespace managed by the kernel’s driver model, replacing UNIX `/dev` with structured nodes.| e.g., `/devices/display0/`, `/devices/disk0/`, `/devices/input/`    |
| `/mount`    | Dynamic mount points for filesystems, network shares, or system services exposed as mounts.              | e.g., `/mount/fs-root/`, `/mount/net/`, `/mount/logs/`             |
| `/mutable`  | Writable runtime state (logs, caches, spool, runtime files). Equivalent to UNIX `/var` but explicit.       | `/mutable/logs/`, `/mutable/cache/`, `/mutable/spool/`, `/mutable/runtime/` |
| `/system`   | Core OS internals and base system components.                                                           | `/system/core/`, `/system/include/`, `/system/lib/`, `/system/run/`, `/system/opt/` |
| `/temp`     | Temporary scratch space cleared on reboot. Used by system and users.                                      | Analogous to UNIX `/tmp`.                                          |
| `/user`     | User-specific files, home directories, and optional userland binaries.                                   | `/user/binary/`, `/user/home/`                                     |
| `/misc`     | Miscellaneous or auxiliary data not fitting other categories.                                            | Non-critical files.                                                |
| `/opt`      | Optional packages, third-party applications, or extensions modularly added.                              | Non-core system software.                                          |

---

### 2. Radian API Compliance

Your system must expose the **Radian API (v1+)**, which defines:

- System calls or message-based primitives for:
  - Processes, memory, and scheduling
  - Device access and I/O
  - Filesystem and path semantics
- Driver interface and registration (`os.drv`)
- Optional services via `/system/run/` or IPC mechanisms

The actual kernel ABI or method (syscalls, handles, messages) is flexible, but must behave according to the API contract.

---

### 3. Driver Model

Drivers must implement the **unified Radian driver model**, and:

- Be loadable at boot or runtime (kernel or userland)
- Appear under `/devices/`
- Respond to standardized init/attach/interrupt APIs

---

### 4. Buildable with `oskit` or Manifest Metadata

Your system must:

- Be buildable or bootstrappable using the `oskit` toolkit  
- Or define a `radian.yml` manifest with:
  - Kernel entrypoint
  - Supported API version
  - Directory layout
  - Features and optional extensions

---

## Example `radian.yml`

```yaml
name: "NovaOS"
version: "0.3"
radian_version: "1.0"
kernel_entry: "system/core/kernel/main.c"
api_version: "1.0"
structure:
  - /system/core
  - /user/home
  - /devices
features:
  driver_model: unified
  ipc: message-based
  oskit_bootstrap: true
custom:
  scheduler: "modular-rr"
  fs: "objfs"
```
