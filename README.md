![flecs](assets/flecs_rust_logo.png)

[![Generic badge](https://img.shields.io/badge/Flecs_Version-4.0.0-E56717.svg)](https://github.com/SanderMertens/flecs/releases)
[![License](https://badgen.net/pypi/license/pip/)](https://github.com/Indra-db/flecs-ecs-rs/blob/master/LICENSE)
[![CI](https://github.com/indra-db/flecs_ecs_rs/actions/workflows/ci.yml/badge.svg)](https://github.com/indra-db/flecs_ecs_rs/actions/workflows/ci.yml)
[![Flecs C/C++ Docs](https://img.shields.io/badge/Flecs%20C%2FC%2B%2B%20Docs-View-161b22)](https://www.flecs.dev/flecs/md_docs_2Docs.html)
[![Discord Badge](https://img.shields.io/badge/Join%20Flecs%20Discord-5865F2?logo=discord&logoColor=fff&style=flat)](https://discord.gg/jkEZ2jQD6F)



Hello there! Welcome to my Rust wrapper around the ECS library called [Flecs](https://github.com/SanderMertens/flecs). This project endeavors to offer Rust developers an intuitive and streamlined interface to harness the full potential of Flecs, the most advanced open source Entity Component System (ECS) library available today.

## ⚠️ Status: Alpha release happening mid June

The project is at a stage where the **core functionality** of Flecs has been **ported successfully** and is available to use today. While you're encouraged to explore, test, and provide feedback, please be aware of potential bugs and breaking changes as we continue to refine the API and add new features.

This library has not been advertized nor published yet on crates.io or other platforms until it's ready for a full alpha release.

### The progress

For detailed feature progress, please visit the [issues](https://github.com/Indra-db/flecs-ecs-rs/issues) page.


- [x] Core library ![](https://geps.dev/progress/100?dangerColor=800000&warningColor=ff9900&successColor=006600)
- [ ] Addons ![](https://geps.dev/progress/65?dangerColor=800000&warningColor=ff9900&successColor=006600) Meta + Json are incomplete / in exprimental phase, not meant to be used by public
- [ ] Documentation ![](https://geps.dev/progress/65?dangerColor=800000&warningColor=ff9900&successColor=006600) codebase documented with C++ alias + description of every function. missing % is adding rust code to Flecs docs + doc tests for examples
- [ ] Test suite ![](https://geps.dev/progress/55?dangerColor=800000&warningColor=ff9900&successColor=006600) (entity, query, observers fully tested), system partially. Still many tests left to do
- [ ] Examples ![](https://geps.dev/progress/70?dangerColor=800000&warningColor=ff9900&successColor=006600)[Click me for more info](https://github.com/Indra-db/flecs-ecs-rs/issues/12) - of the current supported features, all examples are done ✔️

## The Aim

The plan is to match feature parity of the C++ API, starting with the core library (done!) while also being fully documented and tested.

## Contributions

If you're excited about this project and would like to contribute, or if you've found any bugs, please feel free to raise an issue or submit a pull request. We'd love to have your involvement!

## License

MIT license, matching Flecs.

## Acknowledgements

A big shoutout to [Sander Mertens](https://github.com/SanderMertens) and all contributors to Flecs for their excellent work on the original library. This project wouldn't be possible without them.


---

**Thank you for stopping by, and stay tuned for updates as we progress on this exciting journey to bridge Rust and Flecs!**
