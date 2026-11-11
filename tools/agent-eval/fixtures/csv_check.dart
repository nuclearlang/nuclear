// Checks the agent's csv.dart (copied next to it).
import 'csv.dart';

void main() {
  final rows = parseCsv('name,city\nAna,"Bogotá, CO"\nLuis,Lima');
  final ok = rows.length == 2 && rows[0]['city'] == 'Bogotá, CO' && rows[1]['name'] == 'Luis';
  if (!ok) throw StateError('wrong results: $rows');
  print('ok');
}
