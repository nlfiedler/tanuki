//
// Copyright (c) 2025 Nathan Fiedler
//
import { createResource, For, Suspense, type JSX } from 'solid-js'
import { type TypedDocumentNode, gql } from '@apollo/client'
import { useApolloClient } from '../ApolloProvider'
import { type Query } from 'tanuki/generated/graphql.ts'

const ALL_TAGS: TypedDocumentNode<Query, Record<string, never>> = gql`
  query {
    tags {
      label
      count
    }
  }
`

interface TagSelectorProps {
  addfun: (value: string) => void
}

function TagSelector(props: TagSelectorProps) {
  const client = useApolloClient()
  const [tagsQuery] = createResource(async () => {
    const { data } = await client.query({ query: ALL_TAGS })
    return data
  })
  const sortedTags = () => {
    // the tags returned from the server are in no particular order
    const sorted = []
    for (const tag of tagsQuery()?.tags ?? []) {
      sorted.push({ label: tag.label, count: tag.count })
    }
    sorted.sort((a, b) => a.label.localeCompare(b.label))
    return sorted
  }
  //
  // n.b. on:input is called for every single keystroke, while on:change is
  // called under several conditions:
  //
  // - user selects one of the available datalist options
  // - user types some text and presses the Enter key
  // - user types some text and moves the focus
  //
  const onChange: JSX.EventHandlerWithOptionsUnion<
    HTMLInputElement,
    Event,
    JSX.ChangeEventHandler<HTMLInputElement, Event>
  > = (event) => {
    const target = event.currentTarget
    if (target) {
      const value = target.value
      if (value) {
        props.addfun(value)
        target.value = ''
      }
      event.stopPropagation()
    }
  }

  return (
    <Suspense fallback={'...'}>
      <div class="field is-horizontal">
        <div class="field-label is-normal">
          <label class="label" for="tags-input">
            Tags
          </label>
        </div>
        <div class="field-body">
          <p class="control">
            <input
              class="input"
              type="text"
              id="tags-input"
              list="tag-labels"
              placeholder="Choose tags"
              on:change={onChange}
            />
            <datalist id="tag-labels">
              <For each={sortedTags()}>
                {(tag) => <option value={tag.label}></option>}
              </For>
            </datalist>
          </p>
        </div>
      </div>
    </Suspense>
  )
}

export default TagSelector
