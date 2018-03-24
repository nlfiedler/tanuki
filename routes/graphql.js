//
// Copyright (c) 2018 Nathan Fiedler
//
const fs = require('fs')
const path = require('path')
const express = require('express')
const bodyParser = require('body-parser')
const resolvers = require('lib/resolvers')
const {graphqlExpress} = require('apollo-server-express')
const {makeExecutableSchema} = require('graphql-tools')
const router = express.Router()

// assemble the parts into a schema object
const schemaPath = path.join(__dirname, '..', 'lib', 'schema.graphql')
const typeDefs = fs.readFileSync(schemaPath, 'utf8')
const schema = makeExecutableSchema({typeDefs, resolvers})

// bodyParser is needed just for POST
router.use('/', bodyParser.json(), graphqlExpress({schema}))

module.exports = router
