# About Leptos

See the documentation on the [web site](https://leptos.dev) for an introduction.

## Build Setup

Code targeting the front-end will be marked with the `#[component]` macro, while code meant to be compiled to the backend is feature-gated on the `ssr` feature. Hence, much of the backend code will be marked as such partly because it has no need to be on the client, but also some code and dependencies cannot be compiled to WASM.

## Tips and Tricks

Writing code using Rust within a fine-grained reactive framework can be rather tricky, mostly due to contending with the borrow checker. The flow of control through the continuations defined by the `view!` macro can lead to confusing error messages regarding the onwership of objects.

* Leverage the reactive nature of Leptos for the best performance:
    - Use `class:some_name=move || some_predicate_fn()` to reactively enable a CSS class; trying to dynamically configure the `style` will likely result in everything rendering again.
    - Use `For` for iterating over lists of items
* If components are not updating properly, check the console for warnings, you are probably accessing a signal from outside of a `view!` macro. Always access signals and resources inside `view!` code.
* Always access resources (as in `Resource::new()`) from within `Suspense` or `Transition` elements.
* If the error `might not live long enough` occurs, try adding `move` to the closures in `For` and `Show` elements.
* If adding `move` does not help, trying using `StoredValue::new()` to create a reactive copy of the objects.
* If the error `tempoary value is dropped` occurs, try replacing `iter()` with `into_iter()` to take ownership. This works especially well with signals and stored values since they are always cloned anyway.
* Do _not_ try to make a memo out of a resource, the console will show warnings/errors regarding components that could not be hydrated properly.
* Invoking server function complains about **missing fields**: caused by the JSON/QS serde for fields of structs that are lists (`Vec`). If the list is empty the serde doesn't serialize it at all and the receiving side balks because the field is missing. Either use CBOR (e.g. `#[leptos::server(.., input = server_fn::codec::Cbor)]`) or add `#[serde(default)]` in the struct definition for any problematic fields.
* Accessing a `StoredValue` inside `For` complains that the reactive value has been disposed: instead of creating the `StoredValue` inside the `each` of the `For`, add a code block inside the `<For>` in which the stored values are created from the cloned `let:elem`. This will likely come up when trying to make a `StoredValue` out of a `Resource` result in which that result does not change but the `For` is rebuilt for some other reason (a signal changed).

## Troubleshooting

### Formatting RSX in VS Code is not working

No idea why this happens, it will work for one project but not the other, even though the relevant files (`rustfmt.toml` and friends) are exactly the same. One change that made a difference was to add the following to the VS Code workspace file:

```json
"settings": {
    "rust-analyzer.rustfmt.overrideCommand": ["leptosfmt", "--stdin", "--rustfmt"]
}
```

### Attempt to upgrade Leptos results in errors

Example of an error resulting from the upgrade.

```
error[E0599]: the method `get` exists for struct `Signal<Vec<SendWrapper<File>>>`, but its trait bounds were not satisfied
   --> src/preso/leptos/upload.rs:73:43
    |
73  |     let has_files = move || dropped_files.get().len() > 0;
    |                                           ^^^ method cannot be called on `Signal<Vec<SendWrapper<File>>>` due to unsatisfied trait bounds
    |
   ::: /Users/nfiedler/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/reactive_graph-0.1.8/src/wrappers.rs:375:5
    |
375 |     pub struct Signal<T, S = SyncStorage>
    |     ------------------------------------- doesn't satisfy 5 bounds
    |
    = note: the following trait bounds were not satisfied:
[...unimportant details elided...]
    = help: items from traits can only be used if the trait is in scope
help: trait `Get` which provides `get` is implemented but not in scope; perhaps you want to import it
    |
4   + use reactive_graph::traits::Get;
    |
```

There is a mismatch in the versions of the `leptos` crates, likely caused by one of the dependencies. It is probably the `leptos-use` crate that depends on an earlier version of Leptos. Use `cargo tree` to examine the dependencies and see which crate is pulling in an older version of Leptos.

## Tools

* leptosfmt: https://github.com/bram209/leptosfmt

## Resources

* Leptos resources: https://github.com/leptos-rs/awesome-leptos
* browser integration: https://leptos-use.rs
* web-sys/js-sys wrapper: https://gloo-rs.web.app
* SCSS/Sass: https://sass-lang.com
* Bulma CSS: https://bulma.io
* Bulma extensions: https://bulma.io/extensions
* Font Awesome: https://fontawesome.com
