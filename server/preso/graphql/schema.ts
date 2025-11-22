//
// Copyright (c) 2025 Nathan Fiedler
//
import fs from 'node:fs/promises';
import path from 'node:path';
// info parameter type: import { GraphQLResolveInfo } from 'graphql';
import container from 'tanuki/server/container.ts';
import logger from 'tanuki/server/logger.ts';

const countAssets = container.resolve('countAssets');

// The GraphQL schema
const schemaPath = path.join('public', 'schema.graphql');
export const typeDefs = await fs.readFile(schemaPath, 'utf8');

// A map of functions which return data for the schema.
export const resolvers = {
  Query: {
    // params: parent, args, contextValue, info
    async count() {
      return countAssets();
    },
    // search(params: SearchParams!, count: Int = 10, offset: Int = 0): SearchMeta!
    async search(parent: any, args: any) {
      // query Search($params: SearchParams!, $pageSize: Int, $offset: Int) {
      //   search(params: $params, count: $pageSize, offset: $offset) {
      //     results {
      //       id
      //       datetime
      //       filename
      //       location {
      //         label
      //         city
      //         region
      //       }
      //       mediaType
      //     }
      //     count
      //     lastPage
      //   }
      // }
      // {
      //   "params": {
      //     "tags": [],
      //     "locations": [],
      //      "after": "2007-12-03",
      //      "sortField": "FILENAME",
      //      "sortOrder": "DESCENDING"
      //   },
      //   "pageSize": 10,
      //   "offset": 0,
      // }

      logger.info('parent: %o', parent);
      logger.info('args: %o', args);
      // info: parent: undefined
      // info: args: {
      //   params: {
      //     tags: [ [length]: 0 ],
      //     locations: [ [length]: 0 ],
      //     after: '2007-12-03',
      //     sortField: 'FILENAME',
      //     sortOrder: 'DESCENDING'
      //   }
      //   count: 10,
      //   offset: 0
      // }

      // TODO: how to convert incoming parameters to JS/TS objects? (how does Apollo help?)

      // const rows = await backend.query(args.params);
      // // sort by date by default
      // rows.sort((a, b) => b.datetime - a.datetime);
      // const totalCount = rows.length;
      // const count = boundedIntValue(args.count, 10, 1, 10000);
      // const offset = boundedIntValue(args.offset, 0, 0, totalCount);
      // const pageRows = rows.slice(offset, offset + count);
      // // decorate the results with information about the thumbnails
      // const thumbRows = [];
      // for (const elem of pageRows) {
      //   const dims = await thumbs.getSize(elem.id);
      //   thumbRows.push({
      //     ...elem,
      //     thumbnailUrl: `/thumbnail/${elem.id}`,
      //     widethumbUrl: `/widethumb/${elem.id}`,
      //     thumbWidth: dims.width,
      //     thumbHeight: dims.height
      //   });
      // }
      // return {
      //   results: thumbRows,
      //   count: totalCount
      // }
    },
  },
};
