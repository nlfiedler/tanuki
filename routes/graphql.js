//
// Copyright (c) 2018 Nathan Fiedler
//
const fs = require('fs')
const path = require('path')
const express = require('express')
const resolvers = require('lib/resolvers')
const { ApolloServer } = require('apollo-server-express')
const { makeExecutableSchema } = require('apollo-server')

const router = express.Router()

// assemble the parts into a schema object
const schemaPath = path.join(__dirname, '..', 'lib', 'schema.graphql')
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
