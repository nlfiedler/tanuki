//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:get_it/get_it.dart';
import 'all_locations_bloc.dart';
import 'all_tags_bloc.dart';
import 'all_years_bloc.dart';
import 'asset_count_bloc.dart';
import 'query_assets_bloc.dart';

void initBrowseBlocs(GetIt getIt) {
  getIt.registerFactory(
    () => AllLocationsBloc(usecase: getIt()),
  );
  getIt.registerFactory(
    () => AllTagsBloc(usecase: getIt()),
  );
  getIt.registerFactory(
    () => AllYearsBloc(usecase: getIt()),
  );
  getIt.registerFactory(
    () => AssetCountBloc(usecase: getIt()),
  );
  getIt.registerFactory(
    () => QueryAssetsBloc(usecase: getIt()),
  );
}
