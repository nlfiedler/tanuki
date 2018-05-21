//
// Copyright (c) 2017 Nathan Fiedler
//
const {assert} = require('chai')
const {before, describe, it, run} = require('mocha')
const request = require('supertest')
const fs = require('fs-extra')
const config = require('config')

// clean up from previous test runs before starting the server
const dbPath = config.get('backend.dbPath')
fs.removeSync(dbPath)

// start the server, which also modifies the module path
const app = require('../app.js')
const backend = require('lib/backend')

//
// Give the backend a chance to initialize the database asynchronously.
// A timeout of zero is not sufficient, so this timing is fragile.
// A better solution is desired.
//
setTimeout(function () {
  describe('Asset content serving', function () {
    let docId

    before(async function () {
      await backend.reinitDatabase()
    })

    describe('upload an asset', function () {
      it('should create a new document', function (done) {
        request(app)
          .post('/import')
          .attach('asset', './test/fixtures/dcp_1069.jpg')
          .expect(302)
          .expect('Content-Type', /text/)
          .expect((res) => {
            // the asset identifier will be in the Location header
            const paths = res.header['location'].split('/')
            assert.equal(paths.length, 4)
            assert.equal(paths[1], 'assets')
            assert.equal(paths[3], 'edit')
            docId = paths[2]
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should serve the thumbnail', function (done) {
        request(app)
          .get(`/thumbnail/${docId}`)
          .expect(200)
          .expect('Content-Type', /image/)
          .expect((res) => {
            assert.instanceOf(res.body, Buffer)
            assert.approximately(res.body.byteLength, 11000, 1000)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should serve the preview', function (done) {
        request(app)
          .get(`/preview/${docId}`)
          .expect(200)
          .expect('Content-Type', /image/)
          .expect((res) => {
            assert.instanceOf(res.body, Buffer)
            assert.approximately(res.body.byteLength, 39000, 1000)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should serve the asset content', function (done) {
        request(app)
          .get(`/asset/${docId}`)
          .expect(200)
          .expect('Content-Type', /image/)
          .expect('Content-Length', '80977')
          .expect((res) => {
            assert.instanceOf(res.body, Buffer)
            assert.equal(res.body.byteLength, 80977)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('no such asset', function () {
      it('should return 404 for thumbnail', function (done) {
        request(app)
          .get('/thumbnail/nosuchid')
          .expect(404)
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should return 404 for preview', function (done) {
        request(app)
          .get('/preview/nosuchid')
          .expect(404)
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should return 404 for asset', function (done) {
        request(app)
          .get('/asset/nosuchid')
          .expect(404)
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
