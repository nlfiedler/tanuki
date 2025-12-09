# Use Flutter for Frontend

- Status: accepted
- Deciders: Nathan Fiedler
- Date: 2020-08-20

## Context

The application consists of two parts, the server-side backend and the client-side frontend. With regards to the frontend, the programming language and runtime are generally determined by the target environment. For the web, the choice is either JavaScript, or something that compiles to JavaScript. For macOS, this is likely Swift or Objective-C, and for Windows and Linux, C++. However, there are desktop application frameworks that work at a level slighty above the native layer, and often cross-compile to different platforms.

In terms of those application frameworks, several choices are available that stay well above the native layer while still offering adequate integration with the underlying platform. The most popular choice at this time is [Electron](https://www.electronjs.org) which leverages V8, Node.js, and the Chromium library. Using Electron means writing the application in HTML, CSS, and JavaScript. Another option that is rapidly gaining traction in web and desktop development is [Flutter](https://flutter.dev), for which application code is written in [Dart](https://dart.dev).

Other options include [Haxe](https://haxe.org), which is a Java-like language that cross-compiles to a variety of target platforms. However, it seems that it has very little traction compared to either Electron or Flutter. Similarly, there is [ReScript](https://rescript-lang.org) (née ReasonML), which is even less popular and rather difficult to use as it comes with a high cognitive load (it is necessary to learn both ReScript and OCaml, as well as navigating numerous documentation sites for the various standard libraries).

## Decision

At one point a rather basic Electron application was written for this application, and that was working well. However, the runtime is fairly heavyweight in terms of memory usage, and using HTML, CSS, and JavaScript can be rather challenging for a developer that is not very comfortable with those technologies. Developing a sophisticated, desktop-like interface using only web technology is especially difficult.

On the other hand, Flutter is designed from the ground up for developing applications, whether they run on a mobile phone, the web, or a desktop such as macOS. With corporate backing from Google, which is betting big on winning developer mindshare away from Apple and Microsoft, it is a safe bet that Dart and Flutter are not going away any time soon. Compare with GitHub, which has no real reason to be developing Electron at all.

The choice is **Flutter**.

## Consequences

While not yet complete for this application, another application has had its frontend written in Dart/Flutter, and that has worked out very well. It compiles quickly, produces a decent sized binary, and works without any issues. The tooling and platform are updated frequently, and everything is generally getting better each time. Compare with ReasonML which has been stuck in neutral for a long time.

## Links

- Flutter [website](https://flutter.dev)
- Dart [website](https://dart.dev)
