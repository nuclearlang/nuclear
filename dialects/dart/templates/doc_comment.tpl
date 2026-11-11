{{#summary}}
/// {{.}}
{{/summary}}
{{#has_tags}}
///
{{/has_tags}}
{{#params}}
/// * [{{name}}]: {{text}}
{{/params}}
{{#returns}}
/// * Returns {{returns}}
{{/returns}}
{{#throws}}
/// * Throws {{.}}
{{/throws}}
{{#see}}
/// * See {{.}}
{{/see}}
{{#since}}
/// * Since {{since}}
{{/since}}
{{#other}}
/// * {{tag}}: {{text}}
{{/other}}
{{#examples}}
///
{{#caption}}
/// {{caption}}:
{{/caption}}
/// ```nuclear
{{#lines}}
/// {{.}}
{{/lines}}
/// ```
{{/examples}}
{{#deprecated}}
@Deprecated({{deprecated_literal}})
{{/deprecated}}
