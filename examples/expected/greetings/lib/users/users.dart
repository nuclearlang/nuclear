import 'package:fpdart/fpdart.dart';
import 'package:greetings/users/user_store.dart';

class Users {
  Either<String, User> find(String name) {
    return UserStore().find(name);
  }
}

Either<String, String> greet(String name) {
  final userResult = Users().find(name);
  if (userResult case Left(:final value)) return Left(value);
  final user = (userResult as Right<String, User>).value;
  return Right('hello ${user.name}');
}
