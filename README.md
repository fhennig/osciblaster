# osciblaster

Control PWM pins with OSC.

This package builds on top of the amazing [pi-blaster](https://github.com/sarfata/pi-blaster) software 
and allows to control the pins on the Raspberry Pi with OSC packets.

In a config file, map bin ids to paths on which they can be set.
Also in the config, set the port to listen on for OSC traffic.

Every path will accept a single float argument that it will set the pin to.

## Build

This project uses [cross](https://github.com/rust-embedded/cross) to cross-compile for the ARMv7, the Raspberry Pi
architecture. Run

    cross build --release

To build a release binary deployable on the Raspberry Pi.
The file will be placed in `target/armv7-unknown-linux-gnueabihf/release/osciblaster`.

## Setup

First setup pi-blaster by following the instructions in its README.  Note that every pin that should be 
controlled needs to be configured in advance.

Once pi-blaster is setup, you can run osciblaster.  Place the binary somewhere (for example in `/opt/osciblaster`)
and place a `conf.yaml` file alongside it.  The file should look something like this:

    port: 4242
    piblaster: /dev/pi-blaster
    osc_pin_map:
      /slider: 15
      /rotary1: [14, 23, 17, 10]
      /rotary2: [18, 24, 22, 11]

A `port` and the `piblaster` path need to be configured, and in the `osc_pin_map` you can map arbitrary OSC paths
to pins.  Note that you can either configure single pins or a list of pins that should all be bound to the same path.

## Autostart / systemd setup

To have osciblaster run at startup, you can configure it as a systemd service.
In `/etc/systemd/system` create the file `osciblaster.service` with the following contents:

    [Unit]
    Description=osciblaster
    After=pi-blaster.service
    Requires=pi-blaster.service

    [Service]
    Type=simple
    Environment=RUST_BACKTRACE=1
    WorkingDirectory=/opt/osciblaster
    ExecStart=/opt/osciblaster/osciblaster
        
    [Install]
    WantedBy=multi-user.target

Replace `/opt/osciblaster` with a different path if you've installed it somewhere else.
Reload the daemon (`systemctl daemon-reload`), enable the service (`systemctl enable osciblaster`) and you should be good to go!