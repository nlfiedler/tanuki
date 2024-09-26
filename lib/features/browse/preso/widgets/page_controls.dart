//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:responsive_framework/responsive_framework.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart';

class PageControls extends StatelessWidget {
  const PageControls({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<AssetBrowserBloc>(context),
      child: BlocBuilder<AssetBrowserBloc, AssetBrowserState>(
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
                      BlocProvider.of<AssetBrowserBloc>(context)
                          .add(ShowPage(page: state.pageNumber - 1));
                    }
                  : null,
              child: const Icon(Icons.chevron_left),
            );
            final nextPageButton = ElevatedButton(
              onPressed: state.pageNumber < state.lastPage
                  ? () {
                      BlocProvider.of<AssetBrowserBloc>(context)
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
                BlocProvider.of<AssetBrowserBloc>(context)
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
                      child: Text('${state.results.count} results'),
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
                ResponsiveVisibility(
                  hiddenConditions: const [
                    Condition.smallerThan(name: TABLET, value: false),
                  ],
                  child: Expanded(
                    flex: 2,
                    child: PageInputForm(
                      lastPage: state.lastPage,
                      onSubmit: (page) {
                        BlocProvider.of<AssetBrowserBloc>(context)
                            .add(ShowPage(page: page));
                      },
                    ),
                  ),
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

typedef PageCallback = void Function(int);

class PageInputForm extends StatefulWidget {
  final int lastPage;
  final PageCallback onSubmit;

  const PageInputForm({
    Key? key,
    required this.lastPage,
    required this.onSubmit,
  }) : super(key: key);

  @override
  State<PageInputForm> createState() => _PageInputFormState();
}

class _PageInputFormState extends State<PageInputForm> {
  final GlobalKey<FormBuilderState> _fbKey = GlobalKey<FormBuilderState>();

  void submitPageInput() {
    if (_fbKey.currentState?.saveAndValidate() ?? false) {
      widget.onSubmit(_fbKey.currentState?.value['page']);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        const Text('Go to page:'),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8.0),
            child: FormBuilder(
              key: _fbKey,
              initialValue: const {'page': '1'},
              child: FormBuilderTextField(
                readOnly: widget.lastPage < 2,
                name: 'page',
                validator: FormBuilderValidators.compose([
                  FormBuilderValidators.required(),
                  FormBuilderValidators.numeric(),
                  FormBuilderValidators.min(1),
                  FormBuilderValidators.max(widget.lastPage),
                ]),
                valueTransformer: (String? text) {
                  return int.tryParse(text ?? '1');
                },
                onSubmitted: (text) {
                  submitPageInput();
                },
                keyboardType: TextInputType.number,
              ),
            ),
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: TextButton(
            onPressed: widget.lastPage < 2 ? null : submitPageInput,
            child: const Text('GO'),
          ),
        ),
      ],
    );
  }
}
