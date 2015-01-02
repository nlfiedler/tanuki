#!/usr/bin/env hy
; -*- coding: utf-8 -*-
; -------------------------------------------------------------------
;
; Copyright (c) 2014 Nathan Fiedler
;
; This file is provided to you under the Apache License,
; Version 2.0 (the "License"); you may not use this file
; except in compliance with the License. You may obtain
; a copy of the License at
;
; http://www.apache.org/licenses/LICENSE-2.0
;
; Unless required by applicable law or agreed to in writing,
; software distributed under the License is distributed on an
; "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
; KIND, either express or implied. See the License for the
; specific language governing permissions and limitations
; under the License.
;
; -------------------------------------------------------------------
;
; Process files in an 'incoming' folder by adding them to CouchDB.
;
; This script, which is intended to be run via a cron job, examines the
; contents of a particular directory, looking for content to add to CouchDB.
; The parent directories of any files found are used to assign tags to the
; blobs once the files are stored in CouchDB.
;
; See the project home page for details: https://github.com/nlfiedler/tanuki
;
; Prerequisites:
; * couchdb -- https://github.com/djc/couchdb-python
; * exifread -- https://github.com/ianare/exif-py
;

(import argparse)
(import [datetime [datetime]])
(import [email.mime.text [MIMEText]])
(import [functools [partial]])
(import hashlib)
(import mimetypes)
(import os)
(import pwd)
(import re)
(import smtplib)
(import socket)
(import sys)
(import tempfile)

(import couchdb)
(import exifread)

(setv *db-name* "tanuki")
(setv *exif-datetime* "EXIF DateTimeOriginal")
(setv *datetime-format* "%Y-%m-%d %H:%M")
(setv *extraneous-files* [".DS_Store" ".localized"])
(setv *date-regex* (re.compile "(\\d{4})-(\\d{2})-(\\d{2}) (\\d{2}):(\\d{2})"))
; *logger* is a partially applied 'print' function to generate an email report,
; invoke in the same manner as the Python built-in print
(setv *logger* None)

(defn connect-couch []
  ;
  ; Connect to CouchDB or die trying.
  ;
  ; Returns the couchdb.Server instance.
  ;
  (setv couch (couchdb.Server))
  (try
    (couch.version)
    (catch [se socket.error]
      (do
        (sys.stderr.write "Unable to connect to CouchDB at default location.")
        (sys.exit 1))))
  couch)

(defn create-database [couch]
  ;
  ; Create the database for our assets if not already present.
  ;
  ; :param couch: couchdb.Server instance.
  ;
  ; Returns the couchdb.Database instance.
  ;
  (if (not (in *db-name* couch))
    (couch.create *db-name*)
    (get couch *db-name*)))

(defn path-old-enough? [dirpath]
  ;
  ; Check if the given directory is sufficiently old (at least 1 hour).
  ;
  ; :param dirpath: path to incoming assets.
  ;
  (setv now (datetime.now))
  (setv d_time (datetime.fromtimestamp (. (os.stat dirpath) st_ctime)))
  (> (. (- now d_time) seconds) 3600))

(defn process-path [dirpath db destpath]
  ;
  ; Import the assets found with the named path.
  ;
  ; :param dirpath: path to incoming assets.
  ; :param db: instance of couchdb.Database.
  ; :param destpath: path to asset storage area.
  ;
  (setv utcnow (datetime.utcnow))
  (setv importdate (date-string-to-ints-list (utcnow.strftime *datetime-format*)))
  (setv asset_folder (.lower (os.path.basename (os.path.normpath dirpath))))
  (setv parts (.split asset_folder "@" 1))
  (if (> (len parts) 1)
    (do
      (setv asset_folder (get parts 0))
      (setv location (.replace (get parts 1) "_" " ")))
    (setv location None))
  (setv tags (list-comp tag [tag (.split asset_folder "_")] (!= tag "")))
  (*logger* (.format "--- Processing tags: {}\n" (.join ", " tags)))
  (for [entry (os.listdir dirpath)]
    (setv filepath (os.path.join dirpath entry))
    (cond [(in entry *extraneous-files*)
            ; Remove the superfluous files that Mac likes to create.
            (os.unlink filepath)]
          [(= (get entry 0) ".")
            ; Ignore all other hidden entries.
            (continue)]
          [(os.path.isfile filepath)
            (do
              (setv checksum (compute-checksum filepath))
              (setv doc {})
              (assoc doc "exif_date" (date-string-to-ints-list (get-original-date filepath)))
              (assoc doc "file_date" (date-string-to-ints-list (file-date filepath)))
              (assoc doc "file_name" entry)
              (assoc doc "file_owner" (file-owner filepath))
              (assoc doc "file_size" (. (os.stat filepath) st_size))
              (assoc doc "import_date" importdate)
              (assoc doc "location" location)
              (setv mimetype (.guess_type mimetypes filepath))
              (assoc doc "mimetype" (if mimetype (get mimetype 0) None))
              (assoc doc "sha256" checksum)
              (assoc doc "tags" tags)
              (setv (, doc_id doc_rev) (db.save doc))
              (*logger* (.format "{} => id={}, rev={}" entry doc_id doc_rev))
              (store-asset filepath checksum destpath))]
          [True
            (*logger* (.format "Ignoring non-file {}\n" entry))]))
  (*logger* (.format "\n--- done with {}\n" dirpath))
  (if (= (len (os.listdir dirpath)) 0)
    (os.rmdir dirpath)
    (*logger* (.format "Unable to remove non-empty directory: {}\n" asset_folder))))

(defn compute-checksum [filepath]
  ;
  ; Compute the SHA256 for the named file.
  ;
  ; :param filepath: path to file to be read.
  ;
  ; Returns the hex string of the computed checksum.
  ;
  (setv h (hashlib.new "sha256"))
  (with [[fobj (open filepath "rb")]]
    (while True
      (setv data (fobj.read 4096))
      (if (not data)
        (break))
      (h.update data)))
  (h.hexdigest))

(defn file-owner [filepath]
  ;
  ; Return the username of the file owner.
  ;
  ; :param filepath: path to file to be examined.
  ;
  ; Returns the username of the file owner, of None if not available.
  ;
  (setv p (pwd.getpwuid (. (os.stat filepath) st_uid)))
  (if p
    (get p 0)
    None))

(defn file-date [filepath]
  ;
  ; Return the mtime of the file, which is typically when it was created.
  ;
  ; :param filepath: path to file to be examined.
  ;
  (setv d (datetime.utcfromtimestamp (. (os.stat filepath) st_mtime)))
  (.strftime d *datetime-format*))

(defn get-original-date [filepath]
  ;
  ; Attempt to read the original datetime from the EXIF tags.
  ;
  ; :param filepath: path to file to be read.
  ;
  ; Returns a datetime string (e.g. "2005-08-21 19:19") or None.
  ;
  (setv value None)
  (with [[fobj (open filepath "rb")]]
    (setv tags (exifread.process_file fobj))
    (if (in *exif-datetime* tags)
      (do
        (setv value (. (get tags *exif-datetime*) values))
        (setv date (datetime.strptime value "%Y:%m:%d %H:%M:%S"))
        (setv value (datetime.strftime date *datetime-format*)))))
  value)

(defn store-asset [filepath checksum destpath]
  ;
  ; Move the named asset to its sharded location.
  ;
  ; :param filepath: path to file to be read.
  ; :param checksum: the checksum of the file, becomes its new name and path.
  ; :param destpath: path to the storage area.
  ;
  ; If an existing asset with the same checksum already exists, the new
  ; asset will be ignored.
  ;
  (setv dirname (os.path.join destpath (slice checksum 0 2) (slice checksum 2 4)))
  (if (not (os.path.exists dirname))
    (os.makedirs dirname))
  (setv newpath (os.path.join dirname (slice checksum 4)))
  (if (not (os.path.exists newpath))
    (os.rename filepath newpath)))

(defn date-string-to-ints-list [value]
  ;
  ; Convert the date string into a list of ints.
  ;
  ; Example: "2014-07-04 12:01" -> [2014, 7, 4, 12, 1]
  ;
  ; :type value: str or None
  ; :param value: date string to convert
  ;
  ; :return: list of ints, or None if value does not match date format.
  ;
  (setv match (if value (.match *date-regex* value) None))
  (if match
    (list-comp (int part) [part (match.groups)])
    None))

(defn email-report [filename]
  ;
  ; Send the text in the named file via email to the root user.
  ; Failing that, dump the content to stdout.
  ;
  (with [[fobj (open filename)]]
    (setv content (.read fobj)))
  (if (not (empty? content))
    (do
      (setv msg (MIMEText content))
      (assoc msg "Subject" "incoming processor report")
      (setv me "tanuki")
      (setv you "root")
      (assoc msg "From" me)
      (assoc msg "To" you)
      (try
        (do
          (setv server (smtplib.SMTP "localhost"))
          (.sendmail server me [you] (.as_string msg))
          (server.quit))
        (catch [e Exception]
          (print content))))))

(defn revert-all [couch frompath topath]
  ;
  ; Put everything back the way it was.
  ;
  ; :param couch: instance of couchdb.Server.
  ; :param frompath: path of stored assets.
  ; :param topath: where to put the files with their original names.
  ;
  ; Does not handle filename conflicts. This is only intended for reversing
  ; an import that was performed while testing. Presumably the database
  ; contains only a small number of documents and thus file names would not
  ; be in conflict.
  ;
  (setv db (get couch *db-name*))
  (for [row (db.view "_all_docs")]
    (setv doc (db.get row.id))
    (setv filename (get doc "file_name"))
    (setv sha256 (get doc "sha256"))
    (setv fp (os.path.join frompath
                (slice sha256 0 2)
                (slice sha256 2 4)
                (slice sha256 4)))
    (setv tp (os.path.join topath filename))
    (os.rename fp tp)
    (print (.format "{} => {}" filename topath))
    (db.delete doc)))

(defmain [&rest args]
  ;
  ; Process incoming assets according to provided arguments.
  ;
  (setv parser (apply argparse.ArgumentParser []
                {"description" "Import assets to tanuki."}))
  (apply .add_argument
    [parser "-p" "--path"]
    {"required" True
     "help" "base path of incoming assets"})
  (apply .add_argument
    [parser "-d" "--dest"]
    {"required" True
     "help" "base path of asset storage area"})
  (apply .add_argument
    [parser "-N" "--now"]
    {"action" "store_true"
     "help" "consider all paths, even very new ones"})
  (apply .add_argument
    [parser "-r" "--revert"]
    {"action" "store_true"
     "help" "extract everything from CouchDB (oops!)"})
  (setv args (parser.parse_args))
  (if (not (os.path.exists args.path))
    (do
      (sys.stderr.write (.format "No such directory: {}" args.path))
      (sys.exit 1)))
  (setv couch (connect-couch))
  (if args.revert
    (revert-all couch args.dest args.path)
    (do
      (setv db (create-database couch))
      (setv (, report_fd report_file) (tempfile.mkstemp))
      (with [[fobj (open report_fd "w")]]
        (global *logger*)
        (setv *logger* (apply partial [print] {"file" fobj}))
        (for [entry (os.listdir args.path)]
          (setv fullpath (os.path.join args.path entry))
          (cond [(os.path.isfile fullpath)
                  (if (in entry *extraneous-files*)
                    ; Remove the superfluous files that Mac OS X likes to create.
                    (os.unlink fullpath)
                    (*logger* (.format "Ignoring file outside of tagged folder: {}\n" entry)))]
                [(os.path.isdir fullpath)
                  (if (or args.now (path-old-enough? fullpath))
                    (process-path fullpath db args.dest)
                    (*logger* (.format "Ignoring new path {} for now...\n" entry)))]
                [True
                  (*logger* (.format "Ignoring mystery object {}\n" entry))])))
      (email-report report_file)
      (os.unlink report_file))))
