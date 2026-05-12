pub enum TypeStructure {
    Struct(StructMeta),
    Tuple(&'static [fn() -> String]),
    Enum(EnumMeta),
    Transparent(String),
    Unit,
}

pub struct StructMeta {
    pub fields: &'static [FieldMeta],
}

pub struct EnumMeta {
    pub variants: &'static [VariantMeta],
    pub tag: Option<&'static str>,
    pub content: Option<&'static str>,
    pub untagged: bool,
}

pub struct FieldMeta {
    pub name: &'static str,
    pub ty_name: fn() -> String,
    pub optional: bool,
    pub flattened: bool,
    pub flatten_base_name: Option<&'static str>,
}

pub struct VariantMeta {
    pub name: &'static str,
    pub fields: VariantFields,
}

pub enum VariantFields {
    Unit,
    Unnamed(&'static [fn() -> String]),
    Named(&'static [FieldMeta]),
}
