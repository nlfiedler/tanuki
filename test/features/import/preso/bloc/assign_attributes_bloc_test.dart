//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/features/import/preso/bloc/assign_attributes_bloc.dart';

void main() {
  setUpAll(() {
    // mocktail needs a fallback for any() that involves custom types
    const List<AssetInputId> dummy = [];
    registerFallbackValue(dummy);
  });

  group('normal cases', () {
    blocTest(
      'emits [] when nothing is added',
      build: () => AssignAttributesBloc(),
      expect: () => [],
    );

    blocTest(
      'emits [AssignAttributesState] when AssignTags is added',
      build: () => AssignAttributesBloc(),
      act: (AssignAttributesBloc bloc) =>
          bloc.add(AssignTags(tags: const ['cat'])),
      expect: () => [
        AssignAttributesState(tags: const ['cat']),
      ],
    );

    blocTest(
      'emits [AssignAttributesState] when AssignLocation is added',
      build: () => AssignAttributesBloc(),
      act: (AssignAttributesBloc bloc) =>
          bloc.add(AssignLocation(location: 'hawaii')),
      expect: () => [
        AssignAttributesState(location: 'hawaii'),
      ],
    );

    blocTest(
      'emits [AssignAttributesState(x2)] when ToggleAsset, ToggleAsset is added',
      build: () => AssignAttributesBloc(),
      act: (AssignAttributesBloc bloc) {
        bloc.add(ToggleAsset(assetId: 'abc123'));
        bloc.add(ToggleAsset(assetId: 'abc123'));
      },
      expect: () => [
        AssignAttributesState(assets: const {'abc123'}),
        AssignAttributesState(),
      ],
    );

    blocTest(
      'emits [AssignAttributesState(x3)] when AssignTags, AssignLocatin, ToggleAsset is added',
      build: () => AssignAttributesBloc(),
      act: (AssignAttributesBloc bloc) {
        bloc.add(AssignTags(tags: const ['cat']));
        bloc.add(AssignLocation(location: 'hawaii'));
        bloc.add(ToggleAsset(assetId: 'abc123'));
      },
      expect: () => [
        AssignAttributesState(tags: const ['cat']),
        AssignAttributesState(tags: const ['cat'], location: 'hawaii'),
        AssignAttributesState(
          tags: const ['cat'],
          location: 'hawaii',
          assets: const {'abc123'},
        ),
      ],
    );

    test('submittable indicates completeness of AssignAttributesState', () {
      expect(AssignAttributesState().submittable, false);
      expect(AssignAttributesState(tags: const ['cat']).submittable, false);
      expect(AssignAttributesState(location: 'hawaii').submittable, false);
      expect(
          AssignAttributesState(tags: const ['cat'], location: 'hawaii')
              .submittable,
          false);
      expect(
          AssignAttributesState(
              tags: const ['cat'],
              location: 'hawaii',
              assets: const {'abc123'}).submittable,
          true);
      expect(
          AssignAttributesState(tags: const ['cat'], assets: const {'abc123'})
              .submittable,
          true);
    });
  });
}
