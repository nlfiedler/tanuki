//
// Copyright (c) 2017 Nathan Fiedler
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

    describe('assets', function () {
      it('should return 0', function (done) {
        request(app)
          .get('/api/assets')
          .expect('Content-Type', /json/)
          .expect(200)
          .expect('{"assets":[],"count":0}')
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('assets by tag', function () {
      it('should return nothing', function (done) {
        request(app)
          .get('/api/assets')
          .query({'tags[]': ['picnic']})
          .expect('Content-Type', /json/)
          .expect(200)
          .expect('{"assets":[],"count":0}')
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
          .get('/api/tags')
          .expect('Content-Type', /json/)
          .expect('Content-Length', '2')
          .expect(200)
          .expect('[]')
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
          .get('/api/locations')
          .expect('Content-Type', /json/)
          .expect('Content-Length', '2')
          .expect(200)
          .expect('[]')
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
          .get('/api/years')
          .expect('Content-Type', /json/)
          .expect('Content-Length', '2')
          .expect(200)
          .expect('[]')
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
          .get('/api/assets')
          .expect('Content-Type', /json/)
          .expect(200)
          .expect('{"assets":[],"count":3}')
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
          .get('/api/assets')
          .query({'tags[]': ['picnic']})
          .expect('Content-Type', /json/)
          .expect(200)
          .expect(/"file_name":"img0315.jpg"/)
          .expect(/"location":"san francisco"/)
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
          .get('/api/assets')
          .query({'tags[]': ['cat', 'cheeseburger']})
          .expect('Content-Type', /json/)
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.count, 1)
            assert.equal(res.body.assets[0].file_name, 'IMG_6005.JPG')
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
      it('should return list of tags in JSON format', function (done) {
        request(app)
          .get('/api/tags')
          .expect('Content-Type', /json/)
          .expect(200)
          .expect(/{"tag":"cheeseburger","count":1}/)
          .expect(/{"tag":"picnic","count":2}/)
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('locations', function () {
      it('should return list of locations in JSON format', function (done) {
        request(app)
          .get('/api/locations')
          .expect('Content-Type', /json/)
          .expect(200)
          .expect(/{"location":"san francisco","count":2}/)
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('years', function () {
      it('should return list of years in JSON format', function (done) {
        request(app)
          .get('/api/years')
          .expect('Content-Type', /json/)
          .expect(200)
          .expect(/{"year":2014,"count":2}/)
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
      it('should return count of 100', function (done) {
        request(app)
          .get('/api/assets')
          .expect('Content-Type', /json/)
          .expect(200)
          .expect('{"assets":[],"count":49}')
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
          .get('/api/assets')
          .query({'tags[]': ['not_in_here']})
          .expect('Content-Type', /json/)
          .expect(200)
          .expect('{"assets":[],"count":0}')
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
      // With async/await let's go directly against the backend.
      it('should return list of matching assets', function (done) {
        request(app)
          .get('/api/assets')
          .query({'years[]': [2012]})
          .expect('Content-Type', /json/)
          .expect(200)
          .expect((res) => {
            assert.isNotEmpty(res.body.assets)
            for (let row of res.body.assets) {
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
          .get('/api/assets')
          .query({'years[]': ['alpha']})
          .expect(400)
          .expect(/years must be integers/)
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
          .get('/api/assets')
          .query({'years[]': [2012, 2013]})
          .expect('Content-Type', /json/)
          .expect(200)
          .expect((res) => {
            assert.isNotEmpty(res.body.assets)
            for (let row of res.body.assets) {
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
          .get('/api/assets')
          .query({'locations[]': ['osaka']})
          .expect('Content-Type', /json/)
          .expect(200)
          .expect((res) => {
            assert.isNotEmpty(res.body.assets)
            for (let row of res.body.assets) {
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
          .get('/api/assets')
          .query({'locations[]': ['kyoto', 'osaka']})
          .expect('Content-Type', /json/)
          .expect(200)
          .expect((res) => {
            assert.isNotEmpty(res.body.assets)
            for (let row of res.body.assets) {
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
          .get('/api/assets')
          .query({
            'locations[]': ['osaka'],
            'tags[]': ['cat', 'hot'],
            'years[]': [2012]
          })
          .expect('Content-Type', /json/)
          .expect(200)
          .expect((res) => {
            assert.isNotEmpty(res.body.assets)
            for (let row of res.body.assets) {
              assert.equal(row.location, 'osaka')
              let date = new Date(row.date)
              assert.equal(date.getFullYear(), 2012)
              // The tags are not included in the results, but very likely it is
              // working correctly given all of the other tests.
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
      it('should return list of tags in JSON format', function (done) {
        request(app)
          .get('/api/tags')
          .expect('Content-Type', /json/)
          .expect(200)
          .expect(/"tag":"huddle"/)
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('locations', function () {
      it('should return list of locations in JSON format', function (done) {
        request(app)
          .get('/api/locations')
          .expect('Content-Type', /json/)
          .expect(200)
          .expect(/"location":"osaka"/)
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('years', function () {
      it('should return list of years in JSON format', function (done) {
        request(app)
          .get('/api/years')
          .expect('Content-Type', /json/)
          .expect(200)
          .expect(/"year":2014/)
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
  // pagination tests
  //
  describe('Pagination', function () {
    before(async function () {
      await backend.reinitDatabase()
      const userList = ['akemi', 'chise', 'homura', 'kyoko', 'madoka', 'midori', 'sayaka']
      for (let n = 0; n < 16; n++) {
        // The file date will be used to cause the results to appear in the
        // desired order, making the pagination easier to test (by looking at
        // the file name which is cheaper than parsing dates again).
        const fileName = `IMG_${1000 + n}.JPG`
        const fileOwner = sampleOne(userList)
        // produce identifiers that have decent entropy and distribution
        const id = pouchCollate.toIndexableString([fileName, fileOwner])
        let doc = {
          _id: id,
          file_date: [2000 + n, 5, 13, 5, 26],
          file_name: fileName,
          import_date: [2017, 11, 18, 17, 3],
          file_owner: fileOwner,
          file_size: 1048576,
          location: 'kamakura',
          mimetype: 'image/jpeg',
          tags: ['cat']
        }
        await backend.updateDocument(doc)
      }
      // Prime the indices so the tests appear to run faster and do not show
      // duration values in red, which looks bad.
      await backend.byTags(['foobar'])
    })

    describe('page size of 6, default page 1', function () {
      it('should return 6 assets', function (done) {
        request(app)
          .get('/api/assets')
          .query({
            'tags[]': ['cat'],
            'page_size': 6
          })
          .expect('Content-Type', /json/)
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.count, 16)
            assert.isNotEmpty(res.body.assets)
            assert.equal(res.body.assets.length, 6)
            assert.equal(res.body.assets[0].file_name, 'IMG_1015.JPG')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('page size of 6, page 2', function () {
      it('should return 6 assets', function (done) {
        request(app)
          .get('/api/assets')
          .query({
            'tags[]': ['cat'],
            'page_size': 6,
            'page': 2
          })
          .expect('Content-Type', /json/)
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.count, 16)
            assert.isNotEmpty(res.body.assets)
            assert.equal(res.body.assets.length, 6)
            assert.equal(res.body.assets[0].file_name, 'IMG_1009.JPG')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('page size of 6, last page', function () {
      it('should return fewer than page_size assets', function (done) {
        request(app)
          .get('/api/assets')
          .query({
            'tags[]': ['cat'],
            'page_size': 6,
            'page': 3
          })
          .expect('Content-Type', /json/)
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.count, 16)
            assert.isNotEmpty(res.body.assets)
            assert.equal(res.body.assets.length, 4)
            assert.equal(res.body.assets[3].file_name, 'IMG_1000.JPG')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('page size of 6, out of range page', function () {
      it('should return assets for last page', function (done) {
        request(app)
          .get('/api/assets')
          .query({
            'tags[]': ['cat'],
            'page_size': 6,
            'page': 10
          })
          .expect('Content-Type', /json/)
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.count, 16)
            assert.isNotEmpty(res.body.assets)
            assert.equal(res.body.assets.length, 4)
            assert.equal(res.body.assets[3].file_name, 'IMG_1000.JPG')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('page size of 500', function () {
      it('should return all 16 assets', function (done) {
        request(app)
          .get('/api/assets')
          .query({
            'tags[]': ['cat'],
            'page_size': 500
          })
          .expect('Content-Type', /json/)
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.count, 16)
            assert.isNotEmpty(res.body.assets)
            assert.equal(res.body.assets.length, 16)
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
  // fetching assets by identifier
  //
  describe('Asset retrieval', function () {
    const docId = '37665f499b5ddb74ddc297e89dfad4f06a6c8a90'

    before(async function () {
      await backend.reinitDatabase()
      let doc = {
        _id: docId,
        file_date: [2017, 5, 13, 5, 26],
        file_name: 'IMG_1001.JPG',
        import_date: [2017, 11, 18, 17, 3],
        file_owner: 'homura',
        file_size: 1048576,
        location: 'kyoto',
        mimetype: 'image/jpeg',
        tags: ['puella', 'magi', 'madoka', 'magica']
      }
      await backend.updateDocument(doc)
    })

    describe('no such asset', function () {
      it('should return an error', function (done) {
        request(app)
          .get('/api/assets/nosuch')
          .expect('Content-Type', /text/)
          .expect(404)
          .expect(/missing/)
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })
    })

    describe('asset by correct identifier', function () {
      it('should return all asset details', function (done) {
        request(app)
          .get(`/api/assets/${docId}`)
          .expect('Content-Type', /json/)
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.file_name, 'IMG_1001.JPG')
            assert.equal(res.body.file_owner, 'homura')
            assert.equal(res.body.file_size, 1048576)
            assert.equal(res.body.location, 'kyoto')
            assert.equal(res.body.mimetype, 'image/jpeg')
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
          .get(`/api/assets/${docId}`)
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.file_name, 'dcp_1069.jpg')
            assert.equal(res.body.mimetype, 'image/jpeg')
            let date = new Date(res.body.datetime)
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
          .put(`/api/assets/${docId}`)
          .send({
            location: 'hawaii',
            caption: 'a mild mannered cow',
            tags: 'cow,fence,grass'
          })
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.status, 'success')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should serve the updated values', function (done) {
        request(app)
          .get(`/api/assets/${docId}`)
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.caption, 'a mild mannered cow')
            assert.equal(res.body.location, 'hawaii')
            assert.equal(res.body.tags[0], 'cow')
            assert.equal(res.body.tags[1], 'fence')
            assert.equal(res.body.tags[2], 'grass')
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
          .put(`/api/assets/${docId}`)
          .send({user_date: '2003-08-30'})
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.status, 'success')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should serve the custom user date', function (done) {
        request(app)
          .get(`/api/assets/${docId}`)
          .expect(200)
          .expect((res) => {
            let date = new Date(res.body.user_date)
            assert.equal(date.getFullYear(), 2003)
            assert.equal(date.getMonth() + 1, 8)
            assert.equal(date.getDate(), 30)
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
          .put(`/api/assets/${docId}`)
          .send({user_date: null})
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.status, 'success')
          })
          .end(function (err, res) {
            if (err) {
              return done(err)
            }
            done()
          })
      })

      it('should serve the cleared user date', function (done) {
        request(app)
          .get(`/api/assets/${docId}`)
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.user_date, '')
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

  describe('Asset content serving', function () {
    const docId = 'dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07'

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
          .get(`/api/assets/${docId}`)
          .expect(200)
          .expect((res) => {
            assert.equal(res.body.file_name, '100_1206.MOV')
            assert.equal(res.body.mimetype, 'video/quicktime')
            let date = new Date(res.body.datetime)
            assert.equal(date.getFullYear(), 2007)
            assert.equal(date.getMonth() + 1, 9)
            assert.equal(date.getDate(), 14)
            assert.oneOf(date.getHours(), [5, 12])
            assert.equal(date.getMinutes(), 7)
            assert.approximately(res.body.duration, 2, 0.5)
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
