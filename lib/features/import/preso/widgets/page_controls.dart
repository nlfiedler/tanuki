//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/features/import/preso/bloc/recent_imports_bloc.dart';

class PageControls extends StatelessWidget {
  const PageControls({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<RecentImportsBloc>(context),
      child: BlocBuilder<RecentImportsBloc, RecentImportsState>(
        buildWhen: (previous, current) {
          return !(previous is Loaded && current is Loading);
        },
        builder: (context, state) {
          if (state is Error) {
            return Text('Error: ${state.message}');
          }
          if (state is Loaded) {
            final prevPageButton = ElevatedButton(
              onPressed: state.pageNumber > 1
                  ? () {
                      BlocProvider.of<RecentImportsBloc>(context)
                          .add(ShowPage(page: state.pageNumber - 1));
                    }
                  : null,
              child: const Icon(Icons.chevron_left),
            );
            final nextPageButton = ElevatedButton(
              onPressed: state.pageNumber < state.lastPage
                  ? () {
                      BlocProvider.of<RecentImportsBloc>(context)
                          .add(ShowPage(page: state.pageNumber + 1));
                    }
                  : null,
              child: const Icon(Icons.chevron_right),
            );
            final pageNumberText =
                Text('Page ${state.pageNumber} of ${state.lastPage}');
            final pageSizePopup = PopupMenuButton<int>(
              tooltip: 'Set page size',
              icon: const Icon(Icons.pages),
              initialValue: state.pageSize,
              onSelected: (int value) {
                BlocProvider.of<RecentImportsBloc>(context)
                    .add(SetPageSize(size: value));
              },
              itemBuilder: (BuildContext context) => <PopupMenuEntry<int>>[
                const PopupMenuItem<int>(
                  value: 18,
                  child: Text('18'),
                ),
                const PopupMenuItem<int>(
                  value: 36,
                  child: Text('36'),
                ),
                const PopupMenuItem<int>(
                  value: 54,
                  child: Text('54'),
                ),
                const PopupMenuItem<int>(
                  value: 72,
                  child: Text('72'),
                ),
              ],
            );
            final resultsCountText = state.results.count > 0
                ? Expanded(
                    flex: 1,
                    child: Center(
                      child: Text('Pending assets: ${state.results.count}'),
                    ),
                  )
                : const Spacer(flex: 1);
            return Row(
              children: [
                resultsCountText,
                prevPageButton,
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16.0),
                  child: pageNumberText,
                ),
                nextPageButton,
                const SizedBox(
                  width: 48.0,
                ),
                Padding(
                  padding: const EdgeInsets.only(right: 16.0),
                  child: pageSizePopup,
                ),
              ],
            );
          }
          return const Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}
