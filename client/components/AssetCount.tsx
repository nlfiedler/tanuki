//
// Copyright (c) 2025 Nathan Fiedler
//
import { createResource, Suspense } from 'solid-js'
import { type TypedDocumentNode, gql } from '@apollo/client'
import { useApolloClient } from '../ApolloProvider'
import { type Query } from 'tanuki/generated/graphql.ts'

const GET_ASSET_COUNT: TypedDocumentNode<Query, Record<string, never>> = gql`
  query {
    count
  }
`

function AssetCount() {
  const client = useApolloClient()
  const [data] = createResource(async () => {
    const { data } = await client.query({ query: GET_ASSET_COUNT })
    return data
  })
  return (
    <Suspense fallback={<span>...</span>}>
      <span>{data()?.count} assets</span>
    </Suspense>
  )
}

export default AssetCount
