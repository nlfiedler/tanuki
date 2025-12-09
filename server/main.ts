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
import { typeDefs, resolvers } from 'tanuki/server/preso/graphql/schema.ts';

// (asynchronously) prepare the database
const database = container.resolve('recordRepository');
database
  .createIfMissing()
  .then(() => {
    logger.info('database initialization complete');
  })
  .catch((err: any) => {
    logger.error('database initialization error:', err);
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
if (process.env.NODE_ENV !== 'production') {
  app.use(morgan('dev'));
} else {
  app.use(morgan('combined'));
  ViteExpress.config({ mode: 'production' });
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
app.use('/graphql', cors(), express.json(), expressMiddleware(graphqlServer));
app.use('/assets', assetsRouter);
app.use('/records', recordsRouter);

const port = parseInt(process.env['PORT'] || '3000');
ViteExpress.listen(app, port, () => logger.info(`Server listening at ${port}`));

// n.b. when running in production, vite-express will run in "viteless" mode and
// serve the front-end content out of the dist directory by default
