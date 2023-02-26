//
// Copyright (c) 2023 Nathan Fiedler
//
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

class StartUploading extends UploadFileEvent {
  final List files;

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

class Uploading extends UploadFileState {
  final List pending;
  final dynamic current;
  final int uploaded;

  Uploading({
    required pending,
    required this.current,
    this.uploaded = 0,
  }) : pending = List.unmodifiable(pending);

  @override
  List<Object> get props => [current, uploaded];

  // There is always one active upload, plus those waiting.
  int get active => pending.length + 1;
}

class Finished extends UploadFileState {
  final List skipped;

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

class UploadFileBloc extends Bloc<UploadFileEvent, UploadFileState> {
  final UploadAsset usecase;
  // those file that were skipped along the way
  final List skipped = [];
  // queue of files waiting to be uploaded
  late List pending;
  // current file being processed
  late dynamic current;
  late int uploaded;

  UploadFileBloc({required this.usecase}) : super(Initial()) {
    on<StartUploading>((event, emit) {
      // Start the process of uploading the files, yielding a state that
      // indicates the caller should load the current file.
      skipped.clear();
      uploaded = 0;
      current = event.files.last;
      pending = event.files.sublist(0, event.files.length - 1);
      emit(Uploading(pending: pending, current: current));
    });
    on<UploadFile>((event, emit) async {
      // Caller has loaded the current file and is ready to upload.
      final result = await usecase(Params(
        filename: event.filename,
        contents: event.contents,
      ));
      emit(result.mapOrElse(
        (assetId) => _processNext(),
        (failure) => Error(message: failure.toString()),
      ));
    });
    on<SkipCurrent>((event, emit) {
      // Something probably went wrong with loading the file, caller wants to
      // skip it and move on to the next file.
      skipped.add(current);
      emit(_processNext());
    });
  }

  UploadFileState _processNext() {
    // Continue processing the remaining files, or signal that the end has been
    // reached if the pending queue is now empty.
    if (pending.isEmpty) {
      return Finished(skipped: skipped);
    } else {
      uploaded++;
      current = pending.removeLast();
      return Uploading(pending: pending, current: current, uploaded: uploaded);
    }
  }
}
