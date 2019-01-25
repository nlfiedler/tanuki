//
// Copyright (c) 2018 Nathan Fiedler
//
const { assert } = require('chai')
const { before, describe, it, run } = require('mocha')
const request = require('supertest')

// start the server
const app = require('../dist/app.js').default
const backend = require('lib/backend')

//
// Give the backend a chance to initialize the database asynchronously.
// A timeout of zero is not sufficient, so this timing is fragile.
// A better solution is desired.
//
setTimeout(function () {
  //
  // attributes tests (tags, locations, etc) with no data
  //
  describe('Attributes sans data', function () {
    before(async function () {
      await backend.reinitDatabase()
    })

    describe('asset count', function () {
      it('should return 0', function (done) {
        request(app)
          .post('/graphql')
          .send({
            query: `query {
              count
            }`
          })
          .expect(200)
          .expect(res => {
            assert.equal(res.body.data.count, 0)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('asset search', function () {
      it('should return nothing', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {}
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  id
                }
                count
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const search = res.body.data.search
            assert.equal(search.results.length, 0)
            assert.equal(search.count, 0)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('asset search by tag', function () {
      it('should return nothing', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "tags": ["picnic"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  id
                }
                count
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const search = res.body.data.search
            assert.equal(search.results.length, 0)
            assert.equal(search.count, 0)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('tags', function () {
      it('should return nothing', function (done) {
        request(app)
          .post('/graphql')
          .send({
            query: `query {
              tags {
                value
                count
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const tags = res.body.data.tags
            assert.equal(tags.length, 0)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('locations', function () {
      it('should return nothing', function (done) {
        request(app)
          .post('/graphql')
          .send({
            query: `query {
              locations {
                value
                count
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const locations = res.body.data.locations
            assert.equal(locations.length, 0)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('years', function () {
      it('should return nothing', function (done) {
        request(app)
          .post('/graphql')
          .send({
            query: `query {
              years {
                value
                count
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const years = res.body.data.years
            assert.equal(years.length, 0)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })
  })

  run()
}, 500)
