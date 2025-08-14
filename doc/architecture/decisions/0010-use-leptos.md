# Use Leptos for Frontend

* Status: accepted
* Deciders: Nathan Fiedler
* Date: 2024-10-17

## Context

The application consists of two parts, the server-side backend and the client-side frontend. With regards to the frontend, the programming language and runtime are generally determined by the target environment. For the web, the choice is either JavaScript, or something that compiles to JavaScript. More recently, however, [WebAssembly](https://webassembly.org) has become widely available on all major browsers, which offers an intriguing alternative.

In 2020 the front-end was rewritten using [Flutter](https://flutter.dev), and while that was a good approach for targeting a cross-platform product, it has not always been the best choice technically. While Flutter is good at controlling layout and rendering the fine details of interface elements, it is not native to the web and hence some features require plugins. Flutter sometimes has weird behavior exclusive to the web such as [issue #14](https://github.com/nlfiedler/choose_input_chips/issues/14) in the `choose_input_chips` package. Because Flutter renders every pixel on the screen, use of mono-spaced text requires including that mono-spaced font in the assets.

Meanwhile, by leveraging HTML 5 features with a modern browser you can build a full-featured application with less overall effort. HTML 5 offers the `video` and `audio` tags for playing media natively in the browser, and the `input` element in combination with a `datalist` makes completion on text fields both trivial and accessible. Additionally, the `datetime-local` input type provides an easy-to-use date/time picker without the need for an extra package. A CSS-only framework like [Bulma](https://bulma.io) gives the elements a clean design and familiar look-and-feel, and utilizing its styles from any framework is easy.

Considering all that HTML 5 can offer, and the rise of WebAssembly, a framework like [Leptos](https://leptos.dev) begins to be very appealing. Leptos enables easy code-sharing between client and server, avoiding duplicate effort. It renders the initial page quickly and hydrates it after loading, and can begin filling in data during the initial request. The client/server connection in Leptos is effectively seamless, eliminating the need to explicitly serialize and deserialize parameters between the client and server.

A couple of drawbacks with Leptos are that the build setup is a little complicated due to the necessary feature gating to split code into either the native backend or the WASM frontend. Configuring some crates, such as `leptos-use` can be tricky, resulting in a build that can cause the server to panic.

On a related note, the CSS framework of choice is Bulma, for several reasons:

1. Bulma is CSS-only and thus works well with Leptos. Both [Bootstrap](https://getbootstrap.com) and [Semantic UI](https://semantic-ui.com) require writing JavaScript.
1. [Tailwind CSS](https://tailwindcss.com) is a utility CSS that has a steep learning curve and involves remembering a vast library of modifiers to craft elements from scratch.
1. [Material Design Web Components](https://github.com/material-components/material-web) is in maintenance mode as of October 2024. It is built on [web components](https://developer.mozilla.org/en-US/docs/Web/API/Web_components) and thus relies heavily on JavaScript.

## Decision

The choice is **Leptos** with **Bulma**.

## Consequences

An immediate difference between Flutter and Leptos is that Leptos is very fast. For instance, if you search for assets that yield several pages of results, then flip through the pages, the paging is nearly instantaneous. Also, the build output is half the size of the Flutter web build artifact. According to the Chrome browser, the Leptos-based front-end uses much less memory than the Flutter application ever did (~80 MB versus ~300 MB).

## Links

* Leptos [website](https://leptos.dev)
