//
// Copyright (c) 2025 Nathan Fiedler
//
import { existsSync } from 'node:fs';
import dotenv from 'dotenv';

// Importing this module before other application modules will ensure that the
// environment is set for testing, despite using the early-binding import.

// n.b. Bun will read bunfig.toml which overrides anything dotenv does here
// n.b. Bun will read <root>/.env before everything, including bunfig.toml

// Override any existing .env file by loading the test configuration.
if (existsSync('test/.env')) {
  dotenv.config({ quiet: true, path: 'test/.env' });
} else {
  throw new Error('Must define test/.env before testing.');
}
