//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/core/domain/usecases/get_asset_locations.dart';
import 'bulk_update.dart';
import 'get_all_locations.dart';
import 'get_all_tags.dart';
import 'get_all_years.dart';
import 'get_asset.dart';
import 'get_asset_count.dart';
import 'ingest_assets.dart';
import 'query_assets.dart';
import 'query_recents.dart';
import 'update_asset.dart';
import 'upload_asset.dart';

final bulkUpdateUsecaseProvider = Provider<BulkUpdate>(
  (ref) => BulkUpdate(
    ref.read(entityRepositoryProvider),
  ),
);

final getAllLocationsUsecaseProvider = Provider<GetAllLocations>(
  (ref) => GetAllLocations(
    ref.read(entityRepositoryProvider),
  ),
);

final getAssetLocationsUsecaseProvider = Provider<GetAssetLocations>(
  (ref) => GetAssetLocations(
    ref.read(entityRepositoryProvider),
  ),
);

final getAllTagsUsecaseProvider = Provider<GetAllTags>(
  (ref) => GetAllTags(
    ref.read(entityRepositoryProvider),
  ),
);

final getAllYearsUsecaseProvider = Provider<GetAllYears>(
  (ref) => GetAllYears(
    ref.read(entityRepositoryProvider),
  ),
);

final getAssetUsecaseProvider = Provider<GetAsset>(
  (ref) => GetAsset(
    ref.read(entityRepositoryProvider),
  ),
);

final getAssetCountUsecaseProvider = Provider<GetAssetCount>(
  (ref) => GetAssetCount(
    ref.read(entityRepositoryProvider),
  ),
);

final ingestAssetsUsecaseProvider = Provider<IngestAssets>(
  (ref) => IngestAssets(
    ref.read(assetRepositoryProvider),
  ),
);

final queryAssetsUsecaseProvider = Provider<QueryAssets>(
  (ref) => QueryAssets(
    ref.read(entityRepositoryProvider),
  ),
);

final queryRecentsUsecaseProvider = Provider<QueryRecents>(
  (ref) => QueryRecents(
    ref.read(entityRepositoryProvider),
  ),
);

final updateAssetUsecaseProvider = Provider<UpdateAsset>(
  (ref) => UpdateAsset(
    ref.read(entityRepositoryProvider),
  ),
);

final uploadAssetUsecaseProvider = Provider<UploadAsset>(
  (ref) => UploadAsset(
    ref.read(assetRepositoryProvider),
  ),
);
