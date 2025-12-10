//
// Copyright (c) 2025 Nathan Fiedler
//
import { createEffect, createResource, Suspense } from 'solid-js';
import { useLocation } from '@solidjs/router';
import { type TypedDocumentNode, gql } from '@apollo/client';
import { useApolloClient } from '../apollo-provider';
import { type Query } from 'tanuki/generated/graphql.ts';

const GET_ASSET_COUNT: TypedDocumentNode<Query, Record<string, never>> = gql`
  query {
    count
  }
`;

function AssetCount() {
  const client = useApolloClient();
  const [countQuery, { refetch }] = createResource(async () => {
    const { data } = await client.query({ query: GET_ASSET_COUNT });
    return data;
  });
  const location = useLocation();
  // the pathname is not actually used, just listening for route changes
  createEffect(() => refetch(location.pathname));
  return (
    <Suspense fallback={<span>...</span>}>
      <span>{countQuery.latest?.count} assets</span>
    </Suspense>
  );
}

export default AssetCount;
