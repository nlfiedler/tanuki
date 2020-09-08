//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:get_it/get_it.dart';
import 'update_asset_bloc.dart';

void initModifyBlocs(GetIt getIt) {
  getIt.registerFactory(
    () => UpdateAssetBloc(usecase: getIt()),
  );
}
