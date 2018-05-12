//
// Copyright (c) 2018 Nathan Fiedler
//
const {assert} = require('chai')
const {before, describe, it, run} = require('mocha')
const fs = require('fs-extra')
const config = require('config')
const PouchDB = require('pouchdb')

// clean up from previous test runs before starting the server
const dbPath = config.get('backend.dbPath')
fs.removeSync(dbPath)
const db = new PouchDB(dbPath)

// this test does not need the server, but loading it sets the module path
// and configures the logging for testing
require('../app.js')
const backend = require('lib/backend')
const migrate = require('lib/migrate')

//
// Give the backend a chance to initialize the database asynchronously.
// A timeout of zero is not sufficient, so this timing is fragile.
// A better solution is desired.
//
setTimeout(function () {
  describe('Database migration', function () {
    describe('migration of freshly built db', function () {
      const docId = '37665f499b5ddb74ddc297e89dfad4f06a6c8a90'

      before(async function () {
        await backend.reinitDatabase()
        let doc = {
          _id: docId,
          filename: 'IMG_1001.JPG',
          import_date: Date.UTC(2017, 10, 18, 17, 3),
          filesize: 1048576,
          location: 'kyoto',
          mimetype: 'image/jpeg',
          tags: ['puella', 'magi', 'madoka', 'magica']
        }
        await backend.updateDocument(doc)
      })

      it('should not modify up-to-date documents', async function () {
        let index = await db.get('_design/assets')
        let assetBefore = await db.get(docId)
        let ok = await migrate.migrate(db, 0, index.version)
        assert.isTrue(ok, 'migrate() returned true')
        let assetAfter = await db.get(docId)
        assert.equal(assetAfter._rev, assetBefore._rev, 'doc revision unchanged')
      })

      it('should not allow version downgrade', async function () {
        let ok = await migrate.migrate(db, 5, 1)
        assert.isFalse(ok, 'migrate() rejected version downgrade')
      })
    })

    describe('migration from v3 to v4', function () {
      const docId = '37665f499b5ddb74ddc297e89dfad4f06a6c8a90'

      before(async function () {
        await backend.reinitDatabase()
        let doc = {
          _id: docId,
          filename: 'IMG_1001.JPG',
          import_date: Date.UTC(2017, 10, 18, 17, 3),
          filesize: 1048576,
          file_owner: 'chihiro',
          location: 'kyoto',
          mimetype: 'image/jpeg',
          exif_date: [2017, 10, 18, 17, 3],
          tags: ['puella', 'magi', 'madoka', 'magica']
        }
        await backend.updateDocument(doc)
      })

      it('should not change documents from v3 to v4', async function () {
        let assetBefore = await db.get(docId)
        let ok = await migrate.migrate(db, 3, 4)
        assert.isTrue(ok, 'migrate() returned true')
        let assetAfter = await db.get(docId)
        // despite the document obviously needing changes, the versions
        // specified did not require doing anything to them
        assert.equal(assetAfter._rev, assetBefore._rev, 'doc revision unchanged')
      })
    })

    describe('migration from v0 to latest', function () {
      const docId = '37665f499b5ddb74ddc297e89dfad4f06a6c8a90'

      before(async function () {
        await backend.reinitDatabase()
        let doc = {
          _id: docId,
          sha256: docId,
          file_name: 'IMG_1001.JPG',
          file_date: [2017, 10, 18, 17, 3],
          import_date: [2017, 10, 18, 17, 3],
          file_size: 1048576,
          file_owner: 'chihiro',
          location: 'kyoto',
          mimetype: 'image/jpeg',
          exif_date: [2017, 10, 18, 17, 3],
          tags: ['puella', 'magi', 'madoka', 'magica']
        }
        await backend.updateDocument(doc)
      })

      it('should rename, remove, and format fields', async function () {
        let index = await db.get('_design/assets')
        let assetBefore = await db.get(docId)
        let ok = await migrate.migrate(db, 0, index.version)
        assert.isTrue(ok, 'migrate() returned true')
        let assetAfter = await db.get(docId)
        // lots of changes
        assert.notEqual(assetAfter._rev, assetBefore._rev, 'doc revision changed')
        assert.property(assetAfter, 'filesize', 'filesize property defined')
        assert.notProperty(assetAfter, 'file_size', 'file_size property removed')
        assert.notProperty(assetAfter, 'file_date', 'file_date property removed')
        assert.notProperty(assetAfter, 'file_owner', 'file_owner property removed')
        assert.notProperty(assetAfter, 'exif_date', 'exif_date property removed')
        assert.notProperty(assetAfter, 'sha256', 'sha256 property removed')
        assert.property(assetAfter, 'original_date', 'original_date property defined')
        assert.isNumber(assetAfter.import_date, 'import date changed to number')
        assert.isNumber(assetAfter.original_date, 'original date changed to number')
      })
    })
  })

  run()
}, 500)
