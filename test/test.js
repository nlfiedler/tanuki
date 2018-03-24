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
  //
  // attributes tests (tags, locations, etc) with no data
  //
  describe('Attributes sans data', function () {
    before(async function () {
      await backend.reinitDatabase()
    })

    describe('asset count', function () {
      it('should return 0', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              count
            }`
          })
          .expect(200)
          .expect(res => {
            assert.equal(res.body.data.count, 0)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('asset search', function () {
      it('should return nothing', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              search {
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
            assert.equal(search.count, 0)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('asset search by tag', function () {
      it('should return nothing', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              search(tags: ["picnic"]) {
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
            assert.equal(search.count, 0)
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
      it('should return nothing', function (done) {
        request(app)
          .post(`/graphql`)
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
            assert.equal(tags.length, 0)
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
      it('should return nothing', function (done) {
        request(app)
          .post(`/graphql`)
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
            assert.equal(locations.length, 0)
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
      it('should return nothing', function (done) {
        request(app)
          .post(`/graphql`)
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
            assert.equal(years.length, 0)
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

  //
  // attributes tests (tags, locations, etc) with some data
  //
  describe('Attributes with little data', function () {
    before(async function () {
      await backend.reinitDatabase()
      const testData = [
        {
          '_id': '39092991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100',
          'file_date': [2013, 1, 31, 5, 26],
          'file_name': 'IMG_6005.JPG',
          'file_owner': 'akwok',
          'file_size': 159675,
          'import_date': [2014, 1, 21, 17, 8],
          'location': 'san francisco',
          'mimetype': 'image/jpeg',
          'tags': ['cat', 'cheeseburger']
        },
        {
          '_id': 'b8fc5da331100390929c2f323accc5686075e3150cb91d6dde424191780ea7ea',
          'file_date': [2014, 11, 2, 5, 26],
          'file_name': 'IMG_6005.MOV',
          'file_owner': 'nfiedler',
          'file_size': 159612075,
          'import_date': [2014, 11, 2, 6, 1],
          'location': 'san francisco',
          'mimetype': 'video/quicktime',
          'original_date': [2014, 10, 24, 15, 9],
          'tags': ['dog', 'picnic']
        },
        {
          '_id': '9594b84f1d0db2762d1c53b7ee1a12d03adad33d3193d8b5ed1a50fab2bbff15',
          'file_date': [2014, 7, 15, 3, 13],
          'file_name': 'img0315.jpg',
          'file_owner': 'nfiedler',
          'file_size': 431671,
          'import_date': [2014, 7, 21, 5, 34],
          'mimetype': 'image/jpeg',
          'original_date': null,
          'tags': ['cat', 'picnic']
        }
      ]
      for (let doc of testData) {
        await backend.updateDocument(doc)
      }
    })

    describe('assets', function () {
      it('should return count of 3', function (done) {
        request(app)
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              count
            }`
          })
          .expect(200)
          .expect(res => {
            assert.equal(res.body.data.count, 3)
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
          .post(`/graphql`)
          .send({
            query: `query {
              search(tags: ["picnic"]) {
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
    })

    describe('asset by multiple tags', function () {
      it('should return exactly the one matching asset', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              search(tags: ["cat","cheeseburger"]) {
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
          .post(`/graphql`)
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
          .post(`/graphql`)
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
          .post(`/graphql`)
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
            assert.equal(years.length, 2)
            assert.deepEqual(years[0], {value: 2013, count: 1})
            assert.deepEqual(years[1], {value: 2014, count: 2})
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

  //
  // attributes tests (tags, locations, etc) with ample data
  //
  describe('Attributes with more data', function () {
    before(async function () {
      await backend.reinitDatabase()
      const tagList1 = ['cat', 'catastrophe', 'cheddar', 'cheese', 'cheeseburger', 'cuddle', 'cutler']
      const tagList2 = ['diddle', 'dig', 'dipstick', 'dog', 'dogmatic', 'dug', 'duster']
      const tagList3 = ['hag', 'haggle', 'hamster', 'hid', 'hot', 'huckster', 'huddle']
      const userList = ['akemi', 'chise', 'homura', 'kyoko', 'madoka', 'midori', 'sayaka']
      const locations = ['kamakura', 'kanazawa', 'kyoto', 'osaka', 'sapporo', 'tokyo', 'yokohama']
      for (let n = 0; n < 49; n++) {
        const fileName = `IMG_${1000 + n}.JPG`
        const fileOwner = sampleOne(userList)
        // produce identifiers that have decent entropy and distribution
        const id = pouchCollate.toIndexableString([fileName, fileOwner])
        const year = Math.floor(Math.random() * 7) + 2010
        let doc = {
          _id: id,
          // we only search on the year (for now), the rest is meaningless
          file_date: [year, 5, 13, 5, 26],
          file_name: fileName,
          // file date overrides import date in terms of significance,
          // so anything at all is fine here
          import_date: [2017, 11, 18, 17, 3],
          file_owner: fileOwner,
          file_size: Math.floor(Math.random() * 1048576) + 1048576,
          location: sampleOne(locations),
          mimetype: 'image/jpeg',
          tags: [sampleOne(tagList1), sampleOne(tagList2), sampleOne(tagList3)]
        }
        if (n === 5) {
          // make one with several attributes that we can check for later
          doc.file_date[0] = 2012
          doc.location = 'osaka'
          doc.tags = ['cat', 'dog', 'hot']
        } else if (n === 10) {
          // some queries rely on multiple year and location values
          doc.file_date[0] = 2013
          doc.location = 'kyoto'
        }
        await backend.updateDocument(doc)
      }
      // Prime the indices so the tests appear to run faster and do not show
      // duration values in red, which looks bad.
      await backend.allLocations()
      await backend.allTags()
      await backend.allYears()
      await backend.byLocation('osaka')
      await backend.byTags(['foobar'])
      await backend.byYear(2012)
    })

    describe('assets', function () {
      it('should return a large count', function (done) {
        request(app)
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              count
            }`
          })
          .expect(200)
          .expect(res => {
            assert.equal(res.body.data.count, 49)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets by unknown tag', function () {
      it('should return nothing', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              search(tags: ["not_in_here"]) {
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
            assert.equal(search.count, 0)
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets by one tag', function () {
      // With async/await let's go directly against the backend.
      it('should return list of matching assets', async function () {
        let rows = await backend.byTags(['dipstick'])
        assert.isNotEmpty(rows)
        for (let row of rows) {
          let doc = await backend.fetchDocument(row.checksum)
          assert.include(doc.tags, 'dipstick')
        }
      })
    })

    describe('assets by multiple tags', function () {
      // With async/await let's go directly against the backend.
      it('should return list of matching assets', async function () {
        let rows = await backend.byTags(['cat', 'dog', 'hot'])
        assert.isNotEmpty(rows)
        for (let row of rows) {
          let doc = await backend.fetchDocument(row.checksum)
          assert.include(doc.tags, 'cat')
          assert.include(doc.tags, 'dog')
          assert.include(doc.tags, 'hot')
        }
      })
    })

    describe('assets by one year', function () {
      it('should return list of matching assets', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              search(years: [2012]) {
                results {
                  date
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.isNotEmpty(search.results)
            for (let row of search.results) {
              let date = new Date(row.date)
              assert.equal(date.getFullYear(), 2012)
            }
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets with an invalid year', function () {
      it('should return an error', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              search(years: ["foo"]) {
                results {
                  date
                }
                count
              }
            }`
          })
          .expect(400)
          .expect(/Expected type Int/)
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets by multiple years', function () {
      it('should return list of matching assets', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              search(years: [2012, 2013]) {
                results {
                  date
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.isNotEmpty(search.results)
            for (let row of search.results) {
              let date = new Date(row.date)
              assert.oneOf(date.getFullYear(), [2012, 2013])
            }
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets by one location', function () {
      // With async/await let's go directly against the backend.
      it('should return list of matching assets', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              search(locations: ["osaka"]) {
                results {
                  location
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.isNotEmpty(search.results)
            for (let row of search.results) {
              assert.equal(row.location, 'osaka')
            }
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets by multiple locations', function () {
      it('should return list of matching assets', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              search(locations: ["kyoto", "osaka"]) {
                results {
                  location
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.isNotEmpty(search.results)
            for (let row of search.results) {
              assert.oneOf(row.location, ['kyoto', 'osaka'])
            }
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets by tag, location, and year', function () {
      it('should return list of matching assets', function (done) {
        request(app)
          .post(`/graphql`)
          .send({
            query: `query {
              search(tags: ["cat", "hot"], locations: ["osaka"], years: [2012]) {
                results {
                  date
                  location
                }
                count
              }
            }`
          })
          .expect(200)
          .expect((res) => {
            const search = res.body.data.search
            assert.isNotEmpty(search.results)
            for (let row of search.results) {
              assert.equal(row.location, 'osaka')
              let date = new Date(row.date)
              assert.equal(date.getFullYear(), 2012)
              // The tags are not included in the results.
            }
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
          .post(`/graphql`)
          .send({
            query: `query {
              tags {
                value
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const tags = res.body.data.tags
            assert.equal(tags.length, 21)
            assert.equal(tags[0].value, 'cat')
            assert.equal(tags[20].value, 'huddle')
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
          .post(`/graphql`)
          .send({
            query: `query {
              locations {
                value
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const locations = res.body.data.locations
            assert.equal(locations.length, 7)
            assert.equal(locations[0].value, 'kamakura')
            assert.equal(locations[6].value, 'yokohama')
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
          .post(`/graphql`)
          .send({
            query: `query {
              years {
                value
              }
            }`
          })
          .expect(200)
          .expect(res => {
            const years = res.body.data.years
            assert.isNotEmpty(years)
            let found = false
            for (let tag of years) {
              if (tag.value === 2014) {
                found = true
              }
            }
            assert.isTrue(found, 'found year 2014')
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
