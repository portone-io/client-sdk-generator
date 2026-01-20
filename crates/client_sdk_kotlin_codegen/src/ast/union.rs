use std::fmt;

use crate::ast::Indent;

use super::{Comment, Identifier, TypeReference};

#[derive(Debug, Clone)]
pub enum UnionParent {
    Union {
        parent: TypeReference,
        variant_name: Identifier,
    },
}

#[derive(Debug, Clone)]
pub struct Union {
    pub name: Identifier,
    pub description: Option<Comment>,
    pub variants: Vec<UnionVariant>,
}

#[derive(Debug, Clone)]
pub struct UnionVariant {
    pub name: Identifier,
    pub type_name: TypeReference,
    pub description: Option<Comment>,
}

impl fmt::Display for Union {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Comments
        if let Some(ref desc) = self.description {
            writeln!(f, "/**")?;
            for line in desc.lines() {
                writeln!(f, " * {line}")?;
            }
            writeln!(f, " */")?;
        }

        for variant in self.variants.iter() {
            writeln!(
                f,
                "private typealias _{type_name} = {type_name}",
                type_name = variant.type_name.name.as_ref()
            )?;
        }
        writeln!(f)?;

        // Sealed class declaration
        writeln!(f, "@Parcelize")?;
        writeln!(
            f,
            "sealed class {name} : Parcelable {{",
            name = self.name.as_ref()
        )?;
        {
            let indent = Indent(1);

            // Variant declarations
            for variant in self.variants.iter() {
                if let Some(ref desc) = variant.description {
                    writeln!(f, "{indent}/**")?;
                    for line in desc.lines() {
                        writeln!(f, "{indent} * {line}")?;
                    }
                    writeln!(f, "{indent} */")?;
                }
                writeln!(f, "{indent}@Parcelize")?;
                writeln!(
                    f,
                    "{indent}data class {variant_name}(val value: _{type_name}) : {name}()",
                    variant_name = capitalize_first(variant.name.as_ref()),
                    type_name = variant.type_name.name.as_ref(),
                    name = self.name.as_ref(),
                )?;
            }

            writeln!(f)?;

            // toJson method
            writeln!(f, "{indent}fun toJson(): Any = when (this) {{")?;
            {
                let indent = Indent(2);
                for variant in self.variants.iter() {
                    writeln!(
                        f,
                        "{indent}is {variant_name} -> value.toJson()",
                        variant_name = capitalize_first(variant.name.as_ref())
                    )?;
                }
            }
            writeln!(f, "{indent}}}")?;
        }
        writeln!(f, "}}")
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_with_variant() {
        let union = Union {
            name: Identifier::try_from("LoadableUIType").unwrap(),
            description: None,
            variants: vec![
                UnionVariant {
                    name: Identifier::try_from("paymentUiType").unwrap(),
                    type_name: TypeReference {
                        name: Identifier::try_from("PaymentUIType").unwrap(),
                        path: "".into(),
                    },
                    description: None,
                },
                UnionVariant {
                    name: Identifier::try_from("issueBillingKeyUiType").unwrap(),
                    type_name: TypeReference {
                        name: Identifier::try_from("IssueBillingKeyUIType").unwrap(),
                        path: "".into(),
                    },
                    description: None,
                },
            ],
        };
        assert_eq!(
            union.to_string(),
            r#"private typealias _PaymentUIType = PaymentUIType
private typealias _IssueBillingKeyUIType = IssueBillingKeyUIType

@Parcelize
sealed class LoadableUIType : Parcelable {
    @Parcelize
    data class PaymentUiType(val value: _PaymentUIType) : LoadableUIType()
    @Parcelize
    data class IssueBillingKeyUiType(val value: _IssueBillingKeyUIType) : LoadableUIType()

    fun toJson(): Any = when (this) {
        is PaymentUiType -> value.toJson()
        is IssueBillingKeyUiType -> value.toJson()
    }
}
"#
        );
    }
}
