use crate::generator::GeneratorConfig;
use crate::meta;
use crate::registry::TypeMeta;
use crate::utils::snake_to_camel;

struct FieldRenderingInfo {
    name: String,
    ty_name: String,
    optional: bool,
}

fn get_field_rendering_info(f: &meta::FieldMeta, config: GeneratorConfig) -> FieldRenderingInfo {
    let raw_ty = (f.ty_name)();
    if f.optional {
        FieldRenderingInfo {
            name: format!("{}?", f.name),
            ty_name: raw_ty,
            optional: true,
        }
    } else if config.option_fields_as_optional && raw_ty.ends_with(" | null") && !f.nullable {
        let clean_ty = raw_ty[..raw_ty.len() - 7].to_string();
        FieldRenderingInfo {
            name: format!("{}?", f.name),
            ty_name: clean_ty,
            optional: true,
        }
    } else {
        FieldRenderingInfo {
            name: f.name.to_string(),
            ty_name: raw_ty,
            optional: false,
        }
    }
}

pub fn render_type(meta: &TypeMeta, all_metas: &[&TypeMeta], config: GeneratorConfig) -> String {
    let structure = (meta.structure)();
    let def = match structure {
        meta::TypeStructure::Struct(s) => {
            let mut fields = Vec::new();
            collect_fields_from_list(s.fields, all_metas, &mut fields);
            let fields_str = fields
                .iter()
                .map(|f| {
                    let info = get_field_rendering_info(f, config);
                    format!("{}: {}", info.name, info.ty_name)
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "export type {}{} = {{ {} }}",
                meta.name, meta.generic_params, fields_str
            )
        }
        meta::TypeStructure::Tuple(types) => {
            if types.len() == 1 {
                format!(
                    "export type {}{} = {}",
                    meta.name,
                    meta.generic_params,
                    (types[0])()
                )
            } else {
                let inner = types.iter().map(|t| t()).collect::<Vec<_>>().join(", ");
                format!(
                    "export type {}{} = [{}]",
                    meta.name, meta.generic_params, inner
                )
            }
        }
        meta::TypeStructure::Enum(e) => {
            let enum_def = if !e.untagged
                && e.variants
                    .iter()
                    .all(|v| matches!(v.fields, meta::VariantFields::Unit))
            {
                let variant_names: Vec<String> =
                    e.variants.iter().map(|v| v.name.to_string()).collect();
                let fields = variant_names
                    .iter()
                    .map(|n| format!("  {}: \"{}\"", n, n))
                    .collect::<Vec<_>>()
                    .join(",\n");
                format!(
                    "export const {} = {{\n{}\n}} as const;\nexport type {}{} = (typeof {})[keyof typeof {}]",
                    meta.name, fields, meta.name, meta.generic_params, meta.name, meta.name
                )
            } else {
                let variants: Vec<String> = e
                    .variants
                    .iter()
                    .map(|v| render_variant(v, &e, all_metas, config))
                    .collect();
                format!(
                    "export type {}{} = {}",
                    meta.name,
                    meta.generic_params,
                    variants.join(" | ")
                )
            };

            enum_def
        }
        meta::TypeStructure::Transparent(inner) => {
            format!(
                "export type {}{} = {}",
                meta.name, meta.generic_params, inner
            )
        }
        meta::TypeStructure::Unit => {
            format!("export type {} = null", meta.name)
        }
    };

    let mut result = def;
    if let Some(schema_str) = render_zod_schema(meta, all_metas, config) {
        result = format!("{}\n{}", result, schema_str);
    }

    if let Some(transformer) = render_transformer(meta, all_metas) {
        format!("{}\n{}", result, transformer)
    } else {
        result
    }
}
#[allow(dead_code)]
pub fn has_binary_data(structure: &meta::TypeStructure, all_metas: &[&TypeMeta]) -> bool {
    match structure {
        meta::TypeStructure::Struct(s) => {
            let mut fields = Vec::new();
            collect_fields_from_list(s.fields, all_metas, &mut fields);
            fields.iter().any(|f| f.is_binary)
        }
        meta::TypeStructure::Enum(e) => {
            e.variants.iter().any(|v| match &v.fields {
                meta::VariantFields::Named(fields) => {
                    let mut all_fields = Vec::new();
                    collect_fields_from_list(fields, all_metas, &mut all_fields);
                    all_fields.iter().any(|f| f.is_binary)
                }
                _ => false, // Tuple hydration deferred to v0.3.0
            })
        }
        _ => false,
    }
}

pub fn render_transformer(meta: &TypeMeta, all_metas: &[&TypeMeta]) -> Option<String> {
    if !meta.has_binary {
        return None;
    }
    let structure = (meta.structure)();

    match structure {
        meta::TypeStructure::Struct(s) => {
            let mut fields = Vec::new();
            collect_fields_from_list(s.fields, all_metas, &mut fields);
            let binary_fields: Vec<_> = fields.iter().filter(|f| f.is_binary).collect();

            let mut transformations = Vec::new();
            for f in binary_fields {
                transformations.push(format!(
                    "  (res as any).{} = toBinary((res as any).{});",
                    f.name, f.name
                ));
            }

            Some(format!(
                "export function to{}(res: {}): {} {{\n{}\n  return res;\n}}",
                meta.name,
                meta.name,
                meta.name,
                transformations.join("\n")
            ))
        }
        _ => None, // Enum transformer deferred
    }
}

pub fn render_enum_meta(type_meta: &TypeMeta, e: &meta::EnumMeta) -> Option<String> {
    if e.variants.iter().all(|v| v.attrs.is_empty()) {
        return None;
    }

    let meta_name = e
        .meta_name
        .unwrap_or_else(|| Box::leak(format!("{}Meta", type_meta.name).into_boxed_str()));

    let mut lines = Vec::new();
    for v in e.variants {
        let mut attrs: Vec<_> = v.attrs.iter().collect();
        attrs.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

        let mut attrs_str_parts = Vec::new();
        for (k, val) in attrs {
            let k_camel = snake_to_camel(k);
            let val_rendered = match val {
                meta::AttrValue::Str(s) => format!("\"{}\"", s),
                meta::AttrValue::List(items) => {
                    let ts_items: Vec<String> =
                        items.iter().map(|item| item.replace("::", ".")).collect();
                    format!("[{}]", ts_items.join(", "))
                }
            };
            attrs_str_parts.push(format!("{}: {}", k_camel, val_rendered));
        }

        let attrs_str = attrs_str_parts.join(", ");
        if attrs_str.is_empty() {
            lines.push(format!("  {}: {{}}", v.name));
        } else {
            lines.push(format!("  {}: {{ {} }}", v.name, attrs_str));
        }
    }

    Some(format!(
        "export const {} = {{\n{}\n}} as const;",
        meta_name,
        lines.join(",\n")
    ))
}

pub fn collect_fields_from_list(
    fields: &'static [meta::FieldMeta],
    all_metas: &[&TypeMeta],
    out: &mut Vec<&'static meta::FieldMeta>,
) {
    for field in fields {
        if field.flattened {
            if let Some(base_name) = field.flatten_base_name {
                match all_metas.iter().find(|m| m.name == base_name) {
                    Some(target) => {
                        if let meta::TypeStructure::Struct(inner_s) = (target.structure)() {
                            collect_fields_from_list(inner_s.fields, all_metas, out);
                        }
                    }
                    None => {
                        eprintln!(
                            "[tyzen] warning: could not resolve flattened type `{}` — its fields will be missing from generated TypeScript",
                            base_name
                        );
                    }
                }
            }
        } else {
            out.push(field);
        }
    }
}

pub fn render_variant(
    v: &meta::VariantMeta,
    e: &meta::EnumMeta,
    all_metas: &[&TypeMeta],
    config: GeneratorConfig,
) -> String {
    let tag_name = e.tag.unwrap_or("tag");
    let content_name = e.content;

    match &v.fields {
        meta::VariantFields::Unit => {
            if e.untagged {
                "null".to_string()
            } else {
                format!("{{ {}: \"{}\" }}", tag_name, v.name)
            }
        }
        meta::VariantFields::Unnamed(types) => {
            let values_ts = if types.len() == 1 {
                (types[0])()
            } else {
                let inner = types.iter().map(|t| t()).collect::<Vec<_>>().join(", ");
                format!("[{}]", inner)
            };

            if e.untagged {
                values_ts
            } else if let Some(content) = content_name {
                format!(
                    "{{ {}: \"{}\", {}: {} }}",
                    tag_name, v.name, content, values_ts
                )
            } else {
                format!("{{ {}: \"{}\", value: {} }}", tag_name, v.name, values_ts)
            }
        }
        meta::VariantFields::Named(fields) => {
            let mut all_fields = Vec::new();
            collect_fields_from_list(fields, all_metas, &mut all_fields);

            let fields_str = all_fields
                .iter()
                .map(|f| {
                    let info = get_field_rendering_info(f, config);
                    format!("{}: {}", info.name, info.ty_name)
                })
                .collect::<Vec<_>>()
                .join(", ");

            if e.untagged {
                format!("{{ {} }}", fields_str)
            } else if let Some(content) = content_name {
                format!(
                    "{{ {}: \"{}\", {}: {{ {} }} }}",
                    tag_name, v.name, content, fields_str
                )
            } else {
                format!("{{ {}: \"{}\", {} }}", tag_name, v.name, fields_str)
            }
        }
    }
}

pub fn render_zod_schema(
    meta: &TypeMeta,
    all_metas: &[&TypeMeta],
    config: GeneratorConfig,
) -> Option<String> {
    if !meta.schema {
        return None;
    }

    let structure = (meta.structure)();
    let schema_name = format!("{}Schema", pascal_to_camel(meta.name));

    match structure {
        meta::TypeStructure::Struct(s) => {
            let mut fields = Vec::new();
            collect_fields_from_list(s.fields, all_metas, &mut fields);
            let mut field_schemas = Vec::new();
            for f in fields {
                let schema_str = render_field_zod(f, all_metas, config);
                field_schemas.push(format!("  {}: {}", f.name, schema_str));
            }
            Some(format!(
                "export const {} = z.object({{\n{}\n}});\nexport type {}Schema = z.infer<typeof {}>;",
                schema_name,
                field_schemas.join(",\n"),
                meta.name,
                schema_name
            ))
        }
        meta::TypeStructure::Enum(e) => {
            // All unit variants → z.enum([...])
            if !e.untagged
                && e.variants
                    .iter()
                    .all(|v| matches!(v.fields, meta::VariantFields::Unit))
            {
                let variant_strs: Vec<String> = e
                    .variants
                    .iter()
                    .map(|v| format!("\"{}\"", v.name))
                    .collect();
                return Some(format!(
                    "export const {} = z.enum([{}]);\nexport type {}Schema = z.infer<typeof {}>;" ,
                    schema_name,
                    variant_strs.join(", "),
                    meta.name,
                    schema_name
                ));
            }

            // Complex enum: build per-variant schemas
            let variant_schemas: Vec<String> = e
                .variants
                .iter()
                .map(|v| render_variant_as_zod_schema(v, &e, all_metas, config))
                .collect();

            let zod_schema = if let Some(tag_name) = e.tag {
                // Internally or adjacently tagged → z.discriminatedUnion
                if variant_schemas.len() >= 2 {
                    format!(
                        "z.discriminatedUnion(\"{}\", [{}])",
                        tag_name,
                        variant_schemas.join(", ")
                    )
                } else if let Some(s) = variant_schemas.into_iter().next() {
                    s
                } else {
                    "z.never()".to_string()
                }
            } else {
                // Untagged or external-tagged → z.union
                if variant_schemas.len() >= 2 {
                    format!("z.union([{}])", variant_schemas.join(", "))
                } else if let Some(s) = variant_schemas.into_iter().next() {
                    s
                } else {
                    "z.never()".to_string()
                }
            };

            Some(format!(
                "export const {} = {};\nexport type {}Schema = z.infer<typeof {}>;" ,
                schema_name, zod_schema, meta.name, schema_name
            ))
        }
        meta::TypeStructure::Tuple(types) => {
            if types.is_empty() {
                return None;
            }
            let zod_schema = if types.len() == 1 {
                map_type_to_zod_base(&(types[0])(), all_metas)
            } else {
                let inner: Vec<String> = types
                    .iter()
                    .map(|t| map_type_to_zod_base(&t(), all_metas))
                    .collect();
                format!("z.tuple([{}])", inner.join(", "))
            };
            Some(format!(
                "export const {} = {};\nexport type {}Schema = z.infer<typeof {}>;" ,
                schema_name, zod_schema, meta.name, schema_name
            ))
        }
        meta::TypeStructure::Transparent(inner) => {
            let inner_schema = map_type_to_zod_base(&inner, all_metas);
            Some(format!(
                "export const {} = {};\nexport type {}Schema = z.infer<typeof {}>;" ,
                schema_name, inner_schema, meta.name, schema_name
            ))
        }
        meta::TypeStructure::Unit => Some(format!(
            "export const {} = z.null();\nexport type {}Schema = z.infer<typeof {}>;" ,
            schema_name, meta.name, schema_name
        )),
    }
}

fn pascal_to_camel(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap().to_lowercase().to_string();
    let rest: String = chars.collect();
    format!("{}{}", first, rest)
}

/// Generates a Zod schema string for a single enum variant,
/// respecting the enum's tagging strategy (untagged, internally tagged, adjacently tagged, external).
fn render_variant_as_zod_schema(
    v: &meta::VariantMeta,
    e: &meta::EnumMeta,
    all_metas: &[&TypeMeta],
    config: GeneratorConfig,
) -> String {
    let tag_name = e.tag;
    let content_name = e.content;
    let is_untagged = e.untagged;

    match &v.fields {
        meta::VariantFields::Unit => {
            if is_untagged {
                "z.null()".to_string()
            } else if let Some(tag) = tag_name {
                // Internally or adjacently tagged unit: only tag field needed
                format!("z.object({{ {}: z.literal(\"{}\") }})", tag, v.name)
            } else {
                // External tagging: unit variant → string literal
                format!("z.literal(\"{}\")" , v.name)
            }
        }
        meta::VariantFields::Unnamed(types) => {
            let payload = if types.len() == 1 {
                map_type_to_zod_base(&(types[0])(), all_metas)
            } else {
                let inner: Vec<String> = types
                    .iter()
                    .map(|t| map_type_to_zod_base(&t(), all_metas))
                    .collect();
                format!("z.tuple([{}])", inner.join(", "))
            };

            if is_untagged {
                payload
            } else if let Some(tag) = tag_name {
                if let Some(content) = content_name {
                    // Adjacently tagged
                    format!(
                        "z.object({{ {}: z.literal(\"{}\"), {}: {} }})",
                        tag, v.name, content, payload
                    )
                } else {
                    // Internally tagged with unnamed payload → value key
                    format!(
                        "z.object({{ {}: z.literal(\"{}\"), value: {} }})",
                        tag, v.name, payload
                    )
                }
            } else {
                // External tagging
                format!("z.object({{ \"{}\": {} }})", v.name, payload)
            }
        }
        meta::VariantFields::Named(fields) => {
            let mut all_fields = Vec::new();
            collect_fields_from_list(fields, all_metas, &mut all_fields);

            let field_schemas: Vec<String> = all_fields
                .iter()
                .map(|f| {
                    let schema = render_field_zod(f, all_metas, config);
                    format!("{}: {}", f.name, schema)
                })
                .collect();
            let fields_str = field_schemas.join(", ");

            if is_untagged {
                format!("z.object({{ {} }})", fields_str)
            } else if let Some(tag) = tag_name {
                if let Some(content) = content_name {
                    // Adjacently tagged: named fields go into content object
                    format!(
                        "z.object({{ {}: z.literal(\"{}\"), {}: z.object({{ {} }}) }})",
                        tag, v.name, content, fields_str
                    )
                } else {
                    // Internally tagged: fields merged with tag field
                    format!(
                        "z.object({{ {}: z.literal(\"{}\"), {} }})",
                        tag, v.name, fields_str
                    )
                }
            } else {
                // External tagging
                format!("z.object({{ \"{}\": z.object({{ {} }}) }})", v.name, fields_str)
            }
        }
    }
}

/// Finds the index of the first top-level comma in a string,
/// ignoring commas nested inside `<...>` brackets.
/// Used to split `Record<K, V>` into key and value types.
fn find_top_level_comma(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

fn render_field_zod(
    field: &meta::FieldMeta,
    all_metas: &[&TypeMeta],
    config: GeneratorConfig,
) -> String {
    let info = get_field_rendering_info(field, config);
    let clean_ty = info.ty_name.trim();

    let is_nullable = clean_ty.ends_with(" | null");
    let base_ty = if is_nullable {
        &clean_ty[..clean_ty.len() - 7]
    } else {
        clean_ty
    };

    let mut base = if field.name.to_lowercase().contains("date") {
        "z.union([z.string(), z.date()])".to_string()
    } else {
        map_type_to_zod_base(base_ty, all_metas)
    };

    if let Some(ref val) = field.validation {
        if let Some(min) = val.min_length {
            let msg = match val.min_length_message {
                Some(m) => format!(", {{ message: \"{}\" }}", m),
                None => "".to_string(),
            };
            base = format!("{}.min({}{})", base, min, msg);
        }
        if let Some(max) = val.max_length {
            let msg = match val.max_length_message {
                Some(m) => format!(", {{ message: \"{}\" }}", m),
                None => "".to_string(),
            };
            base = format!("{}.max({}{})", base, max, msg);
        }
        if let Some(pattern) = val.regex_pattern {
            let msg = match val.regex_message {
                Some(m) => format!(", {{ message: \"{}\" }}", m),
                None => "".to_string(),
            };
            let pattern_formatted = if pattern.starts_with('/') && pattern.ends_with('/') {
                pattern.to_string()
            } else {
                format!("/{}/", pattern)
            };
            base = format!("{}.regex({}{})", base, pattern_formatted, msg);
        }
        if let Some(min) = val.min_value {
            let msg = match val.min_value_message {
                Some(m) => format!(", {{ message: \"{}\" }}", m),
                None => "".to_string(),
            };
            base = format!("{}.min({}{})", base, min, msg);
        }
        if let Some(max) = val.max_value {
            let msg = match val.max_value_message {
                Some(m) => format!(", {{ message: \"{}\" }}", m),
                None => "".to_string(),
            };
            base = format!("{}.max({}{})", base, max, msg);
        }
        if val.email {
            let msg = match val.email_message {
                Some(m) => format!("{{ message: \"{}\" }}", m),
                None => "".to_string(),
            };
            base = format!("{}.email({})", base, msg);
        }
        if val.url {
            let msg = match val.url_message {
                Some(m) => format!("{{ message: \"{}\" }}", m),
                None => "".to_string(),
            };
            base = format!("{}.url({})", base, msg);
        }
        if let Some(c) = val.contains {
            let msg = match val.contains_message {
                Some(m) => format!(", {{ message: \"{}\" }}", m),
                None => "".to_string(),
            };
            base = format!("{}.includes(\"{}\"{})", base, c, msg);
        }
        if let Some(c) = val.does_not_contain {
            let msg_str = match val.does_not_contain_message {
                Some(m) => format!("\"{}\"", m),
                None => format!("\"Value must not contain '{}'\"", c),
            };
            base = format!(
                "{}.refine(v => !v.includes(\"{}\"), {{ message: {} }})",
                base, c, msg_str
            );
        }
        if let Some(f) = val.custom_fn {
            let msg_str = match val.custom_message {
                Some(m) => format!("\"{}\"", m),
                None => format!("\"validate with {}\"", f),
            };
            base = format!(
                "{}.refine(() => true, {{ message: {} }})",
                base, msg_str
            );
        }
    }

    if is_nullable {
        if info.optional {
            if field.nullable {
                base = format!("{}.nullable().optional()", base);
            } else {
                base = format!("{}.optional()", base);
            }
        } else {
            base = format!("{}.nullable().optional()", base);
        }
    } else if info.optional {
        base = format!("{}.optional()", base);
    }

    base
}

fn map_type_to_zod_base(clean_ty: &str, all_metas: &[&TypeMeta]) -> String {
    match clean_ty {
        "string" => "z.string()".to_string(),
        "number" => "z.number()".to_string(),
        "boolean" => "z.boolean()".to_string(),
        "null" => "z.null()".to_string(),
        "any" => "z.any()".to_string(),
        "Uint8Array" => "z.instanceof(Uint8Array)".to_string(),
        "Date" => "z.coerce.date()".to_string(),
        _ => {
            if clean_ty.ends_with("[]") {
                let inner = &clean_ty[..clean_ty.len() - 2];
                let inner_zod = map_type_to_zod_base(inner, all_metas);
                return format!("z.array({})", inner_zod);
            }

            // Record<K, V> from HashMap / BTreeMap
            if clean_ty.starts_with("Record<") && clean_ty.ends_with('>') {
                let inner = &clean_ty[7..clean_ty.len() - 1];
                if let Some(comma_pos) = find_top_level_comma(inner) {
                    let key_ty = inner[..comma_pos].trim();
                    let val_ty = inner[comma_pos + 1..].trim();
                    let key_schema = map_type_to_zod_base(key_ty, all_metas);
                    let val_schema = map_type_to_zod_base(val_ty, all_metas);
                    return format!("z.record({}, {})", key_schema, val_schema);
                }
            }

            if let Some(target) = all_metas.iter().find(|m| m.name == clean_ty) {
                let structure = (target.structure)();
                match structure {
                    meta::TypeStructure::Enum(ref e) => {
                        if !e.untagged
                            && e.variants
                                .iter()
                                .all(|v| matches!(v.fields, meta::VariantFields::Unit))
                        {
                            let variant_strs: Vec<String> = e
                                .variants
                                .iter()
                                .map(|v| format!("\"{}\"", v.name))
                                .collect();
                            return format!("z.enum([{}])", variant_strs.join(", "));
                        }
                    }
                    _ => {}
                }

                if target.schema {
                    return format!("{}Schema", pascal_to_camel(target.name));
                }
            }

            if clean_ty.contains("Date") || clean_ty.contains("DateTime") {
                return "z.coerce.date()".to_string();
            }

            eprintln!(
                "[tyzen] warning: no Zod mapping for type `{}`, falling back to z.any(). \
                 Consider adding `#[tyzen(schema)]` to that type.",
                clean_ty
            );
            "z.any()".to_string()
        }
    }
}
