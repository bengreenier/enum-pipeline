# enum-pipeline

Provides a way to use enums to describe and execute ordered data pipelines. 🦀🐾

[![CI](https://github.com/bengreenier/enum-pipeline/actions/workflows/ci.yml/badge.svg)](https://github.com/bengreenier/enum-pipeline/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/d/enum-pipeline)](https://crates.io/crates/enum-pipeline)
[![docs.rs](https://img.shields.io/docsrs/enum-pipeline)](https://docs.rs/enum-pipeline)
[![dependency status](https://deps.rs/repo/github/bengreenier/enum-pipeline/status.svg)](https://deps.rs/repo/github/bengreenier/enum-pipeline)

I needed a succinct way to describe 2d pixel map operations for a game I'm working on. I wanted callers to be able to easily determine all possible operations (hence `enum`), with per-operation data (hence variants), and their operation-specific logic. This is what I came up with!

## Quickstart

Some quick examples to get you started. For more information see [docs.rs/enum_pipeline](https://docs.rs/enum_pipeline) and [docs.rs/enum_pipeline_derive](https://docs.rs/enum_pipeline_derive).

### Derive

```
#[derive(Default)]
struct MacroMutRefData {
    a_count: i32,
    b_count: i32,
}

#[derive(ExecuteWithMut)]
#[execute_with(MacroMutRefData)]
enum MacroMutRefPipeline {
    #[handler(handle_a)]
    A(i32),
    #[handler(handle_b)]
    B,
}

impl MacroMutRefPipeline {
    fn handle_a(i: i32, arg: &mut MacroMutRefData) {
        arg.a_count += 1;
    }

    fn handle_b(arg: &mut MacroMutRefData) {
        arg.b_count += 1;
    }
}
```

Then create and execute some pipelines:

```
let mut arg = MacroMutRefData::default();
vec![MacroMutRefPipeline::A(23), MacroMutRefPipeline::B].execute_with_mut(&mut arg);
```

### Manual

```
#[derive(Default)]
struct MutRefData {
    a_count: i32,
    b_count: i32,
}

enum MutRefPipeline {
    A(i32),
    B,
}

impl ExecuteWithMut<MutRefData> for MutRefPipeline {
    fn execute_with_mut(self, arg: &mut MutRefData) {
        match self {
            MutRefPipeline::A(i) => arg.a_count += 1,
            MutRefPipeline::B => arg.b_count += 1,
        }
    }
}
```

Then create and execute some pipelines:

```
let mut arg = MutRefData::default();
vec![MutRefPipeline::A(23), MutRefPipeline::B].execute_with_mut(&mut arg);
```

## License

MIT
