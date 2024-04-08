# [echolot](https://crates.io/crates/echolot)

> exchange files with sound

## Installation

```sh
$ cargo install echolot
```

## Usage

1. Transmitting party runs

```sh
$ echolot transmit <file>
```

The transmitter will then listen for initiation from receiving party.

2. On receiving party run:

```sh
$ echolot receive
```

This will send the initiation to the transmitting party and start the whole transmission process.
