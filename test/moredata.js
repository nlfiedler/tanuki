//
// Copyright (c) 2018 Nathan Fiedler
//
const { assert } = require('chai')
const { before, describe, it, run } = require('mocha')
const request = require('supertest')

// start the server, which also modifies the module path
const app = require('app.js')
const assets = require('lib/assets')
const backend = require('lib/backend')

//
// Give the backend a chance to initialize the database asynchronously.
// A timeout of zero is not sufficient, so this timing is fragile.
// A better solution is desired.
//
setTimeout(function () {
  //
  // attributes tests (tags, locations, etc) with ample data
  //
  describe('Attributes with more data', function () {
    before(async function () {
      await backend.reinitDatabase()
      const tagList1 = ['cat', 'catastrophe', 'cheddar', 'cheese', 'cheeseburger', 'cuddle', 'cutler']
      const tagList2 = ['diddle', 'dig', 'dipstick', 'dog', 'dogmatic', 'dug', 'duster']
      const tagList3 = ['hag', 'haggle', 'hamster', 'hid', 'hot', 'huckster', 'huddle']
      const locations = ['kamakura', 'kanazawa', 'kyoto', 'osaka', 'sapporo', 'tokyo', 'yokohama']
      const importDate = Date.UTC(2017, 10, 18, 17, 3)
      for (let n = 0; n < 49; n++) {
        const filename = `IMG_${1000 + n}.JPG`
        const id = assets.makeAssetId(importDate, filename)
        const year = Math.floor(Math.random() * 7) + 2010
        let doc = {
          _id: id,
          // we only search on the year (for now), the rest is meaningless
          original_date: Date.UTC(year, 4, 13, 5, 26),
          filename,
          // original date overrides import date in terms of significance,
          // so anything at all is fine here
          import_date: importDate,
          filesize: Math.floor(Math.random() * 1048576) + 1048576,
          location: locations[n % locations.length],
          mimetype: 'image/jpeg',
          tags: [
            tagList1[n % tagList1.length],
            tagList2[n % tagList2.length],
            tagList3[n % tagList3.length]
          ]
        }
        if (n === 5) {
          // make one with several attributes that we can check for later
          doc.original_date = Date.UTC(2012, 4, 13, 5, 26)
          doc.location = 'osaka'
          doc.tags = ['cat', 'dog', 'hot']
        } else if (n === 10) {
          // some queries rely on multiple year and location values
          doc.original_date = Date.UTC(2013, 4, 13, 5, 26)
          doc.location = 'kyoto'
        }
        await backend.updateDocument(doc)
      }
    })

    describe('assets', function () {
      it('should return a large count', function (done) {
        request(app)
          .post('/graphql')
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
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "tags": "not_in_here"
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
        let rows = await backend.query({ tags: ['dipstick'] })
        assert.isNotEmpty(rows)
        for (let row of rows) {
          let doc = await backend.fetchDocument(row.id)
          assert.include(doc.tags, 'dipstick')
        }
      })
    })

    describe('assets by multiple tags', function () {
      // With async/await let's go directly against the backend.
      it('should return list of matching assets', async function () {
        let rows = await backend.query({ tags: ['cat', 'dog', 'hot'] })
        assert.isNotEmpty(rows)
        for (let row of rows) {
          let doc = await backend.fetchDocument(row.id)
          assert.include(doc.tags, 'cat')
          assert.include(doc.tags, 'dog')
          assert.include(doc.tags, 'hot')
        }
      })
    })

    describe('assets by date range', function () {
      it('should return list of matching assets', function (done) {
        const afterTime = new Date(2012, 0).getTime()
        const beforeTime = new Date(2013, 0).getTime()
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "after": ${afterTime},
                "before": ${beforeTime}
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  datetime
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
              assert.isNotNull(row)
              let date = new Date(row.datetime)
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

    // strangely this does not result in an error
    // describe('assets with an invalid date range', function () {
    //   it('should return an error', function (done) {
    //     request(app)
    //       .post('/graphql')
    //       .send({
    //         query: `query {
    //           search(after: "foo") {
    //             results {
    //               datetime
    //             }
    //             count
    //           }
    //         }`
    //       })
    //       .expect(400)
    //       .expect(/Expected type Int/)
    //       .end(function (err, res) {
    //         if (err) {
    //           return done(err)
    //         }
    //         done()
    //       })
    //   })
    // })

    describe('assets by date range spanning multiple years', function () {
      it('should return list of matching assets', function (done) {
        const afterTime = new Date(2012, 0).getTime()
        const beforeTime = new Date(2014, 0).getTime()
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "after": ${afterTime},
                "before": ${beforeTime}
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  datetime
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
              assert.isNotNull(row)
              let date = new Date(row.datetime)
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
      it('should return list of matching assets', function (done) {
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "locations": ["osaka"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
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
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "locations": ["kyoto", "osaka"]
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
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

    describe('assets by tags, location, and date range', function () {
      it('should return list of matching assets', function (done) {
        const afterTime = new Date(2012, 0).getTime()
        const beforeTime = new Date(2013, 0).getTime()
        request(app)
          .post('/graphql')
          .send({
            variables: `{
              "params": {
                "tags": ["cat", "hot"],
                "locations": ["osaka"],
                "after": ${afterTime},
                "before": ${beforeTime}
              }
            }`,
            operationName: 'Search',
            query: `query Search($params: SearchParams!) {
              search(params: $params) {
                results {
                  datetime
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
              assert.isNotNull(row)
              assert.equal(row.location, 'osaka')
              let date = new Date(row.datetime)
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
          .post('/graphql')
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
          .post('/graphql')
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
          .post('/graphql')
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
