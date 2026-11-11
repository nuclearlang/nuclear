{{#one_line}}
"""{{#summary}}{{.}}{{/summary}}"""
{{/one_line}}
{{^one_line}}
"""
{{#summary}}
{{.}}
{{/summary}}
{{#params}}
{{#@first}}

Args:
{{/@first}}
    {{name}}: {{text}}
{{/params}}
{{#returns}}

Returns:
    {{returns}}
{{/returns}}
"""
{{/one_line}}
