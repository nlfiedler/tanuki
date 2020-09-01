//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:get_it/get_it.dart';
import 'upload_file_bloc.dart';

void initUploadBlocs(GetIt getIt) {
  getIt.registerFactory(
    () => UploadFileBloc(usecase: getIt()),
  );
}
