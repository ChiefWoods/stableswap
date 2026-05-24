# Stableswap

Stableswap program for [Blueshift](https://learn.blueshift.gg/).

[Source Repository](https://github.com/ChiefWoods/stableswap)

## Built With

### Languages

- [![Anchor](https://img.shields.io/badge/Anchor-007fd7?style=for-the-badge)](https://www.anchor-lang.com/)

## Getting Started

### Prerequisites

1. Update your Solana CLI, avm to the latest version

```sh
agave-install init 3.1.10
avm use 1.0.2
```

### Setup

1. Clone the repository

```sh
git clone https://github.com/ChiefWoods/stableswap.git
```

2. Install all dependencies

```sh
bun i
```

3. Resync your program id

```sh
anchor keys sync
```

4. Build the program

```sh
anchor build
```

#### Testing

Run all tests under `/tests`.

```sh
cargo test
```

#### Deployment

1. Configure to use localnet

```sh
solana config set -ul
```

2. Deploy the program

```sh
anchor deploy
```

3. Optionally initialize IDL

```sh
anchor idl init -f target/idl/stableswap.json <PROGRAM_ID>
```

## Issues

View the [open issues](https://github.com/ChiefWoods/stableswap/issues) for a full list of proposed features and known bugs.

## Acknowledgements

### Resources

- [Shields.io](https://shields.io/)

## Contact

[chii.yuen@hotmail.com](mailto:chii.yuen@hotmail.com)