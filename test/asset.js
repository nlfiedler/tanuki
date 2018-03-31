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
  describe('Asset retrieval', function () {
    const docId = '37665f499b5ddb74ddc297e89dfad4f06a6c8a90'

    before(async function () {
      await backend.reinitDatabase()
      let doc = {
        _id: docId,
        filename: 'IMG_1001.JPG',
        import_date: [2017, 11, 18, 17, 3],
        filesize: 1048576,
        location: 'kyoto',
        mimetype: 'image/jpeg',
        tags: ['puella', 'magi', 'madoka', 'magica']
      }
      await backend.updateDocument(doc)
    })

    // Exception gets logged to the console, kind of ugly.
    // describe('no such asset', function () {
    //   it('should return an error', function (done) {
    //     request(app)
    //       .post(`/graphql`)
    //       .send({
    //         query: `query {
    //           asset(id: "nosuch") {
    //             id
    //           }
    //         }`
    //       })
    //       .expect(200)
    //       .expect(/missing/)
    //       .end(function (err, res) {
    //         if (err) {
    //           return done(err)
    //         }
    //         done()
    //       })
    //   })
    // })

    describe('asset by correct identifier', function () {
      it('should return all asset details', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              asset(id: "${docId}") {
                filename
                filesize
                location
                mimetype
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.asset
            assert.equal(asset.filename, 'IMG_1001.JPG')
            assert.equal(asset.filesize, 1048576)
            assert.equal(asset.location, 'kyoto')
            assert.equal(asset.mimetype, 'image/jpeg')
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

  describe('Asset creation and update', function () {
    const docId = 'dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07'

    before(async function () {
      await backend.reinitDatabase()
    })

    describe('upload an asset', function () {
      it('should create a new document', function (done) {
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

      it('should serve the new asset', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              asset(id: "${docId}") {
                filename
                datetime
                mimetype
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.asset
            assert.equal(asset.filename, 'dcp_1069.jpg')
            assert.equal(asset.mimetype, 'image/jpeg')
            let date = new Date(asset.datetime)
            assert.equal(date.getFullYear(), 2003)
            assert.equal(date.getMonth() + 1, 9)
            assert.equal(date.getDate(), 3)
            assert.equal(date.getHours(), 17)
            assert.equal(date.getMinutes(), 24)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should permit updating asset fields', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            variables: `{
              "input": {
                "caption": "a mild mannered cow",
                "location": "hawaii",
                "tags": ["cow", "fence", "grass"]
              }
            }`,
            operationName: 'Update',
            query: `mutation Update($input: AssetInput!) {
              update(id: "${docId}", asset: $input) {
                caption
                location
                tags
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.update
            assert.equal(asset.caption, 'a mild mannered cow')
            assert.equal(asset.location, 'hawaii')
            assert.equal(asset.tags.length, 3)
            assert.equal(asset.tags[0], 'cow')
            assert.equal(asset.tags[1], 'fence')
            assert.equal(asset.tags[2], 'grass')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should parse optional user dates', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            // datetime is 2003-08-30 12:45
            variables: `{
              "input": {
                "datetime": 1062272700000
              }
            }`,
            operationName: 'Update',
            query: `mutation Update($input: AssetInput!) {
              update(id: "${docId}", asset: $input) {
                datetime
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.update
            let date = new Date(asset.datetime)
            assert.equal(date.getFullYear(), 2003)
            assert.equal(date.getMonth() + 1, 8)
            assert.equal(date.getDate(), 30)
            assert.equal(date.getHours(), 12)
            assert.equal(date.getMinutes(), 45)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should permit clearing the user date', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            variables: `{
              "input": {
                "datetime": null
              }
            }`,
            operationName: 'Update',
            query: `mutation Update($input: AssetInput!) {
              update(id: "${docId}", asset: $input) {
                datetime
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.update
            let date = new Date(asset.datetime)
            assert.equal(date.getFullYear(), 2003)
            assert.equal(date.getMonth() + 1, 9)
            assert.equal(date.getDate(), 3)
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
