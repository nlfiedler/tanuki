//
// Copyright (c) 2018 Nathan Fiedler
//
const { assert } = require('chai')
const { before, describe, it, run } = require('mocha')
const request = require('supertest')

// start the server, which also modifies the module path
const app = require('../dist/app.js').default
const assets = require('lib/assets')
const backend = require('lib/backend')

//
// Give the backend a chance to initialize the database asynchronously.
// A timeout of zero is not sufficient, so this timing is fragile.
// A better solution is desired.
//
setTimeout(function () {
  describe('Pagination', function () {
    before(async function () {
      await backend.reinitDatabase()
      for (let n = 0; n < 16; n++) {
        // The file date will be used to cause the results to appear in the
        // desired order, making the pagination easier to test (by looking at
        // the file name which is cheaper than parsing dates again).
        const fileName = `IMG_${1000 + n}.JPG`
        const importDate = Date.UTC(2000 + n, 10, 18, 17, 3)
        const id = assets.makeAssetId(importDate, fileName)
        let doc = {
          _id: id,
          filename: fileName,
          import_date: importDate,
          filesize: 1048576,
          location: 'kamakura',
          mimetype: 'image/jpeg',
          tags: ['cat']
        }
        await backend.updateDocument(doc)
      }
    })

    describe('count of 6, default offset of 0', function () {
      it('should return 6 assets', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "tags": ["cat"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params, count: 6) {
                results {
                  filename
                }
                count
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const search = res.body.data.search
            assert.equal(search.results.length, 6)
            assert.equal(search.results[0].filename, 'IMG_1015.JPG')
            assert.equal(search.count, 16)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('count of 6, offset of 6', function () {
      it('should return 6 assets', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "tags": ["cat"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params, count: 6, offset: 6) {
                results {
                  filename
                }
                count
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const search = res.body.data.search
            assert.equal(search.results.length, 6)
            assert.equal(search.results[0].filename, 'IMG_1009.JPG')
            assert.equal(search.count, 16)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('count of 6, offset of 12', function () {
      it('should return fewer than count assets', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "tags": ["cat"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params, count: 6, offset: 12) {
                results {
                  filename
                }
                count
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const search = res.body.data.search
            assert.equal(search.results.length, 4)
            assert.equal(search.results[3].filename, 'IMG_1000.JPG')
            assert.equal(search.count, 16)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('count of 6, out of range offset', function () {
      it('should return nothing', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "tags": ["cat"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params, count: 6, offset: 60) {
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
            assert.equal(search.count, 16)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('count of 50, default offset of 0', function () {
      it('should return all 16 assets', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "tags": ["cat"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params, count: 50) {
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
            assert.equal(search.results.length, 16)
            assert.equal(search.count, 16)
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
