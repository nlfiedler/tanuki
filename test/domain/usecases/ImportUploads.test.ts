//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from "bun:test";
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import ImportAsset from 'tanuki/server/domain/usecases/ImportAsset.ts';
import ImportUploads from 'tanuki/server/domain/usecases/ImportUploads.ts';
import { blobRepositoryMock, locationRepositoryMock, recordRepositoryMock } from './mocking.ts';

describe('ImportUploads use case', function () {
  test('should import several new assets', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({
      putAsset: mock((asset: Asset) => {
        // validate some of the assets were processed correctly
        if (asset.checksum === 'sha256-4f86f7dd48474b8e6571beeabbd79111267f143c0786bcd45def0f6b33ae0423') {
          expect(asset.mediaType).toEqual('video/quicktime');
          expect(asset.filename).toEqual('100_1206.MOV');
        }
        if (asset.checksum === 'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07') {
          expect(asset.mediaType).toEqual('image/jpeg');
          expect(asset.filename).toEqual('dcp_1069.jpg');
        }
        if (asset.checksum === 'sha256-c2794b60c938b42635ba254539f4e8fa3f56176511be55153cb5298d029c91e5') {
          expect(asset.mediaType).toEqual('application/octet-stream');
          expect(asset.filename).toEqual('README.dumb');
        }
        if (asset.checksum === 'sha256-095964d07f3e821659d4eb27ed9e20cd5160c53385562df727e98eb815bb371f') {
          expect(asset.mediaType).toEqual('text/plain');
          expect(asset.filename).toEqual('lorem-ipsum.txt');
        }
        if (asset.checksum === 'sha256-2955581c357f7b4b3cd29af11d9bebd32a4ad1746e36c6792dc9fa41a1d967ae') {
          expect(asset.mediaType).toEqual('image/heic');
          expect(asset.filename).toEqual('shirt_small.heic');
        }
        if (asset.checksum === 'sha256-269b5cc1eeb21b8bba2881e96b2e6bc8109c4edf150d3fda69d8f87bf34acd81') {
          expect(asset.mediaType).toEqual('video/mp4');
          expect(asset.filename).toEqual('ooo_tracks.mp4');
        }
        if (asset.checksum === 'sha256-c10f31a5f7a77eae84f9595bef2494226e040c64713454700e04b1f9eb829163') {
          expect(asset.mediaType).toEqual('video/x-msvideo');
          expect(asset.filename).toEqual('MVI_0727.AVI');
        }
        return Promise.resolve();
      }),
    });
    const mockBlobRepository = blobRepositoryMock({});
    const mockLocationRepository = locationRepositoryMock({});
    const importAsset = ImportAsset({
      recordRepository: mockRecordRepository,
      blobRepository: mockBlobRepository,
      locationRepository: mockLocationRepository,
    });
    const usecase = ImportUploads({ importAsset });
    // act
    const actual = await usecase('test/fixtures');
    // assert
    expect(actual).toEqual(20);
    expect(mockBlobRepository.storeBlob).toHaveBeenCalledTimes(20);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(20);
    mock.clearAllMocks();
  });
});
