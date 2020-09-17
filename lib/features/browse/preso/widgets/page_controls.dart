//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart';

class PageControls extends StatelessWidget {
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
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            final prevPageButton = RaisedButton(
              child: Icon(Icons.chevron_left),
              onPressed: state.pageNumber > 1
                  ? () {
                      BlocProvider.of<AssetBrowserBloc>(context)
                          .add(ShowPage(page: state.pageNumber - 1));
                    }
                  : null,
            );
            final nextPageButton = RaisedButton(
              child: Icon(Icons.chevron_right),
              onPressed: state.pageNumber < state.lastPage
                  ? () {
                      BlocProvider.of<AssetBrowserBloc>(context)
                          .add(ShowPage(page: state.pageNumber + 1));
                    }
                  : null,
            );
            final pageNumberText =
                Text('Page ${state.pageNumber} of ${state.lastPage}');
            final pageSizePopup = PopupMenuButton<int>(
              tooltip: 'Set page size',
              icon: Icon(Icons.pages),
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
                : Spacer(flex: 1);
            return Row(
              children: [
                resultsCountText,
                prevPageButton,
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16.0),
                  child: pageNumberText,
                ),
                nextPageButton,
                SizedBox(
                  width: 48.0,
                ),
                Expanded(
                  flex: 2,
                  child: PageInputForm(
                    lastPage: state.lastPage,
                    onSubmit: (page) {
                      BlocProvider.of<AssetBrowserBloc>(context)
                          .add(ShowPage(page: page));
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.only(right: 16.0),
                  child: pageSizePopup,
                ),
              ],
            );
          }
          return Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

typedef PageCallback = void Function(int);

class PageInputForm extends StatefulWidget {
  final int lastPage;
  final PageCallback onSubmit;

  PageInputForm({
    Key key,
    @required this.lastPage,
    @required this.onSubmit,
  }) : super(key: key);

  @override
  _PageInputFormState createState() => _PageInputFormState();
}

class _PageInputFormState extends State<PageInputForm> {
  final GlobalKey<FormBuilderState> _fbKey = GlobalKey<FormBuilderState>();

  void submitPageInput() {
    if (_fbKey.currentState.saveAndValidate()) {
      widget.onSubmit(_fbKey.currentState.value['page']);
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
              initialValue: {'page': '1'},
              child: FormBuilderTextField(
                readOnly: widget.lastPage < 2,
                attribute: 'page',
                validators: [
                  FormBuilderValidators.required(),
                  FormBuilderValidators.numeric(),
                  FormBuilderValidators.min(1),
                  FormBuilderValidators.max(widget.lastPage),
                ],
                valueTransformer: (text) {
                  return text == null ? null : int.tryParse(text);
                },
                onFieldSubmitted: (text) {
                  submitPageInput();
                },
                keyboardType: TextInputType.number,
              ),
            ),
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: FlatButton(
            child: const Text('GO'),
            onPressed: widget.lastPage < 2 ? null : submitPageInput,
          ),
        ),
      ],
    );
  }
}
