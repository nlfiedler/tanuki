//
// Copyright (c) 2018 Nathan Fiedler
//
const {assert} = require('chai')
const {before, describe, it, run} = require('mocha')
const fs = require('fs-extra')
const config = require('config')
const path = require('path')
const PouchDB = require('pouchdb')

// clean up from previous test runs before starting the server
const dbPath = config.get('backend.dbPath')
fs.removeSync(dbPath)
const db = new PouchDB(dbPath)
const assetsPath = config.get('backend.assetPath')
fs.emptyDirSync(assetsPath)

// this test does not need the server, but loading it sets the module path
// and configures the logging for testing
require('../app.js')
const assets = require('lib/assets')
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
      const filename = 'IMG_1001.JPG'
      const importDate = Date.UTC(2017, 10, 18, 17, 3)
      const docId = assets.makeAssetId(importDate, filename)

      before(async function () {
        await backend.reinitDatabase()
        let doc = {
          _id: docId,
          filename,
          import_date: importDate,
          filesize: 1048576,
          location: 'kyoto',
          mimetype: 'image/jpeg',
          sha256: '938f831fb02b313e7317c1e0631b86108a9e4a197e33d581fb68be91a3c6ce2f',
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
      const oldDocId = '4f86f7dd48474b8e6571beeabbd79111267f143c0786bcd45def0f6b33ae0423'
      const ap = path.join(assetsPath, oldDocId.slice(0, 2), oldDocId.slice(2, 4), oldDocId.slice(4))
      const oldpath = path.isAbsolute(ap) ? ap : path.join(process.cwd(), ap)

      before(async function () {
        await backend.reinitDatabase()
        let doc = {
          _id: oldDocId,
          sha256: oldDocId,
          file_name: '100_1206.MOV',
          file_date: [2017, 10, 18, 17, 3],
          import_date: [2017, 10, 18, 17, 3],
          file_size: 311139,
          file_owner: 'chihiro',
          location: 'kyoto',
          mimetype: 'video/quicktime',
          exif_date: [2017, 10, 18, 17, 3],
          tags: ['puella', 'magi', 'madoka', 'magica']
        }
        await backend.updateDocument(doc)
        // put the video file and a fake thumbnail in the old asset location
        fs.ensureDirSync(path.dirname(oldpath))
        fs.copyFileSync('./test/fixtures/100_1206.MOV', oldpath)
        fs.copyFileSync('./test/fixtures/fighting_kittens.jpg', oldpath + '.jpg')
      })

      it('should rename, remove, and format fields', async function () {
        assert.isTrue(fs.existsSync(oldpath))
        let index = await db.get('_design/assets')
        let assetBefore = await db.get(oldDocId)
        let ok = await migrate.migrate(db, 0, index.version)
        assert.isTrue(ok, 'migrate() returned true')
        const newDocId = await backend.byChecksum(oldDocId)
        let assetAfter = await db.get(newDocId)
        // lots of changes
        assert.notEqual(assetAfter._id, assetBefore._id, 'document identifier changed')
        assert.property(assetAfter, 'filesize', 'filesize property defined')
        assert.notProperty(assetAfter, 'file_size', 'file_size property removed')
        assert.notProperty(assetAfter, 'file_date', 'file_date property removed')
        assert.notProperty(assetAfter, 'file_owner', 'file_owner property removed')
        assert.notProperty(assetAfter, 'exif_date', 'exif_date property removed')
        assert.property(assetAfter, 'sha256', 'sha256 property back again')
        assert.property(assetAfter, 'original_date', 'original_date property defined')
        assert.isNumber(assetAfter.import_date, 'import date changed to number')
        assert.isNumber(assetAfter.original_date, 'original date changed to number')
        // ensure the old parent directories are gone
        assert.isFalse(fs.existsSync(path.dirname(oldpath)))
        assert.isFalse(fs.existsSync(path.dirname(path.dirname(oldpath))))
        // ensure new file and thumbnail are where expected
        const newpath = assets.assetPath(newDocId)
        assert.isTrue(fs.existsSync(newpath))
        const dirname = path.dirname(newpath)
        const basename = path.basename(newpath, path.extname(newpath))
        const newthumb = path.join(dirname, basename + '.jpg')
        assert.isTrue(fs.existsSync(newthumb))
      })
    })
  })

  run()
}, 500)
