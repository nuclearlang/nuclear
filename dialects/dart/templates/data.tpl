{{decorators}}class {{name}} {
  const {{class_name}}({{{#fields}}required this.{{name}}{{^@last}}, {{/@last}}{{/fields}}});

{{#fields}}
  final {{type}} {{name}};
{{/fields}}

  {{class_name}} copyWith({{{#fields}}{{type_nullable}} {{name}}{{^@last}}, {{/@last}}{{/fields}}}) {
    return {{class_name}}({{#fields}}{{name}}: {{name}} ?? this.{{name}}{{^@last}}, {{/@last}}{{/fields}});
  }

  Map<String, dynamic> toJson() => {{{#fields}}'{{name}}': {{name}}{{^@last}}, {{/@last}}{{/fields}}};
{{#methods_code}}

  {{methods_code}}
{{/methods_code}}
}
