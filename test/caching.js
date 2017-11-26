const assert = require('chai').assert
const request = require('supertest')
const fs = require('fs-extra')
const config = require('config')
const winston = require('winston')

/* eslint-env mocha */

// clean up from previous test runs before starting the server
const dbPath = config.get('backend.dbPath')
fs.removeSync(dbPath)
const logfile = config.get('backend.logger.file')
fs.removeSync(logfile)

// start the server, which also modifies the module path
const app = require('../app.js')
const backend = require('lib/backend')

// Check if a term appears in any of the log messages.
function termFoundInLog (term) {
  return new Promise((resolve, reject) => {
    winston.query({
      level: 'info',
      fields: ['message']
    }, function (err, results) {
      if (err) {
        reject(err)
      } else {
        let found = false
        for (let row of results.file) {
          if (row.message.includes(term)) {
            found = true
            break
          }
        }
        resolve(found)
      }
    })
  })
}

//
// Give the backend a chance to initialize the database asynchronously.
// A timeout of zero is not sufficient, so this timing is fragile.
// A better solution is desired.
//
setTimeout(function () {
  //
  // Ensure asset caching is happening by examining the log file.
  // Ugly, but the externally visible behavior is not sufficient.
  //
  describe('Asset caching', function () {
    const docId = 'dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07'

    before(async function () {
      await backend.reinitDatabase()
    })

    describe('thumbnails', function () {
      it('should start by uploading an asset', function (done) {
        request(app)
          .post('/api/assets')
          .attach('asset', './test/fixtures/dcp_1069.jpg')
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

      it('should serve (a new) thumbnail', function (done) {
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

      it('should have a cache miss log message', async function () {
        const foundMiss = await termFoundInLog('cache miss for')
        assert.isTrue(foundMiss)
        const foundHit = await termFoundInLog('cache hit for')
        assert.isFalse(foundHit)
      })

      it('should serve (the cached) thumbnail', function (done) {
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

      it('should have a cache hit log message', async function () {
        const foundHit = await termFoundInLog('cache hit for')
        assert.isTrue(foundHit)
      })
    })
  })

  run()
}, 500)
