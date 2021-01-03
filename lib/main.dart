//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/features/browse/preso/bloc/all_locations_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/all_tags_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/all_years_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/providers.dart';
import 'package:tanuki/features/browse/preso/screens/asset_screen.dart';
import 'package:tanuki/features/browse/preso/screens/home_screen.dart';
import 'package:tanuki/features/import/preso/screens/recents_screen.dart';
import 'package:tanuki/features/import/preso/screens/upload_screen.dart';
import 'package:tanuki/features/modify/preso/screens/edit_asset_screen.dart';

void main() {
  runApp(ProviderScope(child: MyApp()));
}

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: [
        BlocProvider<AssetBrowserBloc>(
          create: (_) => BuildContextX(context).read(assetBrowserBlocProvider),
        ),
        BlocProvider<AllLocationsBloc>(
          create: (_) => BuildContextX(context).read(allLocationsBlocProvider),
        ),
        BlocProvider<AllTagsBloc>(
          create: (_) => BuildContextX(context).read(allTagsBlocProvider),
        ),
        BlocProvider<AllYearsBloc>(
          create: (_) => BuildContextX(context).read(allYearsBlocProvider),
        ),
      ],
      child: MaterialApp(
        title: 'Tanuki',
        initialRoute: '/',
        routes: {
          '/': (context) => HomeScreen(),
          '/asset': (context) => AssetScreen(),
          '/edit': (context) => EditAssetScreen(),
          '/recents': (context) => RecentsScreen(),
          '/upload': (context) => UploadScreen(),
        },
        navigatorObservers: [_AttributeRefresher()],
        theme: ThemeData(
          brightness: Brightness.dark,
          visualDensity: VisualDensity.adaptivePlatformDensity,
        ),
      ),
    );
  }
}

class _AttributeRefresher extends NavigatorObserver {
  @override
  void didPop(Route route, Route previousRoute) {
    // The route and previousRoute values seem backward to what their names
    // imply (e.g. route is where we are coming from and previousRoute is the
    // one to which we are returning).
    if (previousRoute.isFirst) {
      // If returning to the home screen, signal the selector blocs to refresh
      // their state since the page we just left may have altered the data in
      // some manner.
      final context = navigator.context;
      BlocProvider.of<AllLocationsBloc>(context).add(LoadAllLocations());
      BlocProvider.of<AllTagsBloc>(context).add(LoadAllTags());
      BlocProvider.of<AllYearsBloc>(context).add(LoadAllYears());
    }
  }
}
