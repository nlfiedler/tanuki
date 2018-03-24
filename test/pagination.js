//
// Copyright (c) 2018 Nathan Fiedler
//
const {assert} = require('chai')
const {before, describe, it, run} = require('mocha')
const request = require('supertest')
const fs = require('fs-extra')
const config = require('config')
const pouchCollate = require('pouchdb-collate')

// clean up from previous test runs before starting the server
const dbPath = config.get('backend.dbPath')
fs.removeSync(dbPath)

// start the server, which also modifies the module path
const app = require('../app.js')
const backend = require('lib/backend')

function sampleOne (arr) {
  return arr[Math.floor(Math.random() * arr.length)]
}

//
// Give the backend a chance to initialize the database asynchronously.
// A timeout of zero is not sufficient, so this timing is fragile.
// A better solution is desired.
//
setTimeout(function () {
  describe('Pagination', function () {
    before(async function () {
      await backend.reinitDatabase()
      const userList = ['akemi', 'chise', 'homura', 'kyoko', 'madoka', 'midori', 'sayaka']
      for (let n = 0; n < 16; n++) {
        // The file date will be used to cause the results to appear in the
        // desired order, making the pagination easier to test (by looking at
        // the file name which is cheaper than parsing dates again).
        const fileName = `IMG_${1000 + n}.JPG`
        const fileOwner = sampleOne(userList)
        // produce identifiers that have decent entropy and distribution
        const id = pouchCollate.toIndexableString([fileName, fileOwner])
        let doc = {
          _id: id,
          file_date: [2000 + n, 5, 13, 5, 26],
          file_name: fileName,
          import_date: [2017, 11, 18, 17, 3],
          file_owner: fileOwner,
          file_size: 1048576,
          location: 'kamakura',
          mimetype: 'image/jpeg',
          tags: ['cat']
        }
        await backend.updateDocument(doc)
      }
      // Prime the indices so the tests appear to run faster and do not show
      // duration values in red, which looks bad.
      await backend.byTags(['foobar'])
    })

    describe('count of 6, default offset of 0', function () {
      it('should return 6 assets', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              search(tags: ["cat"], count: 6) {
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
          .post(`/graphql`)
          .send({
            query: `query {
              search(tags: ["cat"], count: 6, offset: 6) {
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
          .post(`/graphql`)
          .send({
            query: `query {
              search(tags: ["cat"], count: 6, offset: 12) {
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
          .post(`/graphql`)
          .send({
            query: `query {
              search(tags: ["cat"], count: 6, offset: 60) {
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
          .post(`/graphql`)
          .send({
            query: `query {
              search(tags: ["cat"], count: 50) {
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
