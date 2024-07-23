# Introduction

This is an example application used for exploration of embassy/embedded async Rust on esp32c3 board.

The application should exhibit the following behavior:

- Every 5 seconds a temperature is read and published to public MQTT broker.
- If user presses BOOT button, temperature is read and published.
- If specific message is published to a select topic on the said broker, temperature is read and published.

Application publishes to HiveMQ public broker under the topic `temperature/1` (see `src/mqtt.rs`). It's also subscribed to `get-temp/1` and is listening for `it's me` messages that trigger temperature reading (both can be changed in `src/main.rs`). Currently a static IP configuration is used with IP address set to `192.168.1.88/24`. The IP can be set in `src/netstack.rs`.

## Why Rust

- safety
- ergonomics

## Why Embassy

- no-std (no libc)
- async (composable, zero-cost high-level abstraction)
- low-power

## Building and running the application

For toolchain setup see: https://docs.esp-rs.org/no_std-training/02_2_software.html

Afterwards, navigate to anywhere from inside the project directory and run:

```
SSID=test PASSWORD=12345678 cargo r --release
```

where `SSID` and `PASSWORD` env variables are set appropriately.

Assuming you're running a Linux distribution, make sure the device is actually connected to the host computer and that it was detected (e.g., try `lsusb` or simply querry `dmesg` output). Make sure your user account has the appropriate rights to read/write to the serial port (if not you'll probably want to add your user to the appropriate group, e.g. `dialout` -- see file permissions to figure this out).
