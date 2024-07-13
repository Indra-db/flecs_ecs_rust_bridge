![flecs](assets/flecs_rust_logo.png)

![flecs](assets/flecs_rust_logo.png)

[![Generic badge](https://img.shields.io/badge/Flecs_Version-4.0.0-E56717.svg)](https://github.com/SanderMertens/flecs/releases)
[![License](https://badgen.net/pypi/license/pip/)](https://github.com/Indra-db/flecs-ecs-rs/blob/master/LICENSE)
[![CI](https://github.com/indra-db/flecs_ecs_rs/actions/workflows/ci.yml/badge.svg)](https://github.com/indra-db/flecs_ecs_rs/actions/workflows/ci.yml)
[![Flecs Official Docs](https://img.shields.io/badge/Flecs%20C%2FC%2B%2B%20Docs-View-161b22)](https://www.flecs.dev/flecs/md_docs_2Docs.html)
[![Discord Badge](https://img.shields.io/badge/Join%20Flecs%20Discord-5865F2?logo=discord&logoColor=fff&style=flat)](https://discord.gg/jkEZ2jQD6F)

## What is the Flecs Rust API?
The Rust API is a wrapper around the [Flecs](https://github.com/SanderMertens/flecs) C API. The API is designed to offer Rust developers an intuitive and streamlined interface to harness the full potential of Flecs.

## What is Flecs ECS?

[Flecs](https://github.com/SanderMertens/flecs) is a fast and lightweight Entity Component System that lets you build games and simulations with millions of entities ([join the Discord!](https://discord.gg/BEzP5Rgrrp)). Here are some of the framework's highlights:

- Fast and portable. Due to Flecs C core, it has major bindings in several languages, including C++, C#, and now Rust!
- First open source ECS with full support for [Entity Relationships](https://www.flecs.dev/flecs/md_docs_2Relationships.html)!
- Fast native support for [hierarchies](https://www.flecs.dev/flecs/md_docs_2Relationships.html#the-childof-relationship) and [prefabs](https://www.flecs.dev/flecs/md_docs_2Relationships.html#the-isa-relationship)
- Runs [in the browser](https://flecs.dev/city) without modifications with Emscripten
- Cache-friendly [archetype/SoA storage](https://ajmmertens.medium.com/building-an-ecs-2-archetypes-and-vectorization-fe21690805f9) that can process millions of entities every frame
- Supports entities with hundreds of components and applications with tens of thousands of archetypes
- Automatic component registration that works out of the box across shared libraries/DLLs
- Write free functions with [queries](https://github.com/Indra-db/flecs-ecs-rs/blob/main/flecs_ecs/examples/flecs/queries/query_basics.rs) or run code automatically in [systems](https://github.com/Indra-db/flecs-ecs-rs/blob/main/flecs_ecs/examples/flecs/systems/system_pipeline.rs)
- Run games on multiple CPU cores with a fast lockless scheduler
- Flecs is heavily tested, running more than 8000 tests in its core library alone and used in AAA engines. The Rust API itself has 500+ tests and counting.
- Integrated (WIP Rust) [reflection framework](https://www.flecs.dev/flecs/group__c__addons__meta.html) with [JSON serializer](https://github.com/SanderMertens/flecs/tree/master/examples/cpp/reflection/basics_json) and support for [runtime components](https://github.com/SanderMertens/flecs/tree/master/examples/cpp/reflection/runtime_component)
- Powerful [query language](https://github.com/Indra-db/flecs-ecs-rs/tree/main/flecs_ecs/examples/flecs/queries) with support for [joins](https://github.com/Indra-db/flecs-ecs-rs/blob/main/flecs_ecs/examples/flecs/queries/query_setting_variables.rs) and [inheritance](https://github.com/Indra-db/flecs-ecs-rs/blob/main/flecs_ecs/examples/flecs/queries/query_component_inheritance.rs)
- [Statistics addon](https://www.flecs.dev/flecs/group__c__addons__stats.html) for profiling ECS performance
- A web-based UI for monitoring & controlling your apps ([demo](https://flecs.dev/explorer), [code](https://github.com/flecs-hub/explorer)):

## How to get started?

Add the following to your `Cargo.toml`:

```toml
[dependencies]
flecs_ecs = "0.4000.0" 
```

and start hacking away!

Make sure to check out the Rust docs (improvements coming soon), [Flecs docs](https://www.flecs.dev/flecs/md_docs_2Docs.html), and the 70+ examples in the [examples](https://github.com/Indra-db/flecs-ecs-rs/blob/main/flecs_ecs/examples/flecs/) directory.

For an example integration of Flecs with WGPU, Vello, and Winit check out the demo [here](https://github.com/james-j-obrien/flecs-wgpu-demo)

<img src="https://github.com/james-j-obrien/flecs-wgpu-demo/assets/30707409/b3b8f2fc-0758-433e-b82b-e3458f61f244" alt="demo" width="300"/>

## Status: Alpha release

The project is in the alpha release stage where the **core functionality** and some **addons** of Flecs have been **ported** and are available to use today. While there has been a lot of thought put into the current API, it's still in an experimental phase. The project aims to hit stable when all the soundness/safety issues are resolved and the API has been finalized with all of Flecs functionality. We encourage you to explore, test, and provide feedback, but please be aware of potential bugs and breaking changes as we continue to refine the API and add new features.

#### Safety

One important safety factor that has yet to be addressed is having multiple aliases to the same component. This is a known issue and is being worked on. It will be addressed through a table column lock mechanism.

This library was made publicly available on the release date of Flecs V4 release.

#### Performance

From initial benchmarks and tests, the Rust API is on par with C-level performance, except for where overhead was introduced to make the API safe to use in Rust land (e.g. `get` performance). However, performance improvements are planned to be made in the future.

### The progress

For detailed feature progress, please visit the [issues](https://github.com/Indra-db/flecs-ecs-rs/issues) page.

- Core library ![](https://geps.dev/progress/100?dangerColor=800000&warningColor=ff9900&successColor=006600)
- Addons ![](https://geps.dev/progress/75?dangerColor=800000&warningColor=ff9900&successColor=006600) (Meta + Json + Script are a WIP, expected to be released by end of August, experimental phase sooner)
- Documentation ![](https://geps.dev/progress/65?dangerColor=800000&warningColor=ff9900&successColor=006600) Most functions are documented and contain a C++ alias. Flecs documentation site contains Rust code. The remaining % is for adding mostly doc test examples and refining the Rust docs page.
- Test suite ![](https://geps.dev/progress/70?dangerColor=800000&warningColor=ff9900&successColor=006600) (entity, query, observers systems test cases are done)
- Examples ![](https://geps.dev/progress/75?dangerColor=800000&warningColor=ff9900&successColor=006600) For the current feature set, all examples are done.

#### What's next?

* Meta, Json, Script addons. This will allow for reflection, serialization, and scripting capabilities for creating entities and components. See the [Flecs documentation](https://github.com/SanderMertens/flecs/blob/v4/docs/FlecsScript.md) for more information.
* Wasm unknown unknown. The project is currently in the process of supporting wasm32-unknown-unknown target. This is expected to land in some shape or form by the end of August.
* API refinements, resolving safety issues & documentation.
* C# scripting support. Integration with [Flecs.Net](https://github.com/BeanCheeseBurrito/Flecs.NET) to work seamlessly with Flecs Rust API.
* More demos and examples.

## The Aim

The plan is to match feature parity of the C++ API, starting with the core library (done!) while also being fully documented and tested and addressing any safety issues that may arise. The project aims to provide a safe, idiomatic, and efficient Rust API for Flecs, while also being a good citizen in the Rust ecosystem.

## Contributions

If you're excited about this project and would like to contribute, or if you've found any bugs, please feel free to raise an issue or submit a pull request. We'd love to have your involvement!

## License

MIT license, matching Flecs.

## Acknowledgements

A big shoutout to [Sander Mertens](https://github.com/SanderMertens) for creating such a wonderful library and the pre-alpha testers who contributed to Flecs Rust API, especially [James](https://github.com/james-j-obrien), [Bruce](https://github.com/waywardmonkeys), and [Andrew](https://github.com/andrewgazelka).
