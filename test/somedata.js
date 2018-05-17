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
  //
  // attributes tests (tags, locations, etc) with some data
  //
  describe('Attributes with little data', function () {
    before(async function () {
      await backend.reinitDatabase()
      const testData = [
        {
          '_id': '39092991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100',
          'filename': 'IMG_6005.JPG',
          'filesize': 159675,
          'import_date': Date.UTC(2014, 0, 21, 17, 8),
          'location': 'San Francisco',
          'mimetype': 'image/jpeg',
          'tags': ['cat', 'CHEESEburger']
        },
        {
          '_id': 'b8fc5da331100390929c2f323accc5686075e3150cb91d6dde424191780ea7ea',
          'filename': 'IMG_6005.MOV',
          'filesize': 159612075,
          'import_date': Date.UTC(2014, 10, 2, 6, 1),
          'location': 'san francisco',
          'mimetype': 'video/quicktime',
          'original_date': Date.UTC(2013, 9, 24, 15, 9),
          'tags': ['dog', 'picnic']
        },
        {
          '_id': '9594b84f1d0db2762d1c53b7ee1a12d03adad33d3193d8b5ed1a50fab2bbff15',
          'filename': 'img0315.jpg',
          'filesize': 431671,
          'import_date': Date.UTC(2014, 6, 21, 5, 34),
          'mimetype': 'image/jpeg',
          'original_date': null,
          'tags': ['cat', 'picnic']
        },
        {
          '_id': 'e4f78c848a4ebcf180c68e2a80e117f3c710577994e337454177a80f0c9d6042',
          'filename': 'tagless.jpg',
          'filesize': 123456,
          'import_date': Date.UTC(2015, 6, 9, 10, 15),
          'mimetype': 'image/jpeg',
          'original_date': null,
          'tags': [] // tags should never be null, just an empty list
        }
      ]
      for (let doc of testData) {
        await backend.updateDocument(doc)
      }
    })

    describe('assets', function () {
      it('should return count of 4', function (done) {
        request(app)
        request(app)
          .post('/graphql')
          .send({
            query: `query {
              count
            }`
          })
          .expect(200)
          .expect(res => {
            assert.equal(res.body.data.count, 4)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets by filename', function () {
      it('should return assets with matching filename', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "filename": "img0315.jpg"
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  id
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 1)
            assert.equal(
              search.results[0].id,
              '9594b84f1d0db2762d1c53b7ee1a12d03adad33d3193d8b5ed1a50fab2bbff15'
            )
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should match filenames case insensitively', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "filename": "IMG0315.JPG"
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  id
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 1)
            assert.equal(
              search.results[0].id,
              '9594b84f1d0db2762d1c53b7ee1a12d03adad33d3193d8b5ed1a50fab2bbff15'
            )
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets by mimetype', function () {
      it('should return assets with matching mimetype', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "mimetype": "video/quicktime"
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  filename
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 1)
            assert.equal(search.results[0].filename, 'IMG_6005.MOV')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should match mimetypes case insensitively', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "mimetype": "VIDEO/QUICKTIME"
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  filename
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 1)
            assert.equal(search.results[0].filename, 'IMG_6005.MOV')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets by location', function () {
      it('should return assets with matching locations', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "locations": ["san francisco"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  id
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 2)
            assert.equal(
              search.results[0].id,
              '39092991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100'
            )
            assert.equal(
              search.results[1].id,
              'b8fc5da331100390929c2f323accc5686075e3150cb91d6dde424191780ea7ea'
            )
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should match locations case insensitively', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "locations": ["SAN FRANCISCO"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  id
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 2)
            assert.equal(
              search.results[0].id,
              '39092991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100'
            )
            assert.equal(
              search.results[1].id,
              'b8fc5da331100390929c2f323accc5686075e3150cb91d6dde424191780ea7ea'
            )
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets by tag', function () {
      it('should return assets with matching tags', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "tags": ["picnic"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  filename
                  location
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 2)
            assert.include([
              search.results[0].filename,
              search.results[1].filename
            ], 'img0315.jpg')
            assert.include([
              search.results[0].location,
              search.results[1].location
            ], 'san francisco')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should match tags case insensitively', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "tags": ["cheeseburger"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  filename
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 1)
            assert.equal(search.results[0].filename, 'IMG_6005.JPG')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets without any tags', function () {
      it('should return assets with no tags', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "tags": [null]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  filename
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 1)
            assert.equal(search.results[0].filename, 'tagless.jpg')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets without any location', function () {
      it('should return assets with no location', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "locations": [null]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  filename
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 2)
            assert.include([
              search.results[0].filename,
              search.results[1].filename
            ], 'img0315.jpg')
            assert.include([
              search.results[0].filename,
              search.results[1].filename
            ], 'tagless.jpg')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets without tags and location', function () {
      it('should return assets with missing tags/location', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "locations": [null],
                "tags": [null]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  filename
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 1)
            assert.equal(search.results[0].filename, 'tagless.jpg')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets with filename in 2015', function () {
      it('should return assets matching filename in 2015', function (done) {
        const afterTime = new Date(2015, 0).getTime()
        const beforeTime = new Date(2016, 0).getTime()
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "after": ${afterTime},
                "before": ${beforeTime},
                "filename": "tagless.jpg"
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  id
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 1)
            assert.equal(
              search.results[0].id,
              'e4f78c848a4ebcf180c68e2a80e117f3c710577994e337454177a80f0c9d6042'
            )
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets with mimetype in 2013 or 2014', function () {
      it('should return assets matching mimetype in 2013 or 2014', function (done) {
        const afterTime = new Date(2013, 0).getTime()
        const beforeTime = new Date(2015, 0).getTime()
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "after": ${afterTime},
                "before": ${beforeTime},
                "mimetype": "video/quicktime"
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  id
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 1)
            assert.equal(
              search.results[0].id,
              'b8fc5da331100390929c2f323accc5686075e3150cb91d6dde424191780ea7ea'
            )
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets with filename, mimetype, in 2013 or 2014', function () {
      it('should return assets matching filename, mimetype, in 2013 or 2014', function (done) {
        const afterTime = new Date(2013, 0).getTime()
        const beforeTime = new Date(2015, 0).getTime()
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "after": ${afterTime},
                "before": ${beforeTime},
                "filename": "IMG_6005.MOV",
                "mimetype": "video/quicktime"
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  id
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 1)
            assert.equal(
              search.results[0].id,
              'b8fc5da331100390929c2f323accc5686075e3150cb91d6dde424191780ea7ea'
            )
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets w/o tags/location in 2015', function () {
      it('should return assets matching query', function (done) {
        const afterTime = new Date(2015, 0).getTime()
        const beforeTime = new Date(2016, 0).getTime()
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "after": ${afterTime},
                "before": ${beforeTime},
                "locations": [null],
                "tags": [null]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  filename
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 1)
            assert.equal(search.results[0].filename, 'tagless.jpg')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('asset by multiple tags', function () {
      it('should return exactly the one matching asset', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "tags": ["cat", "cheeseburger"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  filename
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.equal(search.count, 1)
            assert.equal(search.results[0].filename, 'IMG_6005.JPG')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('tags', function () {
      it('should return list of tags', function (done) {
        request(app)
          .post('/graphql')
          .send({
            query: `query {
              tags {
                value
                count
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const tags = res.body.data.tags
            assert.equal(tags.length, 4)
            assert.deepEqual(tags[0], {value: 'cat', count: 2})
            assert.deepEqual(tags[1], {value: 'cheeseburger', count: 1})
            assert.deepEqual(tags[2], {value: 'dog', count: 1})
            assert.deepEqual(tags[3], {value: 'picnic', count: 2})
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('locations', function () {
      it('should return list of locations', function (done) {
        request(app)
          .post('/graphql')
          .send({
            query: `query {
              locations {
                value
                count
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const locations = res.body.data.locations
            assert.equal(locations.length, 1)
            assert.deepEqual(locations[0], {value: 'san francisco', count: 2})
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('years', function () {
      it('should return list of years', function (done) {
        request(app)
          .post('/graphql')
          .send({
            query: `query {
              years {
                value
                count
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const years = res.body.data.years
            assert.equal(years.length, 3)
            assert.deepEqual(years[0], {value: 2013, count: 1})
            assert.deepEqual(years[1], {value: 2014, count: 2})
            assert.deepEqual(years[2], {value: 2015, count: 1})
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
