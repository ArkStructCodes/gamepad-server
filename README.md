<div align="center">
  <h1>🪈 PadPipe</h1>
  <h4>Pipes inputs over the network into a virtual gamepad device on linux.</h4>
  <br>

  ![Version](https://badge.fury.io/gh/arkstructcodes%2Fpadpipe.svg?icon=si%3Agithub)
</div>


## Install

Pre-built binaries can be downloaded directly from the [releases](https://github.com/arkstructcodes/padpipe/releases/latest) page.

To install from source, run:

    cargo install --locked --git https://github.com/ArkStructCodes/padpipe

This requires the nightly toolchain to build.


## Documentation

Soon™ :)


## Troubleshooting

If the device cannot be accessed, ensure the `uinput` [kernel module](https://wiki.archlinux.org/title/Kernel_module) is loaded.

If any issues occur regarding permissions, create a [udev rule](https://wiki.archlinux.org/title/Udev#About_udev_rules) to allow access to the device:

    echo 'KERNEL=="uinput", SUBSYSTEM=="misc", TAG+="uaccess", OPTIONS+="static_node=uinput"' | sudo tee /etc/udev/rules.d/60-allow-uinput-access.rules

This is unnecessary when steam is installed as the rule is already bundled with the package.
