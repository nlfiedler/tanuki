//
// Copyright (c) 2025 Nathan Fiedler
//
import { createResource, Show, Suspense } from 'solid-js'
import { type TypedDocumentNode, gql } from '@apollo/client'
import { useApolloClient } from './ApolloProvider'

type AssetCountQuery = {
  count: 'Int'
}

type AssetCountQueryVariables = Record<string, never>

type AssetCountQueryType = TypedDocumentNode<
  AssetCountQuery,
  AssetCountQueryVariables
>

const GET_ASSET_COUNT: AssetCountQueryType = gql`
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
