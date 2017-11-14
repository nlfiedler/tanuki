/* eslint-env mocha */
const request = require('supertest')

// start the server
const app = require('../app.js')

//
// attributes tests (tags, locations, etc)
//
describe('Attributes', function () {
  describe('tags', function () {
    it('should return list of tags in JSON format', function (done) {
      request(app)
        .get('/api/tags')
        .expect('Content-Type', /json/)
        .expect(200)
        .expect(/puppy/)
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
        .expect(/hawaii/)
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
        .expect(/1994/)
        .end(function (err, res) {
          if (err) {
            return done(err)
          }
          done()
        })
    })
  })
})
