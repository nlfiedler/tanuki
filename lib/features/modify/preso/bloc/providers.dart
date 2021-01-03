//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/core/domain/usecases/providers.dart';
import 'update_asset_bloc.dart';

final updateAssetBlocProvider = Provider.autoDispose<UpdateAssetBloc>(
  (ref) => UpdateAssetBloc(
    usecase: ref.read(updateAssetUsecaseProvider),
  ),
);
