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
  describe('Asset without Exif original date', function () {
    let docId

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
            docId = res.body.id
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
                filesize
                mimetype
                caption
                location
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.asset
            assert.equal(asset.filename, 'fighting_kittens.jpg')
            assert.equal(asset.filesize, 39932)
            assert.equal(asset.mimetype, 'image/jpeg')
            assert.isNull(asset.caption)
            assert.isNull(asset.location)
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
    let docId

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
            docId = res.body.id
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
                mimetype
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.asset
            assert.equal(asset.filename, 'lorem-ipsum.txt')
            assert.equal(asset.mimetype, 'text/plain')
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

  describe('Special metadata handling', function () {
    let docId

    before(async function () {
      await backend.reinitDatabase()
    })

    describe('update tags and location via caption', function () {
      it('should create a new document successfully', function (done) {
        request(app)
          .post('/api/assets')
          .attach('asset', './test/fixtures/lorem-ipsum.txt')
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.status, 'success')
            docId = res.body.id
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should update the asset', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            variables: `{
              "input": {
                "tags": ["fence"]
              }
            }`,
            operationName: 'Update',
            query: `mutation Update($input: AssetInput!) {
              update(id: "${docId}", asset: $input) {
                filename
                tags
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.update
            assert.equal(asset.filename, 'lorem-ipsum.txt')
            assert.equal(asset.tags.length, 1)
            assert.equal(asset.tags[0], 'fence')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should extract tags and location from caption', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            variables: `{
              "input": {
                "caption": "a mild mannered #cow in @hawaii eating #grass"
              }
            }`,
            operationName: 'Update',
            query: `mutation Update($input: AssetInput!) {
              update(id: "${docId}", asset: $input) {
                caption
                filename
                location
                tags
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.update
            assert.equal(asset.caption, 'a mild mannered #cow in @hawaii eating #grass')
            assert.equal(asset.filename, 'lorem-ipsum.txt')
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

      it('should not overwrite location or clobber tags', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            variables: `{
              "input": {
                "caption": "a #mild mannered #cow in @field eating #grass"
              }
            }`,
            operationName: 'Update',
            query: `mutation Update($input: AssetInput!) {
              update(id: "${docId}", asset: $input) {
                caption
                filename
                location
                tags
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.update
            assert.equal(asset.caption, 'a #mild mannered #cow in @field eating #grass')
            assert.equal(asset.filename, 'lorem-ipsum.txt')
            assert.equal(asset.location, 'hawaii')
            assert.equal(asset.tags.length, 4)
            assert.equal(asset.tags[0], 'cow')
            assert.equal(asset.tags[1], 'fence')
            assert.equal(asset.tags[2], 'grass')
            assert.equal(asset.tags[3], 'mild')
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
