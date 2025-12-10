//
// Copyright (c) 2025 Nathan Fiedler
//
import fs from 'node:fs/promises';
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import LoadAssets from 'tanuki/server/domain/usecases/load-assets.ts';
import { recordRepositoryMock } from './mocking.ts';

describe('LoadAssets use case', function () {
  test('should insert nothing for empty inputs', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({
      storeAssets: mock(() => Promise.resolve())
    });
    const usecase = LoadAssets({
      recordRepository: mockRecordRepository
    });
    // act
    const inputs = new Array<any>();
    await usecase(inputs);
    // assert
    expect(mockRecordRepository.storeAssets).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });

  test('should load old dump format records into database', async function () {
    // arrange
    const mockDatabase = new Array<Asset>();
    const mockRecordRepository = recordRepositoryMock({
      storeAssets: mock((incoming: Asset[]) => {
        for (const entry of incoming) {
          mockDatabase.push(entry);
        }
        return Promise.resolve();
      })
    });
    const usecase = LoadAssets({
      recordRepository: mockRecordRepository
    });
    // act
    const raw = await fs.readFile('test/fixtures/dump.json', {
      encoding: 'utf8'
    });
    const lines = raw.split(/\r?\n/).filter((ln) => ln.length > 0);
    const inputs = lines.map((ln) => JSON.parse(ln));
    await usecase(inputs);
    // assert
    expect(mockDatabase).toHaveLength(4);

    // location field is null
    expect(mockDatabase[0]?.key).toEqual('dGVzdHMvZml4dHVyZXMvZjF0LmpwZw==');
    expect(mockDatabase[0]?.checksum).toEqual(
      'sha256-5514da7cbe82ef4a0c8dd7c025fba78d8ad085b47ae8cee74fb87705b3d0a630'
    );
    expect(mockDatabase[0]?.filename).toEqual('f1t.jpg');
    expect(mockDatabase[0]?.byteLength).toEqual(841);
    expect(mockDatabase[0]?.mediaType).toEqual('image/jpeg');
    expect(mockDatabase[0]?.tags).toEqual(['cat', 'dog']);
    expect(mockDatabase[0]?.importDate.getFullYear()).toEqual(2024);
    expect(mockDatabase[0]?.caption).toEqual('cute cat and dog playing');
    expect(mockDatabase[0]?.location).toBeNull();
    expect(mockDatabase[0]?.userDate).toBeNull();
    expect(mockDatabase[0]?.originalDate).toBeNull();

    // location has only a label
    expect(mockDatabase[1]?.key).toEqual(
      'dGVzdHMvZml4dHVyZXMvZGNwXzEwNjkuanBn'
    );
    expect(mockDatabase[1]?.checksum).toEqual(
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07'
    );
    expect(mockDatabase[1]?.filename).toEqual('dcp_1069.jpg');
    expect(mockDatabase[1]?.byteLength).toEqual(80_977);
    expect(mockDatabase[1]?.mediaType).toEqual('image/jpeg');
    expect(mockDatabase[1]?.tags).toEqual(['mariachi']);
    expect(mockDatabase[1]?.importDate.getFullYear()).toEqual(2024);
    expect(mockDatabase[1]?.caption).toEqual('mariachi band playing');
    expect(mockDatabase[1]?.location?.label).toEqual('cabo san lucas');
    expect(mockDatabase[1]?.location?.city).toBeNull();
    expect(mockDatabase[1]?.location?.region).toBeNull();
    expect(mockDatabase[1]?.userDate).toBeNull();
    expect(mockDatabase[1]?.originalDate).toBeNull();

    // location has all 3 fields
    expect(mockDatabase[2]?.key).toEqual(
      'dGVzdHMvZml4dHVyZXMvc2hpcnRfc21hbGwuaGVpYw=='
    );
    expect(mockDatabase[2]?.checksum).toEqual(
      'sha256-2955581c357f7b4b3cd29af11d9bebd32a4ad1746e36c6792dc9fa41a1d967ae'
    );
    expect(mockDatabase[2]?.filename).toEqual('shirt_small.heic');
    expect(mockDatabase[2]?.byteLength).toEqual(4995);
    expect(mockDatabase[2]?.mediaType).toEqual('image/jpeg');
    expect(mockDatabase[2]?.tags).toEqual(['coffee']);
    expect(mockDatabase[2]?.importDate.getFullYear()).toEqual(2024);
    expect(mockDatabase[2]?.caption).toBeNull();
    expect(mockDatabase[2]?.location).toEqual(
      Location.parse("peet's; Berkeley, CA")
    );
    expect(mockDatabase[2]?.userDate?.getFullYear()).toEqual(1914);
    expect(mockDatabase[2]?.originalDate).toBeNull();

    // location is just a string, not an object
    expect(mockDatabase[3]?.key).toEqual('2eHJndjc4ZzF6bjZ4anN6c2s4Lm1vdg==');
    expect(mockDatabase[3]?.checksum).toEqual(
      'sha256-0c4cf4269e9ab56913d54a917bd46fc8f947f4d0ea3750f1909edc779e0a0de5'
    );
    expect(mockDatabase[3]?.filename).toEqual('IMG_6019.MOV');
    expect(mockDatabase[3]?.byteLength).toEqual(37_190_970);
    expect(mockDatabase[3]?.mediaType).toEqual('video/quicktime');
    expect(mockDatabase[3]?.tags).toEqual(['joseph', 'singing']);
    expect(mockDatabase[3]?.importDate.getFullYear()).toEqual(2014);
    expect(mockDatabase[3]?.caption).toBeNull();
    expect(mockDatabase[3]?.location).toEqual(Location.parse('car'));
    expect(mockDatabase[3]?.userDate).toBeNull();
    expect(mockDatabase[3]?.originalDate?.getFullYear()).toEqual(2014);

    expect(mockRecordRepository.storeAssets).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });
});
