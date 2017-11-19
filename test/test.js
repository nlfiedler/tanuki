const assert = require('chai').assert
const request = require('supertest')
const fs = require('fs-extra')
const config = require('config')
const pouchCollate = require('pouchdb-collate')

/* eslint-env mocha */

// clean up from previous test runs before starting the server
const dbPath = config.get('backend.dbPath')
fs.removeSync(dbPath)

// start the server
const app = require('../app.js')
const backend = require('backend')

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
          .expect(/"filename":"img0315.jpg"/)
          .expect(/"location":"san francisco"/)
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
          .expect(/{"value":1,"key":"cheeseburger"}/)
          .expect(/{"value":2,"key":"picnic"}/)
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
          .expect(/{"value":2,"key":"san francisco"}/)
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
          .expect(/{"value":2,"key":2014}/)
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
          tags: [
            sampleOne(tagList1),
            sampleOne(tagList2),
            sampleOne(tagList3)
          ]
        }
        await backend.updateDocument(doc)
      }
      // Prime the indices so the tests appear to run faster and do not show
      // duration values in red, which looks bad.
      await backend.allLocations()
      await backend.allTags()
      await backend.allYears()
      await backend.byTags(['foobar'])
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

    describe('assets by a tag', function () {
      // With async/await let's go directly against the backend.
      it('should return list of matching assets', async function () {
        let rows = await backend.byTags(['dipstick'])
        for (let row of rows) {
          let doc = await backend.fetchDocument(row.id)
          assert.include(doc.tags, 'dipstick')
        }
      })
    })

    // TODO: perform asset queries with years, locations, and tags
    // TODO: ensure results contain the expected years, locations, tags
    // TODO: test pagination

    describe('tags', function () {
      it('should return list of tags in JSON format', function (done) {
        request(app)
          .get('/api/tags')
          .expect('Content-Type', /json/)
          .expect(200)
          .expect(/"key":"huddle"/)
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
          .expect(/"key":"osaka"/)
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
          .expect(/"key":2014/)
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
