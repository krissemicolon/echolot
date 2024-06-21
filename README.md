# [echolot](https://crates.io/crates/echolot)

> a cli tool for exchanging files over sound waves

## Installation

```sh
cargo install echolot
```

## Usage

1. Transmitting party runs

```sh
echolot transmit <file>
```

The transmitter will then listen for initiation from receiving party.

2. On receiving party run:

```sh
echolot receive
```

This will send the initiation to the transmitting party and start the whole transmission process.

## Technicals

Modulation is done with a simple 256-MFSK.
Each Frequency therefore represents one byte.
