//
// Copyright (c) 2025 Nathan Fiedler
//
import { createResource, Suspense } from 'solid-js'
import { type TypedDocumentNode, gql } from '@apollo/client'
import { useApolloClient } from '../ApolloProvider'
import {
  type QuerySearchArgs,
  type Query,
  SortField,
  SortOrder,
} from 'tanuki/generated/graphql.ts'
import CardsGrid from '../components/CardsGrid.tsx'

const SEARCH_ASSETS: TypedDocumentNode<Query, QuerySearchArgs> = gql`
  query Search($params: SearchParams!, $offset: Int, $limit: Int) {
    search(params: $params, offset: $offset, limit: $limit) {
      results {
        assetId
        datetime
        filename
        location {
          label
          city
          region
        }
        mediaType
      }
      count
      lastPage
    }
  }
`

function buildParams(): QuerySearchArgs {
  return {
    params: {
      tags: [],
      locations: [],
      before: '275760-09-12',
      sortField: SortField.Date,
      sortOrder: SortOrder.Descending,
    },
    offset: 0,
    limit: 18,
  }
}

function Home() {
  const client = useApolloClient()
  // TODO: add a 'getter' function as the first argument to createResource() to
  // ensure resource is run whenever inputs change
  const [data] = createResource(async () => {
    const { data } = await client.query({
      query: SEARCH_ASSETS,
      variables: buildParams(),
    })
    return data
  })
  return (
    <Suspense fallback={<button class="button is-loading">...</button>}>
      <CardsGrid
        results={data()?.search.results}
        onClick={() => console.log('click')}
      />
    </Suspense>
  )
}

export default Home
