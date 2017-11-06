/* eslint-env mocha */
const assert = require('chai').assert
const request = require('request')

// start the server
require('../app.js')

//
// attributes tests (tags, locations, etc)
//
describe('Attributes', function () {
  describe('tags', function () {
    it('should return list of tags in JSON foramt', function () {
      request.get('http://localhost:' + process.env.PORT + '/api/tags', function (err, res, body) {
        assert.isNull(err)
        assert.equal(res.statusCode, 200)
        assert.equal(body, '{"tag":"cat","count":2},{"tag":"dog","count":5},{"tag":"kitten","count":100},{"tag":"puppy","count":40}')
      })
    })
  })
  describe('locations', function () {
    it('should return list of locations in JSON foramt', function () {
      request.get('http://localhost:' + process.env.PORT + '/api/locations', function (err, res, body) {
        assert.isNull(err)
        assert.equal(res.statusCode, 200)
        assert.equal(body, '{"location":"dublin","count":6},{"location":"hawaii","count":4},{"location":"piedmont","count":5},{"location":"vancouver","count":7}')
      })
    })
  })
  describe('years', function () {
    it('should return list of years in JSON foramt', function () {
      request.get('http://localhost:' + process.env.PORT + '/api/years', function (err, res, body) {
        assert.isNull(err)
        assert.equal(res.statusCode, 200)
        assert.equal(body, '{"year":1986,"count":1},{"year":1992,"count":1},{"year":1994,"count":1},{"year":2002,"count":9}')
      })
    })
  })
})
