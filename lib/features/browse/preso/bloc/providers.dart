//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/core/domain/usecases/providers.dart';
import 'all_locations_bloc.dart';
import 'all_tags_bloc.dart';
import 'all_years_bloc.dart';
import 'asset_bloc.dart';
import 'asset_browser_bloc.dart';
import 'asset_count_bloc.dart';

final allLocationsBlocProvider = Provider.autoDispose<AllLocationsBloc>(
  (ref) => AllLocationsBloc(
    usecase: ref.read(getAllLocationsUsecaseProvider),
  ),
);

final allTagsBlocProvider = Provider.autoDispose<AllTagsBloc>(
  (ref) => AllTagsBloc(
    usecase: ref.read(getAllTagsUsecaseProvider),
  ),
);

final allYearsBlocProvider = Provider.autoDispose<AllYearsBloc>(
  (ref) => AllYearsBloc(
    usecase: ref.read(getAllYearsUsecaseProvider),
  ),
);

final assetBlocProvider = Provider.autoDispose<AssetBloc>(
  (ref) => AssetBloc(
    usecase: ref.read(getAssetUsecaseProvider),
  ),
);

final assetBrowserBlocProvider = Provider.autoDispose<AssetBrowserBloc>(
  (ref) => AssetBrowserBloc(
    usecase: ref.read(queryAssetsUsecaseProvider),
  ),
);

final assetCountBlocProvider = Provider.autoDispose<AssetCountBloc>(
  (ref) => AssetCountBloc(
    usecase: ref.read(getAssetCountUsecaseProvider),
  ),
);
