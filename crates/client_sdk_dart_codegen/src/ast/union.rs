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
        for comment in self.description.iter().flat_map(Comment::lines) {
            writeln!(f, "/// {comment}")?;
        }
        writeln!(f, "class {name} {{", name = self.name.as_ref())?;
        {
            let indent = Indent(1);
            for variant in self.variants.iter() {
                for comment in variant.description.iter().flat_map(Comment::lines) {
                    writeln!(f, "{indent}/// {comment}")?;
                }
                writeln!(
                    f,
                    "{indent}final {variant_type}? {variant_name};",
                    variant_type = variant.type_name.name.as_ref(),
                    variant_name = variant.name.as_ref(),
                )?;
            }
            writeln!(f)?;
            writeln!(f, "{indent}{name}.internal({{", name = self.name.as_ref())?;
            {
                let indent = Indent(2);
                for variant in self.variants.iter() {
                    writeln!(
                        f,
                        "{indent}this.{variant_name},",
                        variant_name = variant.name.as_ref()
                    )?;
                }
            }
            writeln!(f, "{indent}}});")?;
            writeln!(f)?;
            let (first, rest) = self.variants.split_first().expect("at least one variant");
            if rest.is_empty() {
                writeln!(
                    f,
                    "{indent}dynamic toJson() => {variant_name}?.toJson();",
                    variant_name = first.name.as_ref()
                )?;
            } else {
                writeln!(
                    f,
                    "{indent}dynamic toJson() => {variant_name}?.toJson()",
                    variant_name = first.name.as_ref()
                )?;
                {
                    let indent = Indent(2);
                    let len = rest.len();
                    for (i, variant) in rest.iter().enumerate() {
                        let terminator = if i + 1 == len { ";" } else { "" };
                        writeln!(
                            f,
                            "{indent}?? {variant_name}?.toJson(){terminator}",
                            variant_name = variant.name.as_ref()
                        )?;
                    }
                }
            }
        }
        writeln!(f, "}}")
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
            r"class LoadableUIType {
    final PaymentUIType? paymentUiType;
    final IssueBillingKeyUIType? issueBillingKeyUiType;

    LoadableUIType.internal({
        this.paymentUiType,
        this.issueBillingKeyUiType,
    });

    dynamic toJson() => paymentUiType?.toJson()
        ?? issueBillingKeyUiType?.toJson();
}
"
        );
    }
}
