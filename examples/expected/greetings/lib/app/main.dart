import 'package:fpdart/fpdart.dart';
import 'package:greetings/users/users.dart';

void main() {
  final greetResult = greet('ana');
  if (greetResult.isRight()) {
    print((greetResult as Right<String, String>).value);
  } else {
    print((greetResult as Left<String, String>).value);
  }
}
