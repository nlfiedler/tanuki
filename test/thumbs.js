//
// Copyright (c) 2018 Nathan Fiedler
//
const { assert } = require('chai')
const { before, describe, it, run } = require('mocha')
const request = require('supertest')

// start the server, which also modifies the module path
const app = require('../dist/app.js').default
const backend = require('lib/backend')
const thumbs = require('lib/thumbs')

//
// Give the backend a chance to initialize the database asynchronously.
// A timeout of zero is not sufficient, so this timing is fragile.
// A better solution is desired.
//
setTimeout(function () {
  describe('Track dimensions of wide thumbnails', function () {
    before(async function () {
      await backend.reinitDatabase()
    })

    describe('image asset', function () {
      let docId

      it('should create a new document', function (done) {
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
          .attach('1', './test/fixtures/animal-cat-cute-126407.jpg')
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

      it('should compute the size of the thumbnail', async function () {
        const size = await thumbs.getSize(docId)
        assert.equal(size.width, 533)
        assert.equal(size.height, 300)
        // first time it will not have a revision field
        assert.isUndefined(size._rev)
      })

      it('second time should return cached result', async function () {
        const size = await thumbs.getSize(docId)
        assert.equal(size.width, 533)
        assert.equal(size.height, 300)
        // subsequent calls will have the revision field
        assert.isDefined(size._rev)
      })
    })

    describe('textual asset', function () {
      let docId

      it('should create a new document', function (done) {
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

      it('should give -1 for size of the thumbnail', async function () {
        const size = await thumbs.getSize(docId)
        assert.equal(size.width, -1)
        assert.equal(size.height, -1)
      })
    })

    describe('missing asset', function () {
      it('should give 0 for size of the thumbnail', async function () {
        const size = await thumbs.getSize('nosuchid')
        assert.equal(size.width, 0)
        assert.equal(size.height, 0)
      })
    })
  })

  run()
}, 500)
