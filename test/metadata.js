const assert = require('chai').assert
const request = require('supertest')
const fs = require('fs-extra')
const config = require('config')

/* eslint-env mocha */

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
  //
  // Test asset metadata processing.
  //
  describe('Asset without Exif original date', function () {
    const docId = '82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09'

    before(async function () {
      await backend.reinitDatabase()
    })

    describe('upload an asset without date', function () {
      it('should create a new document successfully', function (done) {
        request(app)
          .post('/api/assets')
          .attach('asset', './test/fixtures/fighting_kittens.jpg')
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.status, 'success')
            assert.equal(res.body.id, docId)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should serve the new asset', function (done) {
        request(app)
          .get(`/api/assets/${docId}`)
          .expect(200)
          .expect((res) => {
            // without DateTime fields, original_date is null
            assert.isNull(res.body.original_date)
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

  describe('Asset without any Exif data', function () {
    const docId = '095964d07f3e821659d4eb27ed9e20cd5160c53385562df727e98eb815bb371f'

    before(async function () {
      await backend.reinitDatabase()
    })

    describe('upload an asset without date', function () {
      it('should create a new document successfully', function (done) {
        request(app)
          .post('/api/assets')
          .attach('asset', './test/fixtures/lorem-ipsum.txt')
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.status, 'success')
            assert.equal(res.body.id, docId)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should serve the new asset', function (done) {
        request(app)
          .get(`/api/assets/${docId}`)
          .expect(200)
          .expect((res) => {
            // without Exif data, original_date is null
            assert.isNull(res.body.original_date)
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
