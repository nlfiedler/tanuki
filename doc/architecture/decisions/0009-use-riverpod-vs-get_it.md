# Use Riverpod for Service Location

* Status: accepted
* Deciders: Nathan Fiedler
* Date: 2021-01-02

## Context

With the layered design and pluggable nature of the various components in the
application, maintaining loose coupling between those layers is important. For
instance, creating and configuring a GraphQL client for the data sources to use
should be done in way that is easy to change based on the environment. Finding
these configured components is the primary purpose of a service locator, which
is essentially a global map of components. Retrieving entries is typically done
based on the type of the component, such as `GraphQLClient`. One such popular
library is [get_it](https://pub.dev/packages/get_it) which offers a variety of
functions for building and managing objects.

While `get_it` has worked well for several months, it does have some drawbacks:

1. Easy to forget to register services, resulting in runtime exceptions
1. Back-tracking service usage is difficult
1. Parameterized factory method is clumsy and suffers from runtime exceptions

A recent alternative to the well-known **Provider** library is **Riverpod**,
developed by the same author. It offers a type-safe alternative to the original
provider interface, and also works well as a service locator. Its primary
advantage is that if the code compiles, then it probably works. This is quite
unlike both the original **Provider** and **get_it**, which are error prone and
often result in runtime exceptions.

## Decision

Use **Riverpod** instead of both **Provider** and **get_it**. Note, however,
that the application will still be using the **bloc** library for the remote
queries and mutations. The BLoC design pattern is very well suited to the
complexity of managing our application state.

## Consequences

The code is much easier to navigate and discovering the usage of each provider
is trivial. With the type-safe nature of the provider interface, if the code
compiles, then it works.

## Links

* Riverpod [website](https://riverpod.dev/)
* bloc [website](https://bloclibrary.dev/)
