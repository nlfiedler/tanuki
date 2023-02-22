//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:equatable/equatable.dart';

abstract class Failure extends Equatable {
  final String message;

  const Failure({required this.message});

  @override
  List<Object> get props => [message];

  @override
  bool get stringify => true;
}

// General failures
class ServerFailure extends Failure {
  const ServerFailure(String message) : super(message: message);
}

class ValidationFailure extends Failure {
  const ValidationFailure(String message) : super(message: message);
}
