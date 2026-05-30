//
// Copyright (c) 2026 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { AttributeCount } from 'tanuki/server/domain/entities/attributes.ts';
import GetLabels from 'tanuki/server/domain/usecases/get-labels.ts';
import { recordRepositoryMock } from './mocking.ts';

describe('GetLabels use case', function () {
  test('returns one entry per distinct label, preferring the case-preserved label from the representative asset', async function () {
    const allPrimaryLabels = mock(() =>
      Promise.resolve([
        // aggregate may return either lowercased (Couch) or case-preserved
        // (SQLite) keys; use case takes display from the sample doc
        new AttributeCount('beach', 3),
        new AttributeCount('cat', 2)
      ])
    );
    const latestAssetByLabel = mock((label: string) => {
      if (label === 'beach') {
        return Promise.resolve({
          assetId: 'newest-beach',
          primaryLabel: 'Beach' // original case from the representative doc
        });
      }
      return Promise.resolve({
        assetId: 'cat-1',
        primaryLabel: 'Cat'
      });
    });
    const recordRepository = recordRepositoryMock({
      allPrimaryLabels,
      latestAssetByLabel
    });

    const entries = await GetLabels({ recordRepository })();

    expect(entries).toHaveLength(2);
    expect(entries[0]!.label).toEqual('Beach');
    expect(entries[0]!.count).toEqual(3);
    expect(entries[0]!.thumbnailAssetId).toEqual('newest-beach');
    expect(entries[1]!.label).toEqual('Cat');
    expect(entries[1]!.thumbnailAssetId).toEqual('cat-1');
  });

  test('falls back to the aggregate label when the sample has no primaryLabel', async function () {
    const recordRepository = recordRepositoryMock({
      allPrimaryLabels: mock(() =>
        Promise.resolve([new AttributeCount('sunset', 1)])
      ),
      latestAssetByLabel: mock(() =>
        Promise.resolve({ assetId: 'a-1', primaryLabel: '' })
      )
    });
    const entries = await GetLabels({ recordRepository })();
    expect(entries).toHaveLength(1);
    expect(entries[0]!.label).toEqual('sunset');
  });

  test('skips a label whose representative asset cannot be located', async function () {
    const recordRepository = recordRepositoryMock({
      allPrimaryLabels: mock(() =>
        Promise.resolve([new AttributeCount('phantom', 1)])
      ),
      latestAssetByLabel: mock(() => Promise.resolve(null))
    });
    const entries = await GetLabels({ recordRepository })();
    expect(entries).toEqual([]);
  });

  test('returns an empty list when nothing is labelled', async function () {
    const recordRepository = recordRepositoryMock({});
    expect(await GetLabels({ recordRepository })()).toEqual([]);
  });
});
