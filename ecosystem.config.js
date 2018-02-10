//
// pm2 configuration file (see https://github.com/Unitech/pm2)
//
module.exports = {
  apps: [{
    name: 'web',
    script: './bin/www',
    // change name of instance var because of node-config
    // (https://github.com/Unitech/pm2/issues/2045)
    instance_var: 'INSTANCE_ID',
    env: {
      'NODE_ENV': 'development',
      'port': 3000
    },
    env_production: {
      'NODE_ENV': 'production',
      'NODE_CONFIG_ENV': 'production',
      'port': 3000
    }
  }]
}
