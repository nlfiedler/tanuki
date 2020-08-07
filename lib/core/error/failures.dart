//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:equatable/equatable.dart';

abstract class Failure extends Equatable {
  final String message;

  const Failure([this.message]);

  @override
  List<Object> get props => [message];

  @override
  bool get stringify => true;
}

// General failures
class ServerFailure extends Failure {
  ServerFailure([var message]) : super(message);
}

class ValidationFailure extends Failure {
  ValidationFailure([var message]) : super(message);
}
