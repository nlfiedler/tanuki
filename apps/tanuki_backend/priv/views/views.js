// -*- coding: utf-8 -*-
// -------------------------------------------------------------------
//
// Copyright (c) 2014 Nathan Fiedler
//
// This file is provided to you under the Apache License,
// Version 2.0 (the "License"); you may not use this file
// except in compliance with the License. You may obtain
// a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. See the License for the
// specific language governing permissions and limitations
// under the License.
//
// -------------------------------------------------------------------
//
// View map functions for our assets design document. Squeeze them into a single
// line and paste into the design_assets.json document.
//

// reduce: none
var by_date_map = function (doc) {
	if (doc.exif_date) {
		emit(doc.exif_date, null);
	} else if (doc.file_date) {
		emit(doc.file_date, null);
	} else {
		emit(doc.import_date, null);
	}
};

// reduce: none
var by_tag_map = function (doc) {
	if (doc.tags && Array.isArray(doc.tags)) {
        doc.tags.forEach(function (tag) {
            emit(tag.toLowerCase(), null);
	    });
	}
};

// reduce: _count
var tags_map = function (doc) {
    if (doc.tags && Array.isArray(doc.tags)) {
        doc.tags.forEach(function (tag) {
            emit(tag.toLowerCase(), 1);
        });
    }
};
