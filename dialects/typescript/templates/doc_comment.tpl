{{#one_line}}
/** {{summary}} */
{{/one_line}}
{{^one_line}}
/**
{{#summary}}
 * {{.}}
{{/summary}}
{{#has_tags}}
 *
{{/has_tags}}
{{#deprecated}}
 * @deprecated {{deprecated_text}}
{{/deprecated}}
{{#params}}
 * @param {{name}} {{text}}
{{/params}}
{{#returns}}
 * @returns {{returns}}
{{/returns}}
{{#throws}}
 * @throws {{.}}
{{/throws}}
{{#examples}}
 * @example{{#caption}} {{caption}}{{/caption}}
{{#lines}}
 * {{.}}
{{/lines}}
{{/examples}}
{{#see}}
 * @see {{.}}
{{/see}}
{{#since}}
 * @since {{since}}
{{/since}}
{{#other}}
 * @{{tag}} {{text}}
{{/other}}
 */
{{/one_line}}
