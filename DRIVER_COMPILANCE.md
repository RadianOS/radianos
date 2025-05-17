This document outlines how drivers must be structured, loaded, and interact with the Radian kernel or message-based core.

Radian is **not POSIX**, **not UNIX**, and does **not follow traditional Linux driver models**. This spec provides a clean and modular abstraction layer for devices.

---

## High-Level Design Goals

- **Modular**: Drivers should be loadable/unloadable at runtime.
- **Portable**: Target abstract interfaces, not specific hardware assumptions.
- **Safe Interfaces**: Prefer message-passing or structured APIs.
- **Declarative**: Drivers include metadata to describe capabilities.
- **Userland or Kernel-space**: Drivers can live in either context.

---

## Driver Location

Drivers **must** be exposed under:

```
/devices/
```

Each device gets a subdirectory or node:

```
/devices/
├── pci/
│   ├── 0000:00:01.0/
│   │   └── net.intel.e1000.drv
├── input/
│   └── keyboard.drv
├── display/
│   └── vga.drv
└── fs/
    └── objfs.drv
```

---

## Required Files for Each Driver

| File                   | Description |
|------------------------|-------------|
| `driver.drv`           | Loadable driver binary |
| `manifest.yml`         | Metadata file (device info, entrypoint, API version) |
| `init` (optional)      | Init script or binary (for userland setup) |
| `events/`              | Directory for event/message files |
| `state/`               | Virtual FS for control or status (optional) |

---

## manifest.yml Example

```yaml
name: "intel-e1000"
version: "1.0.0"
vendor: "Intel"
device_class: "network"
pci_id: "0x8086:0x100E"
entrypoint: "driver.drv"
interface: "v1"
mode: "kernel" # or "user"
provides:
  - net.if
  - dev.bus.pci
depends:
  - core.io
  - system.memory
```

---

## Initialization Requirements

### Kernel-Mode ABI
```c
int drv_init(struct drv_context* ctx);
int drv_attach(struct device* dev);
int drv_interrupt(struct device* dev);
void drv_shutdown(void);
```

### Userland Driver Messaging

Userland drivers expose an IPC interface:

- Channel: `/system/run/drivers/<name>`
- Must respond to: `INIT`, `ATTACH`, `IRQ`, `SHUTDOWN`

#### Example message:

```json
{
  "cmd": "INIT",
  "args": {
    "device_id": "0x100E"
  }
}
```

#### Response:

```json
{
  "status": "ok",
  "features": ["net.if", "dma", "irq"]
}
```

---

## Safety and Isolation

- **Userland drivers preferred** for crash safety.
- Kernel-mode must be sandboxed (in the future: capability-based model).
- Logging to `/mutable/log/` is encouraged.

---

## Compliance Checklist

| Requirement                     | Mandatory |
|--------------------------------|-----------|
| Expose manifest.yml            | YES        |
| Register in `/devices/`        | YES        |
| Implement INIT/ATTACH APIs     | YES        |
| Specify API version            | YES        |
| Use portable interface         | YES        |
| Userland message API | OPT-OUT        |
| Log and recover from errors    | YES        |

---

## Planned Extensions

- Driver signing and validation
- Dynamic bus scanning tools
- Hotplug notification system
- Blacklist/override framework
- Performance introspection

---

## See Also

- [radian.yml](../radian.yml) for OS-level metadata
- `oskit` for bootstrapping drivers
