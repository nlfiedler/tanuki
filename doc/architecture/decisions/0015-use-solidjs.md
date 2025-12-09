# Use SolidJS for the client

- Status: accepted
- Deciders: Nathan Fiedler
- Date: 2025-11-14

## Context

While writing Rust on the frontend was interesting, it was cumbersome due to the need for every single line of code to be both memory-safe and thread-safe. With a fine-grained reactive framework like [Leptos](https://leptos.dev), that was especially painful. Additionally, the Web API support available for Leptos applications was a bit limited, and the libraries were buggy (in 2025).

What's more, the build setup and compile time of Leptos was hugely significant when compared to the typical JavaScript application. With JavaScript, the application starts in mere seconds and runs very quickly in modern web browsers.

In the last few years, it has become undeniable that this application relies heavily on Web API support in modern browsers. As such, there is really no better choice than JavaScript to take advantage of these browser-based features. This includes the file selector, drag-and-drop, color theme, and so on.

[SolidJS](https://www.solidjs.com) is the front-end web framework on which the design of Leptos is largely based. It makes use of signals, actions, resources, and memos to create a fine-grained reactive web framework. It offers many more features than Leptos does as of 2025, and JavaScript works perfectly with this paradigm.

With regard to the language, [TypeScript](https://www.typescriptlang.org) works well and is easy to set up with [Vite](https://vite.dev). SolidJS itself is written in TypeScript and hence it makes sense to use it for the application.

## Decision

Choose **SolidJS** and **TypeScript** for the client application.

## Consequences

TBD

## Links

- SolidJS [website](https://www.solidjs.com)
- TypeScript [website](https://www.typescriptlang.org)
