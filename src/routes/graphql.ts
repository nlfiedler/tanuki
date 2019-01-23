//
// Copyright (c) 2018 Nathan Fiedler
//
import * as fs from 'fs'
import * as path from 'path'
import * as express from 'express'
const resolvers = require('lib/resolvers')
const { ApolloServer } = require('apollo-server-express')
const { makeExecutableSchema } = require('apollo-server')

const router = express.Router()

// assemble the parts into a schema object
const schemaPath = path.join(__dirname, '..', '..', 'lib', 'schema.graphql')
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

export default router
