import 'dart:math' hide Comparable;
import 'package:fpdart/fpdart.dart';

// Examples of spec/nuclear-base.md: the same file is transpiled with typescript and with dart.

enum Role {
  user('user'),
  admin('administrator');

  const Role(this.value);

  final String value;
}

abstract class Message {
  String get role;
  String get content;
  String? get name;
}

class User {
  String name;
  String email;
  Role role;
  String? token;

  User(this.name, this.email, [this.role = Role.user]);

  bool isAdmin() {
    return role == Role.admin;
  }

  void rename(String newName) {
    this.name = newName;
  }
}

class Users {
  Map<String, String> get(String name) {
    return {'name': name};
  }
}

class Sales {
  Map<String, String> getSales(String name, String lastName) {
    print('sale by: $name');
    return Users().get(lastName);
  }
}

double addDoubles(double a, double b) {
  return a + b;
}

String greet(String name) {
  return 'Hello $name';
}

String classify(int n) {
  final label = n == 0 ? 'zero' : n == 1 || n == 2 ? 'small' : 'big';
  return label;
}

int sumUntil(int limit) {
  var total = 0;
  for (var i = 0; i < limit; i++) {
    if (i % 2 == 0) {
      total += i;
    } else if (i > 100) {
      break;
    }
  }
  return total;
}

/// if and match are values: the last line of each branch
String describe(User? user) {
  var result;
  if (user == null) {
    final prefix = 'no';
    result = prefix + 'body';
  } else {
    result = user.name;
  }
  return result;
}

Future<List<User>> loadUsers(List<String> ids) async {
  final users = [];
  for (final id in ids) {
    try {
      users.add(await fetchUser(id));
    } catch (error) {
      print(error);
    }
  }
  return users;
}

User fetchUser(String id) {
  return User(id, '$id@example.com');
}

run() {
  final doubles = [1, 2, 3].map((n) => n * 2).toList();
  final adults = List.generate(10 - 1, (i) => 1 + i).map((int i) => i * 2).toList();
  final log = (String message) => print(message);
  log('done $doubles $adults');
  final config = {'debug': true, 'retries': 3};
  for (final MapEntry(key: key, value: value) in config.entries) {
    print(key);
  }
  final ready = !(doubles.length == 0) && ['x'].contains('x');
  return ready;
}

/// destructuring: lists and maps
Map<String, int> bounds(List<int> xs) {
  return {'min': xs[0], 'max': xs[1]};
}

int unpack(List<int> xs) {
  final [head, ...tail] = xs;
  final {'min': min, 'max': max} = bounds(xs);
  return head + tail.length + min + max;
}

/// generators: yield
Iterable<int> countTo(int limit) sync* {
  for (var i = 0; i < limit; i++) {
    yield i;
  }
}

/// generics with a bound
abstract class Comparable {
  int compareTo(Comparable other);
}

T larger<T extends Comparable>(T a, T b) {
  return a.compareTo(b) > 0 ? a : b;
}

/// if inside a list
List<String> labels(bool loading) {
  return ['start', if (loading) 'loading', 'end'];
}

/// results: errors propagate on their own
Either<String, int> parseAge(String text) {
  if (text == '') {
    return Left('empty');
  }
  return Right(text.length);
}

Either<String, String> describeAge(String text) {
  final ageResult = parseAge(text);
  if (ageResult case Left(:final value)) return Left(value);
  final age = (ageResult as Right<String, int>).value;
  final safeResult = parseAge('0').orElse((e) => Right(0));
  if (safeResult case Left(:final value)) return Left(value);
  final safe = (safeResult as Right<dynamic, int>).value;
  return Right('${age + safe}');
}

String ageLabel(String text) {
  final describeAgeResult = describeAge(text);
  if (describeAgeResult.isRight()) {
    return (describeAgeResult as Right<String, String>).value;
  } else {
    return 'error: ${(describeAgeResult as Left<String, String>).value}';
  }
}

/// alias: ageText is the same function as Ages.text
class Ages {
  String text(int age) {
    return '$age years';
  }
}

class Profile {
  String ageText(int age) {
    return Ages().text(age);
  }
}
