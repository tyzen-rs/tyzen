use crate::meta;
use crate::registry::TypeMeta;
use crate::utils::snake_to_camel;

pub fn render_type(meta: &TypeMeta, all_metas: &[&TypeMeta]) -> String {
    let structure = (meta.structure)();
    let def = match structure {
        meta::TypeStructure::Struct(s) => {
            let mut fields = Vec::new();
            collect_fields_from_list(s.fields, all_metas, &mut fields);
            let fields_str = fields
                .iter()
                .map(|f| {
                    let label = if f.optional {
                        format!("{}?", f.name)
                    } else {
                        f.name.to_string()
                    };
                    format!("{}: {}", label, (f.ty_name)())
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("export type {}{} = {{ {} }}", meta.name, meta.generic_params, fields_str)
        }
        meta::TypeStructure::Tuple(types) => {
            if types.len() == 1 {
                format!("export type {}{} = {}", meta.name, meta.generic_params, (types[0])())
            } else {
                let inner = types.iter().map(|t| t()).collect::<Vec<_>>().join(", ");
                format!("export type {}{} = [{}]", meta.name, meta.generic_params, inner)
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
                    .map(|v| render_variant(v, &e, all_metas))
                    .collect();
                format!("export type {}{} = {}", meta.name, meta.generic_params, variants.join(" | "))
            };

            if let Some(meta_obj) = render_enum_meta(meta, &e) {
                format!("{}\n\n{}", enum_def, meta_obj)
            } else {
                enum_def
            }
        }
        meta::TypeStructure::Transparent(inner) => {
            format!("export type {}{} = {}", meta.name, meta.generic_params, inner)
        }
        meta::TypeStructure::Unit => {
            format!("export type {} = null", meta.name)
        }
    };

    if let Some(transformer) = render_transformer(meta, all_metas) {
        format!("{}\n{}", def, transformer)
    } else {
        def
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
                transformations.push(format!("  (res as any).{} = toBinary((res as any).{});", f.name, f.name));
            }

            Some(format!(
                "export function to{}(res: {}): {} {{\n{}\n  return res;\n}}",
                meta.name, meta.name, meta.name, transformations.join("\n")
            ))
        }
        _ => None, // Enum transformer deferred
    }
}

pub fn render_enum_meta(type_meta: &TypeMeta, e: &meta::EnumMeta) -> Option<String> {
    if e.variants.iter().all(|v| v.attrs.is_empty()) {
        return None;
    }

    let meta_name = e.meta_name.unwrap_or_else(|| {
        Box::leak(format!("{}Meta", type_meta.name).into_boxed_str())
    });

    let mut lines = Vec::new();
    for v in e.variants {
        let mut attrs: Vec<_> = v.attrs.iter().map(|(k, val)| (snake_to_camel(k), val)).collect();
        attrs.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        let attrs_str = attrs.iter().map(|(k, val)| format!("{}: \"{}\"", k, val)).collect::<Vec<_>>().join(", ");
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

pub fn render_variant(v: &meta::VariantMeta, e: &meta::EnumMeta, all_metas: &[&TypeMeta]) -> String {
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
                    let label = if f.optional {
                        format!("{}?", f.name)
                    } else {
                        f.name.to_string()
                    };
                    format!("{}: {}", label, (f.ty_name)())
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
