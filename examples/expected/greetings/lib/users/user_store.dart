import 'package:fpdart/fpdart.dart';

class User {
  String name;
  int age;

  User(this.name, this.age);
}

class UserStore {
  List<User> users = [];

  Either<String, User> find(String name) {
    final found = users.where((u) => u.name == name).toList();
    if (found.isEmpty) {
      return Left('no user $name');
    }
    return Right(found.first);
  }
}
