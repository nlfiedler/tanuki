# Use GraphQL protocol

- Status: accepted
- Deciders: Nathan Fiedler
- Date: 2020-08-20

## Context

This application consists of a server-side backend and client-side frontend. While these will often run on the same system, as in the case of a desktop application, that is not always going to be the case. As such, the two sides of the application must be able to communicate with each other.

Popular protocols include a RESTful API and more recently GraphQL. The former is rather basic and has limitations on the complexity that can be succinctly represented in the queries and results. The latter is a recent innovation that leverages HTTP and JSON to facilitate complex queries independent of the programming language or framework. Its hallmark is the ability to make queries for deeply nested data and have the results returned in as efficient a manner as possible (i.e. one request, with only the desired data).

Another option would be to implement a custom protocol, but that is tangential to the requirements of this application. While the data format might be easily outsourced to a third party library, the protocol itself still remains as non-trivial work.

## Decision

**GraphQL**, it has many available third party libraries and frameworks, tooling support, and plenty of documentation. It is efficient in terms of the query and response, and leverages existing technologies, thus lowering the learning curve.

## Consequences

The application has been using GraphQL for a long time now, and it has been very satisfactory. One interesting by-product is the GraphQL Playground (née GraphiQL) that offers a graphical user interface to the backend.

## Links

- GraphQL [website](https://graphql.org)
- RESTful API [Wikipedia](https://en.wikipedia.org/wiki/Representational_state_transfer)
