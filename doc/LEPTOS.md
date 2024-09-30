# About Leptos

See the documentation on the [web site](https://leptos.dev) for an introduction.

## Build Setup

Code targeting the front-end will be marked with the `#[component]` macro, while code meant to be compiled to the backend is feature-gated on the `ssr` feature. Hence, much of the backend code will be marked as such partly because it has no need to be on the client, but also some code and dependencies cannot be compiled to WASM.

## Tips and Tricks

Writing code using Rust within a fine-grained reactive framework can be rather tricky, mostly due to contending with the borrow checker. The flow of control through the continuations defined by the `view!` macro can lead to confusing error messages regarding the onwership of objects.

* If components are not updating properly, check the console for warnings, you are probably accessing a signal from outside of a `view!` macro. Always access signals and resources inside `view!` code.
* Always access resources (as in `create_resource()`) from within `Suspense` or `Transition` elements.
* If the error `might not live long enough` occurs, try adding `move` to the closures in `For` and `Show` elements.
* If adding `move` does not help, trying using `store_value()` to create a reactive copy of the objects.
* If the error `tempoary value is dropped` occurs, try replacing `iter()` with `into_iter()` to take ownership. This works especially well with signals and stored values since they are always cloned anyway.

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
