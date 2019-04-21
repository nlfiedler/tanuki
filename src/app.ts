//
// Copyright (c) 2018 Nathan Fiedler
//
const createError = require('http-errors')
import * as express from 'express'
import * as morgan from 'morgan'
import pageRoutes from './routes/pages'
import gqlRoutes from './routes/graphql'
const backend = require('lib/backend')
const config = require('config')
const logger = require('lib/logging')
const rfs = require('rotating-file-stream')

// Initialize the database asynchronously.
backend.initDatabase().then(function (res: any) {
  logger.info('database initialization result:', res)
}).catch(function (err: Error) {
  logger.error('database initialization error:', err)
})

const app = express()

// view engine setup
app.set('views', 'views')
app.set('view engine', 'ejs')

// Configure the HTTP logging.
if (config.has('morgan.logger.logPath')) {
  const logDirectory = config.get('morgan.logger.logPath')
  const accessLogStream = rfs('access.log', {
    size: '1M',
    maxFiles: 4,
    path: logDirectory
  })
  app.use(morgan('combined', { stream: accessLogStream }))
} else if (process.env.NODE_ENV !== 'production') {
  app.use(morgan('dev'))
} else {
  app.use(morgan('combined'))
}

app.use('/graphql', gqlRoutes)
app.use('/', pageRoutes)

// catch 404 and forward to error handler
app.use((req, res, next) => {
  next(createError(404))
})

// error handler
app.use((err: any, req: express.Request, res: express.Response, next: Function) => {
  // set locals, only providing error in development
  res.locals.message = err.message
  res.locals.error = req.app.get('env') === 'development' ? err : {}

  // render the error page
  res.status(err.status || 500)
  res.render('error')
})

export default app