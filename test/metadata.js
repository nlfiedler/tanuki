//
// Copyright (c) 2017 Nathan Fiedler
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
  describe('Asset without Exif original date', function () {
    let docId

    before(async function () {
      await backend.reinitDatabase()
    })

    describe('upload an asset without date', function () {
      it('should create a new document successfully', function (done) {
        request(app)
          .post('/graphql')
          // graphql-upload expects the multi-part request to look a certain way
          // c.f. https://github.com/jaydenseric/graphql-multipart-request-spec
          .field('operations', JSON.stringify({
            variables: { file: null },
            operationName: 'Upload',
            query: `mutation Upload($file: Upload!) {
              upload(file: $file)
            }`
          }))
          .field('map', JSON.stringify({ 1: ['variables.file'] }))
          .attach('1', './test/fixtures/fighting_kittens.jpg')
          .expect(200)
          .expect((res) => {
            docId = res.body.data.upload
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
          .post('/graphql')
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
          .post('/graphql')
          .field('operations', JSON.stringify({
            variables: { file: null },
            operationName: 'Upload',
            query: `mutation Upload($file: Upload!) {
              upload(file: $file)
            }`
          }))
          .field('map', JSON.stringify({ 1: ['variables.file'] }))
          .attach('1', './test/fixtures/lorem-ipsum.txt')
          .expect(200)
          .expect((res) => {
            docId = res.body.data.upload
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
          .post('/graphql')
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
          .post('/graphql')
          .field('operations', JSON.stringify({
            variables: { file: null },
            operationName: 'Upload',
            query: `mutation Upload($file: Upload!) {
              upload(file: $file)
            }`
          }))
          .field('map', JSON.stringify({ 1: ['variables.file'] }))
          .attach('1', './test/fixtures/lorem-ipsum.txt')
          .expect(200)
          .expect((res) => {
            docId = res.body.data.upload
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
          .post('/graphql')
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
          .post('/graphql')
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
          .post('/graphql')
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

      it('should trim blank tags', function (done) {
        request(app)
          .post('/graphql')
          .send({
            // zapping the existing tags, adding a blank value, and having a
            // hash with nothing after it, which would also introduce a blank
            variables: `{
              "input": {
                "caption": "a #mild mannered #cow in @field eating #grass #",
                "tags": [""]
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
            assert.equal(asset.caption, 'a #mild mannered #cow in @field eating #grass #')
            assert.equal(asset.filename, 'lorem-ipsum.txt')
            assert.equal(asset.location, 'hawaii')
            assert.equal(asset.tags.length, 3)
            assert.equal(asset.tags[0], 'cow')
            assert.equal(asset.tags[1], 'grass')
            assert.equal(asset.tags[2], 'mild')
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

  describe('Special caption location handling', function () {
    let docId

    before(async function () {
      await backend.reinitDatabase()
    })

    describe('set location with quoted string', function () {
      it('should create a new document successfully', function (done) {
        request(app)
          .post('/graphql')
          .field('operations', JSON.stringify({
            variables: { file: null },
            operationName: 'Upload',
            query: `mutation Upload($file: Upload!) {
              upload(file: $file)
            }`
          }))
          .field('map', JSON.stringify({ 1: ['variables.file'] }))
          .attach('1', './test/fixtures/lorem-ipsum.txt')
          .expect(200)
          .expect((res) => {
            docId = res.body.data.upload
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should extract multi-word location from caption', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "input": {
                "caption": "#cow on the @\\"big island\\" eating #grass"
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
            assert.equal(asset.caption, '#cow on the @"big island" eating #grass')
            assert.equal(asset.location, 'big island')
            assert.equal(asset.tags.length, 2)
            assert.equal(asset.tags[0], 'cow')
            assert.equal(asset.tags[1], 'grass')
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

  describe('Strip commas from tags in captions', function () {
    let docId

    before(async function () {
      await backend.reinitDatabase()
    })

    describe('handle trailing commas on tags', function () {
      it('should create a new document successfully', function (done) {
        request(app)
          .post('/graphql')
          .field('operations', JSON.stringify({
            variables: { file: null },
            operationName: 'Upload',
            query: `mutation Upload($file: Upload!) {
              upload(file: $file)
            }`
          }))
          .field('map', JSON.stringify({ 1: ['variables.file'] }))
          .attach('1', './test/fixtures/lorem-ipsum.txt')
          .expect(200)
          .expect((res) => {
            docId = res.body.data.upload
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should remove trailing commas from tags', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "input": {
                "caption": "#cow, #grass, #fence, hot @hawaii"
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
            assert.equal(asset.caption, '#cow, #grass, #fence, hot @hawaii')
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
    })
  })

  describe('Asset with unknown media type', function () {
    let docId

    before(async function () {
      await backend.reinitDatabase()
    })

    describe('upload an asset with unrecognized extension', function () {
      it('should create a new document successfully', function (done) {
        request(app)
          .post('/graphql')
          .field('operations', JSON.stringify({
            variables: { file: null },
            operationName: 'Upload',
            query: `mutation Upload($file: Upload!) {
              upload(file: $file)
            }`
          }))
          .field('map', JSON.stringify({ 1: ['variables.file'] }))
          .attach('1', './test/fixtures/README.dumb')
          .expect(200)
          .expect((res) => {
            docId = res.body.data.upload
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should indicate generic media type', function (done) {
        request(app)
          .post('/graphql')
          .send({
            query: `query {
              asset(id: "${docId}") {
                mimetype
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.asset
            assert.equal(asset.mimetype, 'application/octet-stream')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should permit changing the media type', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "input": {
                "mimetype": "text/markdown"
              }
            }`,
            operationName: 'Update',
            query: `mutation Update($input: AssetInput!) {
              update(id: "${docId}", asset: $input) {
                mimetype
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const asset = res.body.data.update
            assert.equal(asset.mimetype, 'text/markdown')
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
