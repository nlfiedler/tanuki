//
// Copyright (c) 2026 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { SearchResult } from 'tanuki/server/domain/entities/search.ts';
import AssetsByLabel from 'tanuki/server/domain/usecases/assets-by-label.ts';
import { recordRepositoryMock } from './mocking.ts';

describe('AssetsByLabel use case', function () {
  test('delegates to the record repository keyed by label', async function () {
    const hit = new SearchResult('a-1', 'a-1.jpg', 'image/jpeg', null, new Date());
    const queryByLabel = mock(() => Promise.resolve([hit]));
    const recordRepository = recordRepositoryMock({ queryByLabel });

    const usecase = AssetsByLabel({ recordRepository });
    const results = await usecase('Beach');

    expect(results).toEqual([hit]);
    expect(queryByLabel).toHaveBeenCalledWith('Beach');
  });
});
