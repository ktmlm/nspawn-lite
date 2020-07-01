# NSpawn-Lite

A minimal `systemd-nspawn` implement.

- 用于适配 CentOS 6.x 等内核版本 >=2.6.34, 但不使用 Systemd 的平台
- 用于适配 CentOS 8.x 及 Ubuntu 16.04/18.04/20.04 等不预装 `systemd-nspawn` 的平台
  - 此类平台, 发行方将 `systemd-nspawn` 功能拆分为独立的安装包 `systemd-container`

```shell
nspawn-lite 0.1
FanHui. <hui.fan@mail.ru>
A mininal container engine.

USAGE:
    nspawn-lite [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --cmd-path <PATH>     The command path **after** chroot.
    -r, --root-path <PATH>    The new rootfs path **before** chroot.
```
