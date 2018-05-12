//
// Copyright (c) 2018 Nathan Fiedler
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
  describe('Video file metadata', function () {
    const docId = '4f86f7dd48474b8e6571beeabbd79111267f143c0786bcd45def0f6b33ae0423'

    before(async function () {
      await backend.reinitDatabase()
    })

    describe('upload a video asset', function () {
      it('should create a new document', function (done) {
        request(app)
          .post('/api/assets')
          .attach('asset', './test/fixtures/100_1206.MOV')
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

      it('should serve video asset details', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              asset(id: "${docId}") {
                datetime
                duration
                filename
                filesize
                mimetype
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.asset
            assert.equal(asset.filename, '100_1206.MOV')
            assert.equal(asset.filesize, 311139)
            assert.equal(asset.mimetype, 'video/quicktime')
            let date = new Date(asset.datetime)
            assert.equal(date.getFullYear(), 2007)
            assert.equal(date.getMonth() + 1, 9)
            assert.equal(date.getDate(), 14)
            assert.oneOf(date.getHours(), [5, 12])
            assert.equal(date.getMinutes(), 7)
            assert.approximately(asset.duration, 2, 0.5)
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
            assert.approximately(res.body.byteLength, 9000, 1000)
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