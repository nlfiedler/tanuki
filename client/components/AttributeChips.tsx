//
// Copyright (c) 2025 Nathan Fiedler
//
import type { Accessor } from 'solid-js';
import { For } from 'solid-js';

interface AttributeChipsProps {
  attrs: Accessor<string[]>;
  rmfun: (name: string) => void;
}

function AttributeChips(props: AttributeChipsProps) {
  return (
    <For each={props.attrs()}>
      {(item) => (
        <div class="control">
          <div class="tags has-addons">
            <a class="tag">{item}</a>
            <a class="tag is-delete" on:click={(_) => props.rmfun(item)}></a>
          </div>
        </div>
      )}
    </For>
  );
}

export default AttributeChips;
