//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:async';
import 'dart:typed_data';
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:tanuki/core/domain/usecases/upload_asset.dart';

//
// events
//

abstract class UploadFileEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class StartUploading<T> extends UploadFileEvent {
  final List<T> files;

  StartUploading({required this.files});

  @override
  List<Object> get props => [files];
}

class UploadFile extends UploadFileEvent {
  final String filename;
  final Uint8List contents;

  UploadFile({required this.filename, required this.contents});

  @override
  List<Object> get props => [filename];
}

// Something went wrong with the file, move on to the next.
class SkipCurrent extends UploadFileEvent {}

//
// states
//

abstract class UploadFileState extends Equatable {
  @override
  List<Object> get props => [];
}

class Initial extends UploadFileState {}

class Uploading<T extends Object> extends UploadFileState {
  final List<T> pending;
  final T current;
  final int uploaded;

  Uploading({
    required pending,
    required this.current,
    this.uploaded = 0,
  }) : pending = List.unmodifiable(pending);

  @override
  List<Object> get props => [current, uploaded];
}

class Finished<T> extends UploadFileState {
  final List<T> skipped;

  Finished({required skipped}) : skipped = List.unmodifiable(skipped);

  @override
  List<Object> get props => [skipped];
}

class Error extends UploadFileState {
  final String message;

  Error({required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class UploadFileBloc<T extends Object>
    extends Bloc<UploadFileEvent, UploadFileState> {
  final UploadAsset usecase;
  // those file that were skipped along the way
  final List<T> skipped = [];
  // queue of files waiting to be uploaded
  late List<T> pending;
  // current file being processed
  late T current;
  late int uploaded;

  UploadFileBloc({required this.usecase}) : super(Initial());

  @override
  Stream<UploadFileState> mapEventToState(
    UploadFileEvent event,
  ) async* {
    if (event is StartUploading) {
      // Start the process of uploading the files, yielding a state that
      // indicates the caller should load the current file.
      skipped.clear();
      uploaded = 0;
      current = event.files.last;
      pending = event.files.sublist(0, event.files.length - 1) as List<T>;
      yield Uploading<T>(pending: pending, current: current);
    } else if (event is UploadFile) {
      // Caller has loaded the current file and is ready to upload.
      final result = await usecase(Params(
        filename: event.filename,
        contents: event.contents,
      ));
      yield result.mapOrElse(
        (assetId) => _processNext(),
        (failure) => Error(message: failure.toString()),
      );
    } else if (event is SkipCurrent) {
      // Something probably went wrong with loading the file, caller wants to
      // skip it and move on to the next file.
      skipped.add(current);
      yield _processNext();
    }
  }

  UploadFileState _processNext() {
    // Continue processing the remaining files, or signal that the end has
    // been reached if the pending queue is now empty.
    if (pending.isEmpty) {
      return Finished<T>(skipped: skipped);
    } else {
      uploaded++;
      current = pending.removeLast();
      return Uploading<T>(
          pending: pending, current: current, uploaded: uploaded);
    }
  }
}
