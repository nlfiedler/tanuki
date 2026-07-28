//
// Copyright (c) 2025 Nathan Fiedler
//
import { createContext, useContext } from 'solid-js';
import { HttpLink, ApolloClient, InMemoryCache } from '@apollo/client';

// Apollo Client 4.2+ requires application-wide default options to be declared
// at the type level before they can be set. See:
// https://www.apollographql.com/docs/react/data/typescript#declaring-default-options-for-type-safety
declare module '@apollo/client' {
  namespace ApolloClient {
    namespace DeclareDefaultOptions {
      interface Query {
        errorPolicy: 'all';
      }
      interface WatchQuery {
        errorPolicy: 'all';
      }
    }
  }
}

const ApolloContext = createContext<ApolloClient | undefined>();

export function ApolloProvider(props: { children: any }) {
  const client = new ApolloClient({
    link: new HttpLink({ uri: '/graphql' }),
    // a cache is required, but caching can be disabled
    cache: new InMemoryCache(),
    defaultOptions: {
      query: {
        fetchPolicy: 'no-cache',
        errorPolicy: 'all'
      },
      watchQuery: {
        fetchPolicy: 'no-cache',
        errorPolicy: 'all'
      }
    }
  });
  return (
    <ApolloContext.Provider value={client}>
      {props.children}
    </ApolloContext.Provider>
  );
}

export function useApolloClient() {
  const client = useContext(ApolloContext);
  if (!client) {
    throw new Error('useApolloClient must be used within an ApolloProvider');
  }
  return client;
}
