//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:responsive_framework/responsive_framework.dart';
import 'package:tanuki/features/browse/preso/bloc/all_locations_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/all_tags_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/all_years_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_count_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/providers.dart';
import 'package:tanuki/features/browse/preso/bloc/raw_locations_bloc.dart';
import 'package:tanuki/features/browse/preso/screens/asset_screen.dart';
import 'package:tanuki/features/browse/preso/screens/home_screen.dart';
import 'package:tanuki/features/import/preso/screens/recents_screen.dart';
import 'package:tanuki/features/import/preso/screens/upload_screen.dart';
import 'package:tanuki/features/modify/preso/screens/edit_asset_screen.dart';

void main() {
  runApp(const ProviderScope(child: MyApp()));
}

class MyApp extends ConsumerWidget {
  const MyApp({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MultiBlocProvider(
      providers: [
        BlocProvider<AssetCountBloc>(
          create: (_) => ref.read(assetCountBlocProvider),
        ),
        BlocProvider<AssetBrowserBloc>(
          create: (_) => ref.read(assetBrowserBlocProvider),
        ),
        BlocProvider<AllLocationsBloc>(
          create: (_) => ref.read(allLocationsBlocProvider),
        ),
        BlocProvider<AllTagsBloc>(
          create: (_) => ref.read(allTagsBlocProvider),
        ),
        BlocProvider<AllYearsBloc>(
          create: (_) => ref.read(allYearsBlocProvider),
        ),
        BlocProvider<RawLocationsBloc>(
          create: (_) => ref.read(rawLocationsBlocProvider),
        ),
      ],
      child: MaterialApp(
        builder: (context, child) => ResponsiveBreakpoints.builder(
          child: child!,
          breakpoints: [
            // const ResponsiveBreakpoint.resize(450, name: MOBILE),
            // const ResponsiveBreakpoint.autoScale(800, name: TABLET),
            // const ResponsiveBreakpoint.autoScale(1000, name: TABLET),
            // const ResponsiveBreakpoint.resize(1200, name: DESKTOP),
            // const ResponsiveBreakpoint.autoScale(2460, name: "4K"),
            const Breakpoint(start: 0, end: 450, name: MOBILE),
            const Breakpoint(start: 451, end: 800, name: TABLET),
            const Breakpoint(start: 801, end: 1920, name: DESKTOP),
            const Breakpoint(start: 1921, end: double.infinity, name: '4K'),
          ],
        ),
        title: 'Tanuki',
        initialRoute: '/',
        routes: {
          '/': (context) => HomeScreen(),
          '/asset': (context) => const AssetScreen(),
          '/edit': (context) => const EditAssetScreen(),
          '/recents': (context) => const RecentsScreen(),
          '/upload': (context) => const UploadScreen(),
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
  void didPop(Route route, Route? previousRoute) {
    // The route and previousRoute values seem backward to what their names
    // imply (e.g. route is where we are coming from and previousRoute is the
    // one to which we are returning).
    if (previousRoute?.isFirst ?? false) {
      // If returning to the home screen, signal the selector blocs to refresh
      // their state since the page we just left may have altered the data in
      // some manner.
      final context = navigator!.context;
      BlocProvider.of<AssetCountBloc>(context).add(LoadAssetCount());
      BlocProvider.of<AllLocationsBloc>(context).add(LoadAllLocations());
      BlocProvider.of<AllTagsBloc>(context).add(LoadAllTags());
      BlocProvider.of<AllYearsBloc>(context).add(LoadAllYears());
      BlocProvider.of<RawLocationsBloc>(context).add(LoadRawLocations());
    }
  }
}
