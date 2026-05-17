/// Describes the shape of a Rust type for TypeScript code generation.
///
/// Produced at runtime by the closure stored in [`super::TypeMeta::structure`].
/// The generator pattern-matches on this to emit the correct TypeScript syntax.
pub enum TypeStructure {
    /// A named-field struct: `export type Foo = { field: T }`
    Struct(StructMeta),
    /// A tuple struct or single-type alias: `export type Foo = [A, B]`
    Tuple(&'static [fn() -> String]),
    /// An enum with variants: union types or `as const` objects.
    Enum(EnumMeta),
    /// A `#[serde(transparent)]` newtype: `export type Foo = Inner`
    Transparent(String),
    /// A unit struct: `export type Foo = null`
    Unit,
}

/// Metadata for a named-field struct.
pub struct StructMeta {
    /// The struct's fields in declaration order.
    pub fields: &'static [FieldMeta],
}

/// Metadata for an enum, capturing its Serde tagging configuration.
pub struct EnumMeta {
    /// The enum's variants in declaration order.
    pub variants: &'static [VariantMeta],
    /// The `#[serde(tag = "...")]` field name, if any.
    pub tag: Option<&'static str>,
    /// The `#[serde(content = "...")]` field name, if any.
    pub content: Option<&'static str>,
    /// Whether `#[serde(untagged)]` is present.
    pub untagged: bool,
    /// The name for the generated metadata object, if any.
    ///
    /// Set by `#[tyzen(meta = "...")]`.
    pub meta_name: Option<&'static str>,
}

/// Metadata for a single struct or variant field.
pub struct FieldMeta {
    /// The field name as it will appear in TypeScript (after renaming).
    pub name: &'static str,
    /// Returns the TypeScript type name for this field.
    pub ty_name: fn() -> String,
    /// Whether this field should be emitted with a `?` optional marker.
    ///
    /// True when `#[tyzen(optional)]` or `#[serde(default)]` is present.
    pub optional: bool,
    /// Whether this field is `#[serde(flatten)]`-ed into the parent.
    pub flattened: bool,
    /// The type name of the flattened struct, used to resolve its fields.
    pub flatten_base_name: Option<&'static str>,
    /// Whether this field contains binary data that should be transformed to `Uint8Array`.
    pub is_binary: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AttrValue {
    Str(&'static str),
    List(&'static [&'static str]),
}

/// Metadata for a single enum variant.
pub struct VariantMeta {
    /// The variant name as it will appear in TypeScript (after renaming).
    pub name: &'static str,
    /// The shape of this variant's fields.
    pub fields: VariantFields,
    /// Arbitrary metadata attributes attached to this variant.
    ///
    /// Captured from `#[tyzen(key = "value")]`.
    pub attrs: &'static [(&'static str, AttrValue)],
}

/// Describes the field shape of an enum variant.
pub enum VariantFields {
    /// No fields: `Foo::Bar`
    Unit,
    /// Positional fields: `Foo::Bar(A, B)`
    Unnamed(&'static [fn() -> String]),
    /// Named fields: `Foo::Bar { a: A, b: B }`
    Named(&'static [FieldMeta]),
}
