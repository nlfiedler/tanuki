//
// Copyright (c) 2026 Nathan Fiedler
//
import { createReadStream } from 'node:fs';
import { createInterface } from 'node:readline';

//
// Migrate asset identifiers from Base64-encoded to Base64URL-encoded.
//
// 1. dump the database to a JSON file
// 2. run the output through this script to a new file
// 3. stop the database
// 4. move the database aside
// 5. start the database
// 6. load the modified dump file
//
const userArgs = process.argv.slice(2);
if (userArgs.length === 0) {
  console.error('Usage: bun base64url.ts dump.json');
} else {
  const fileStream = createReadStream(userArgs[0]!);
  const rl = createInterface({
    input: fileStream,
    crlfDelay: Infinity
  });
  for await (const line of rl) {
    if (line.trim().length > 0) {
      const entry = JSON.parse(line);
      entry.key = Buffer.from(entry.key, 'base64').toString('base64url');
      console.info(JSON.stringify(entry));
    }
  }
}
