//
// Copyright (c) 2026 Nathan Fiedler
//
const port = process.env.APPLICATION_PORT || 3000;
const path = process.env.HEALTHCHECK_PATH || '/';
const url = `http://localhost:${port}${path}`;

/* eslint-disable unicorn/no-process-exit */

try {
  const response = await fetch(url, { method: 'HEAD' });
  if (response.status > 199 && response.status < 399) {
    process.exit(0);
  } else {
    console.error(`healthcheck failed with status: ${response.status}`);
    process.exit(1);
  }
} catch (error: any) {
  console.error('healthcheck failed to connect:', error.message);
  process.exit(1);
}
