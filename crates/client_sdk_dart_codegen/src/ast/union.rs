use std::fmt;

use crate::ast::Indent;

use super::{Comment, Identifier, TypeReference, capitalize_first};

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
        for comment in self.description.iter().flat_map(Comment::lines) {
            writeln!(f, "/// {comment}")?;
        }
        writeln!(f, "sealed class {name} {{", name = self.name.as_ref())?;
        {
            let indent = Indent(1);
            writeln!(f, "{indent}dynamic toJson();")?;
        }
        writeln!(f, "}}")?;

        // Subclasses
        for variant in self.variants.iter() {
            writeln!(f)?;
            let subclass_name = format!(
                "{}{}",
                self.name.as_ref(),
                capitalize_first(variant.name.as_ref())
            );
            for comment in variant.description.iter().flat_map(Comment::lines) {
                writeln!(f, "/// {comment}")?;
            }
            writeln!(
                f,
                "class {subclass_name} extends {name} {{",
                name = self.name.as_ref()
            )?;
            {
                let indent = Indent(1);
                writeln!(
                    f,
                    "{indent}final {variant_type} value;",
                    variant_type = variant.type_name.name.as_ref(),
                )?;
                writeln!(f, "{indent}{subclass_name}(this.value);")?;
                writeln!(f, "{indent}@override")?;
                writeln!(f, "{indent}dynamic toJson() => value.toJson();")?;
            }
            writeln!(f, "}}")?;
        }
        Ok(())
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
            r"sealed class LoadableUIType {
    dynamic toJson();
}

class LoadableUITypePaymentUiType extends LoadableUIType {
    final PaymentUIType value;
    LoadableUITypePaymentUiType(this.value);
    @override
    dynamic toJson() => value.toJson();
}

class LoadableUITypeIssueBillingKeyUiType extends LoadableUIType {
    final IssueBillingKeyUIType value;
    LoadableUITypeIssueBillingKeyUiType(this.value);
    @override
    dynamic toJson() => value.toJson();
}
"
        );
    }
}
