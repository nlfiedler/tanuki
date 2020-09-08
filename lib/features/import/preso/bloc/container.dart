//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:get_it/get_it.dart';
import 'bulk_update_bloc.dart';
import 'ingest_assets_bloc.dart';
import 'recent_imports_bloc.dart';
import 'upload_file_bloc.dart';

void initImportBlocs(GetIt getIt) {
  getIt.registerFactory(
    () => BulkUpdateBloc(usecase: getIt()),
  );
  getIt.registerFactory(
    () => IngestAssetsBloc(usecase: getIt()),
  );
  getIt.registerFactory(
    () => RecentImportsBloc(usecase: getIt()),
  );
  getIt.registerFactory(
    () => UploadFileBloc(usecase: getIt()),
  );
}
