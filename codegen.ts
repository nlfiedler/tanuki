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
        useTypeImports: true,
        // The custom Date/BigInt scalars are untyped end-to-end (input arrives as
        // JSON strings/numbers, output is serialized verbatim). The typescript
        // plugin v6 changed the default scalar type from `any` to `unknown`;
        // restore `any` so both client and server keep their prior semantics.
        defaultScalarType: 'any'
      },
      plugins: ['typescript', 'typescript-resolvers']
    }
  }
};

export default config;
