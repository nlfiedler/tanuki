import type { CodegenConfig } from '@graphql-codegen/cli';

const config: CodegenConfig = {
  overwrite: true,
  schema: 'server/preso/graphql/schema.graphql',
  generates: {
    'generated/graphql.ts': {
      config: {
        // typescript-resolvers plugin: Adds an index signature to any generated resolver(?)
        useIndexSignature: true,
        // typescript plugin: Will use import type {} rather than import {} when importing only types. 
        useTypeImports: true
      },
      plugins: ['typescript', 'typescript-resolvers']
    }
  }
};

export default config;
