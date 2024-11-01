# Scroll Proving SDK
[![Twitter Follow](https://img.shields.io/twitter/follow/Scroll_ZKP?style=social)](https://twitter.com/Scroll_ZKP)
[![Discord](https://img.shields.io/discord/984015101017346058?color=%235865F2&label=Discord&logo=discord&logoColor=%23fff)](https://discord.gg/scroll)

## Introduction

Scroll Proving SDK is a Rust library for building services that interface with the [Scroll SDK](https://github.com/scroll-tech/scroll-sdk) Coordinator service.

It is designed to be used in conjunction with the Scroll SDK, allowing proof generation to be outsourced easily to third parties, without proof generators needing to run their own full node or know about your network. 

## Repo Structure
This repo contains:
- `src/`: Core Rust library implementing the proving service interface
- `examples/`: Example implementations of external proving services using this SDK
- `docker`: Dockerfile for creating containerized versions of the examples
- `charts/scroll-proving-sdk`: Helm chart for deploying the examples on Kubernetes
- `docs/`: Documentation and implementation information

## Services Built with the Scroll Proving SDK

> [!NOTE]
> The following charts are developed and maintained by third parties. Please use with caution.

Proving Services:
- [Sindri](https://github.com/Sindri-Labs/sindri-scroll-sdk/)

## Other Scroll SDK Repos

- [Scroll SDK](https://www.github.com/scroll-tech/scroll-sdk)
- [Scroll SDK CLI](https://www.github.com/scroll-tech/scroll-sdk-cli)

## Contributing

The SDK is currently under heavy development, so if you wish to contribute, we recommend reaching out and coordinating closely with the Scroll team or raising an issue on the repo.

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.
