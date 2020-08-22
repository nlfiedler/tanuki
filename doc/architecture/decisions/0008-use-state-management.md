# Use Application State Management

* Status: accepted
* Deciders: Nathan Fiedler
* Date: 2020-08-20

## Context

The backend of the application does not have any need for state management, but the frontend does. State management in terms of the frontend generally refers to form input data, remote data being displayed in the interface, and the progress of actions being performed asynchronously. The main objective of state management is to mediate between the data repositories and the application widgets.

For Flutter applications, there are several choices for managing application state, including Redux, Mobx, BLoC, and plain old Dart objects. Previous iterations of the application used something similar to Redux, and that worked well. However, compared to the `flutter_bloc` package, it is rather simplistic. BLoC is a simple pattern that is easily extended to manage events originated from streams, and likewise presents a stream of state updates to consumers.

## Decision

The choice is **BLoC** in the form of the `bloc` and `flutter_bloc` Dart packages. They are easy to implement, can model and manage any sort of state, as well as perform complex interactions.

## Consequences

BLoC has been in use with another application and has proven to be very suitable for the task. The design pattern facilitates easy unit testing, allowing the business logic to be tested separately from the interface components.

## Links

* BLoC library [website](https://bloclibrary.dev/)
