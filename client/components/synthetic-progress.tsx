//
// Copyright (c) 2026 Nathan Fiedler
//
import { createResource, createSignal, onCleanup, Show } from 'solid-js';
import { type TypedDocumentNode, gql } from '@apollo/client';
import { useApolloClient } from '../apollo-provider';
import { type Query } from 'tanuki/generated/graphql.ts';

const SYNTHETIC_JOB_STATUS: TypedDocumentNode<Query, Record<string, never>> =
  gql`
    query SyntheticJobStatus {
      syntheticJobStatus {
        queued
        facesQueued
        facesReady
        facesFailed
      }
    }
  `;

/** How often to re-poll the queue while the page is open, in milliseconds. */
const POLL_MS = 5000;

interface SyntheticProgressProps {
  /** Which pipeline's progress this banner reports. */
  kind: 'faces' | 'labels';
}

/**
 * Live progress banner for the background synthetic-data extraction queue,
 * shown on the Labels and People pages so an operator can watch a backfill
 * drain. Polls `syntheticJobStatus` every few seconds and hides itself when the
 * relevant pipeline's queue is empty — its disappearance is the "done" signal.
 *
 * The faces pipeline exposes a real completion ratio (done vs. still-queued),
 * so it renders a determinate bar; the labels pipeline only exposes a remaining
 * count, so it renders an indeterminate bar with that count.
 */
function SyntheticProgress(props: SyntheticProgressProps) {
  const client = useApolloClient();
  // Bump a tick on an interval; createResource re-fetches whenever its source
  // changes. The source is wrapped in an object so it is always truthy — a 0
  // tick would otherwise make createResource skip the initial fetch.
  const [tick, setTick] = createSignal(0);
  const timer = setInterval(() => setTick((t) => t + 1), POLL_MS);
  onCleanup(() => clearInterval(timer));
  const [statusQuery] = createResource(
    () => ({ tick: tick() }),
    async () => {
      const { data } = await client.query({
        query: SYNTHETIC_JOB_STATUS,
        fetchPolicy: 'network-only'
      });
      return data?.syntheticJobStatus ?? null;
    }
  );

  // `.latest` keeps the prior value visible across re-polls, so the banner
  // updates in place rather than blanking out on every fetch.
  const status = () => statusQuery.latest;
  const remaining = () => {
    const s = status();
    if (!s) return 0;
    // Labels has no dedicated queue count; derive it from the total.
    return props.kind === 'faces' ? s.facesQueued : s.queued - s.facesQueued;
  };
  const ready = () => status()?.facesReady ?? 0;
  const failed = () => status()?.facesFailed ?? 0;
  // Faces total = already-done + still-queued. In-flight jobs are briefly
  // absent from the queue, so this can read a hair low mid-run.
  const total = () => ready() + remaining();
  const percent = () => (total() > 0 ? Math.floor((ready() / total()) * 100) : 0);

  return (
    <Show when={remaining() > 0}>
      <div class="notification is-info is-light py-2 px-4 mb-4">
        <Show
          when={props.kind === 'faces'}
          fallback={
            <>
              <div class="is-size-7 mb-1">
                <strong>Label extraction in progress</strong> — {remaining()}{' '}
                {remaining() === 1 ? 'job' : 'jobs'} remaining
              </div>
              <progress class="progress is-info is-small">working…</progress>
            </>
          }
        >
          <div class="is-flex is-justify-content-space-between is-size-7 mb-1">
            <span>
              <strong>Face recognition in progress</strong> — {ready()} of{' '}
              {total()} done, {remaining()} queued
              <Show when={failed() > 0}>
                {' '}
                · <span class="has-text-danger">{failed()} failed</span>
              </Show>
            </span>
            <span class="has-text-grey">{percent()}%</span>
          </div>
          <progress
            class="progress is-info is-small"
            value={ready()}
            max={total()}
          >
            {percent()}%
          </progress>
        </Show>
      </div>
    </Show>
  );
}

export default SyntheticProgress;
