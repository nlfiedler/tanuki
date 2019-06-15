//
// Copyright (c) 2019 Nathan Fiedler
//
const fs = require('fs')
const path = require('path')
const express = require('express')
const resolvers = require('resolvers')
const { ApolloServer } = require('apollo-server-express')
const { makeExecutableSchema } = require('apollo-server')

const router = express.Router()

// Assemble the parts into a schema object; by using a separate file for the
// schema and invoking makeExecutableSchema() we get the docs in the playground.
const schemaPath = path.join('public', 'schema.graphql')
const typeDefs = fs.readFileSync(schemaPath, 'utf8')
const schema = makeExecutableSchema({ typeDefs, resolvers })

const server = new ApolloServer({
  schema,
  // enable the playground even in production mode
  introspection: true,
  playground: {
    tabs: [{
      endpoint: '/graphql'
    }]
  }
})
server.applyMiddleware({ app: router, path: '/' })

module.exports = router
