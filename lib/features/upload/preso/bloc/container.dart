//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:get_it/get_it.dart';
import 'ingest_assets_bloc.dart';
import 'upload_file_bloc.dart';

void initUploadBlocs(GetIt getIt) {
  getIt.registerFactory(
    () => IngestAssetsBloc(usecase: getIt()),
  );
  getIt.registerFactory(
    () => UploadFileBloc(usecase: getIt()),
  );
}
