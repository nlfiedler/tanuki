//
// Copyright (c) 2025 Nathan Fiedler
//
import http from 'node:http';
import express from 'express';
import morgan from 'morgan';
import helmet from 'helmet';
import ViteExpress from 'vite-express';
import { ApolloServer } from '@apollo/server';
import { expressMiddleware } from '@as-integrations/express5';
import { ApolloServerPluginDrainHttpServer } from '@apollo/server/plugin/drainHttpServer';
import { ApolloServerPluginLandingPageLocalDefault } from '@apollo/server/plugin/landingPage/default';
import cors from 'cors';
// load the environment settings before anything else
import 'tanuki/server/env.ts';
import logger from 'tanuki/server/logger.ts';
import container from 'tanuki/server/container.ts';
import assetsRouter from 'tanuki/server/preso/routes/assets.ts';
import recordsRouter from 'tanuki/server/preso/routes/records.ts';
import facesRouter from 'tanuki/server/preso/routes/faces.ts';
import { typeDefs, resolvers } from 'tanuki/server/preso/graphql/schema.ts';
import {
  createMetadataLoader,
  createPeopleLoader,
  createSyntheticLoader,
  createSyntheticStatusLoader
} from 'tanuki/server/preso/graphql/metadata-loader.ts';

// (asynchronously) prepare the asset record store and the face store, then
// start the background worker pool. The pool must wait for BOTH databases —
// it calls recordRepository.getAssetById(), which would throw if the asset
// DB were still null while the face store has leftover queue rows.
const settings: any = container.resolve('settingsRepository');
const database: any = container.resolve('recordRepository');
const faceStore: any = container.resolve('faceStore');
const pool: any = container.resolve('syntheticWorkerPool');
const sweepOrphanFaces: any = container.resolve('sweepOrphanFaces');
// eslint-disable-next-line unicorn/prefer-top-level-await
const databaseReady = database.initialize().then(() => {
  logger.info('database initialization complete');
});
// eslint-disable-next-line unicorn/prefer-top-level-await
const faceStoreReady = faceStore.initialize().then(() => {
  logger.info('face store initialization complete');
});
Promise.all([databaseReady, faceStoreReady])
  .then(() => {
    pool.start();
    scheduleOrphanSweep();
  })
  // eslint-disable-next-line unicorn/prefer-top-level-await
  .catch((error: any) => {
    logger.error('synthetic store initialization error:', error);
  });

// Defensive, periodic cross-store sweep removing faces whose asset no longer
// exists (belt-and-braces against any delete path that bypassed the use case).
// Runs once after startup and then on an interval; set the interval to 0 to
// disable. Skips a tick if the previous sweep is still running.
let orphanSweepTimer: ReturnType<typeof setInterval> | null = null;
function scheduleOrphanSweep(): void {
  const intervalMs = settings.getInt('ORPHAN_SWEEP_INTERVAL_MS', 86_400_000);
  if (intervalMs <= 0) return;
  let running = false;
  const run = async (): Promise<void> => {
    if (running) return;
    running = true;
    try {
      await sweepOrphanFaces();
    } catch (error: any) {
      logger.error('orphan sweep failed:', error);
    } finally {
      running = false;
    }
  };
  void run();
  orphanSweepTimer = setInterval(() => void run(), intervalMs);
}

// Drain the worker pool on graceful shutdown so claimed-but-not-yet-finished
// jobs are recorded properly rather than vanishing with the process.
let shuttingDown = false;
async function shutdown(signal: string): Promise<void> {
  if (shuttingDown) return;
  shuttingDown = true;
  logger.info(`received ${signal}, stopping worker pool…`);
  if (orphanSweepTimer) clearInterval(orphanSweepTimer);
  try {
    await pool.stop();
  } catch (error: any) {
    logger.error('worker pool stop error:', error);
  }
  // server process: signal handlers really do need to exit
  // eslint-disable-next-line unicorn/no-process-exit
  process.exit(0);
}
process.on('SIGTERM', () => {
  void shutdown('SIGTERM');
});
process.on('SIGINT', () => {
  void shutdown('SIGINT');
});

// set up Express.js application
const app = express();
app.use(
  helmet({
    // allow GraphQL to work properly
    crossOriginEmbedderPolicy: false,
    contentSecurityPolicy: false
  })
);
// configure for development or production
if (process.env.NODE_ENV === 'production') {
  app.use(morgan('combined'));
  ViteExpress.config({ mode: 'production' });
} else {
  app.use(morgan('dev'));
}

// set up Apollo Server
const httpServer = http.createServer(app);
const graphqlServer = new ApolloServer({
  typeDefs,
  resolvers,
  plugins: [
    ApolloServerPluginDrainHttpServer({ httpServer }),
    // use local default landing page even in production
    ApolloServerPluginLandingPageLocalDefault()
  ],
  // allow introspection even in production
  introspection: true
});
await graphqlServer.start();

app.get('/liveness', (_req, res) => {
  res.status(200).json({ status: 'healthy', uptime: process.uptime() });
});
app.use(
  '/graphql',
  cors(),
  express.json(),
  expressMiddleware(graphqlServer, {
    context: async () => {
      const recordRepository: any = container.resolve('recordRepository');
      const faceStore: any = container.resolve('faceStore');
      return {
        metadataLoader: createMetadataLoader(recordRepository),
        syntheticLoader: createSyntheticLoader(recordRepository),
        syntheticStatusLoader: createSyntheticStatusLoader(
          recordRepository,
          faceStore
        ),
        peopleLoader: createPeopleLoader(faceStore)
      };
    }
  })
);
app.use('/assets', assetsRouter);
app.use('/records', recordsRouter);
app.use('/faces', facesRouter);

const port = Number.parseInt(process.env['PORT'] || '3000');
ViteExpress.listen(app, port, () => logger.info(`Server listening at ${port}`));

// n.b. when running in production, vite-express will run in "viteless" mode and
// serve the front-end content out of the dist directory by default
