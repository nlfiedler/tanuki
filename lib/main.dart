//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart';
import 'package:tanuki/features/browse/preso/screens/asset_screen.dart';
import 'package:tanuki/features/browse/preso/screens/home_screen.dart';
import 'package:tanuki/features/upload/preso/screens/recents_screen.dart';
import 'package:tanuki/features/upload/preso/screens/upload_screen.dart';
import 'container.dart' as ioc;

void main() {
  ioc.init();
  runApp(MyApp());
}

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<AssetBrowserBloc>(
      create: (_) => ioc.getIt<AssetBrowserBloc>(),
      child: MaterialApp(
        title: 'Tanuki',
        initialRoute: '/',
        routes: {
          '/': (context) => HomeScreen(),
          '/asset': (context) => AssetScreen(),
          '/recents': (context) => RecentsScreen(),
          '/upload': (context) => UploadScreen(),
        },
        theme: ThemeData(
          brightness: Brightness.dark,
          visualDensity: VisualDensity.adaptivePlatformDensity,
        ),
      ),
    );
  }
}
