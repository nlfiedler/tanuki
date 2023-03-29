//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/core/domain/usecases/providers.dart';
import 'assign_attributes_bloc.dart';
import 'bulk_update_bloc.dart';
import 'ingest_assets_bloc.dart';
import 'recent_imports_bloc.dart';
import 'upload_file_bloc.dart';

final assignAttributesBlocProvider = Provider.autoDispose<AssignAttributesBloc>(
  (ref) => AssignAttributesBloc(),
);

final bulkUpdateBlocProvider = Provider.autoDispose<BulkUpdateBloc>(
  (ref) => BulkUpdateBloc(
    usecase: ref.read(bulkUpdateUsecaseProvider),
  ),
);

final ingestAssetsBlocProvider = Provider.autoDispose<IngestAssetsBloc>(
  (ref) => IngestAssetsBloc(
    usecase: ref.read(ingestAssetsUsecaseProvider),
  ),
);

final recentImportsBlocProvider = Provider.autoDispose<RecentImportsBloc>(
  (ref) => RecentImportsBloc(
    usecase: ref.read(queryRecentsUsecaseProvider),
  ),
);

final uploadFileBlocProvider = Provider.autoDispose<UploadFileBloc>(
  (ref) => UploadFileBloc(
    usecase: ref.read(uploadAssetUsecaseProvider),
  ),
);
