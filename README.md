# anilistarr-rs

> anilist custom list provider for sonarr/radarr, now in Rust 🦀

[![GitHub Issues](https://img.shields.io/github/issues/wwmoraes/anilistarr-rs.svg)](https://github.com/wwmoraes/anilistarr-rs/issues)
[![GitHub Pull Requests](https://img.shields.io/github/issues-pr/wwmoraes/anilistarr-rs.svg)](https://github.com/wwmoraes/anilistarr-rs/pulls)

![Codecov](https://img.shields.io/codecov/c/github/wwmoraes/anilistarr-rs)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](/LICENSE)

<!-- [![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fwwmoraes%2Fanilistarr-rs.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Fwwmoraes%2Fanilistarr-rs?ref=badge_shield) -->
<!-- [![CII Best Practices](https://bestpractices.coreinfrastructure.org/projects/0/badge)](https://bestpractices.coreinfrastructure.org/projects/0) -->

<!-- [![Docker Image Size (latest semver)](https://img.shields.io/docker/image-size/wwmoraes/anilistarr-rs)](https://hub.docker.com/r/wwmoraes/anilistarr-rs)
[![Docker Image Version (latest semver)](https://img.shields.io/docker/v/wwmoraes/anilistarr-rs?label=image%20version)](https://hub.docker.com/r/wwmoraes/anilistarr-rs)
[![Docker Pulls](https://img.shields.io/docker/pulls/wwmoraes/anilistarr-rs)](https://hub.docker.com/r/wwmoraes/anilistarr-rs) -->

[![built with nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)

---

## 📝 Table of Contents

- [About](#-about)
- [Getting Started](#-getting-started)
- [Deployment](#-deployment)
- [Usage](#-usage)
- [Built Using](#-built-using)
- [TODO](./TODO.md)
- [Contributing](./CONTRIBUTING.md)
- [Authors](#-authors)
- [Acknowledgments](#-acknowledgements)

## 🧐 About

> This is a re-write of <https://github.com/wwmoraes/anilistarr>, mostly for my
> own educational purposes. Albeit not top-notch rustacean-quality code, it gave
> me great insights about how to pull a clean architecture style model in Rust.

Converts an Anilist user watching list to a custom list format which *arr apps
support.

It works by fetching the user info directly from Anilist thanks to its API, and
converts the IDs using community-provided mappings.

Try it out on a live instance at `https://anilistarr.fly.dev/`. For API details
check either the [source Swagger definition](./swagger.yaml) or the generated
[online version here][swagger-ui].

[swagger-ui]: https://editor-next.swagger.io/?url=https%3A%2F%2Fraw.githubusercontent.com%2Fwwmoraes%2Fanilistarr%2Fmaster%2Fswagger.yaml

## 🏁 Getting Started

Clone the repository and use `cargo run` to get the REST API up.

## 🔧 Running the tests

Explain how to run the automated tests for this system.

## 🎈 Usage

Configuration in general is a WIP. The code supports distinct storage and cache
options and has built-in support for different caches and stores. The handler
needs flags/configuration file support to allow switching at runtime.

Implemented solutions:

- Cache
  - LMDB
  - Redis
- Store
  - LMDB

## 🚀 Deployment

The `handler` binary is statically compiled and serves both the REST API and the
telemetry to an OTLP endpoint. Extra requirements depend on which storage and
cache technologies you've chosen; e.g. using LMDB requires a database file.

## 🔧 Built Using

- [Rust](https://www.rust-lang.org) - Base language
- [Axum](https://github.com/tokio-rs/axum) - Tower-compatible HTTP router
- [graphql-client](https://github.com/graphql-rust/graphql-client) - type-safe GraphQL client generator
- [Open Telemetry](https://opentelemetry.io) - Observability

## 🧑‍💻 Authors

- [@wwmoraes](https://github.com/wwmoraes) - Idea & Initial work

## 🎉 Acknowledgements

- Anilist for their great service and API <https://anilist.gitbook.io/anilist-apiv2-docs/>
- The community for their efforts to map IDs between services
  - <https://github.com/Fribb/anime-lists>
  - <https://github.com/Anime-Lists/anime-lists/>
