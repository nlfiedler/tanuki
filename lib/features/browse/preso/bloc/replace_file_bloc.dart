//
// Copyright (c) 2024 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:tanuki/core/domain/usecases/replace_asset.dart';

//
// events
//

abstract class ReplaceFileEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class StartUploading extends ReplaceFileEvent {
  /// The file object from the file chooser
  final dynamic file;

  StartUploading({required this.file});

  @override
  List<Object> get props => [file];
}

class ReplaceFile extends ReplaceFileEvent {
  final String assetId;
  final String filename;
  final Uint8List contents;

  ReplaceFile({
    required this.assetId,
    required this.filename,
    required this.contents,
  });

  @override
  List<Object> get props => [assetId];
}

// Something went wrong with retreiving the file, abandon the upload.
class SkipCurrent extends ReplaceFileEvent {}

//
// states
//

abstract class ReplaceFileState extends Equatable {
  @override
  List<Object> get props => [];
}

class Initial extends ReplaceFileState {}

class Uploading extends ReplaceFileState {
  final dynamic current;

  Uploading({required this.current});

  @override
  List<Object> get props => [current];
}

class Finished extends ReplaceFileState {
  /// The new asset identifier after being replaced.
  final String assetId;

  Finished({required this.assetId});

  @override
  List<Object> get props => [assetId];
}

class Error extends ReplaceFileState {
  final String message;

  Error({required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class ReplaceFileBloc extends Bloc<ReplaceFileEvent, ReplaceFileState> {
  final ReplaceAsset usecase;
  // current file being processed
  late dynamic current;

  ReplaceFileBloc({required this.usecase}) : super(Initial()) {
    on<StartUploading>((event, emit) {
      emit(Uploading(current: event.file));
    });
    on<ReplaceFile>((event, emit) async {
      // Caller has loaded the current file and is ready to upload.
      final result = await usecase(Params(
        assetId: event.assetId,
        filename: event.filename,
        contents: event.contents,
      ));
      emit(result.mapOrElse(
        (assetId) => Finished(assetId: assetId),
        (failure) => Error(message: failure.toString()),
      ));
    });
    on<SkipCurrent>((event, emit) {
      // something probably went wrong with loading the file
      emit(Initial());
    });
  }
}
