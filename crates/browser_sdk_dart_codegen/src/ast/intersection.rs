use core::fmt;

use super::{Comment, Identifier, Indent, TypeReference, UnionParent};

pub struct Intersection {
    pub name: Identifier,
    pub description: Option<Comment>,
    pub constituents: Vec<IntersectionConstituent>,
    pub union_parents: Vec<UnionParent>,
}

pub struct IntersectionConstituent {
    pub name: Identifier,
    pub type_name: TypeReference,
}

impl fmt::Display for Intersection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "class {name} {{", name = self.name.as_ref())?;
        {
            let indent = Indent(1);
            for constituent in self.constituents.iter() {
                writeln!(
                    f,
                    "{indent}final {type_name} {name};",
                    type_name = constituent.type_name.name.as_ref(),
                    name = constituent.name.as_ref()
                )?;
            }
            writeln!(f)?;
            writeln!(f, "{indent}{name}(", name = self.name.as_ref())?;
            {
                let indent = Indent(2);
                for constituent in self.constituents.iter() {
                    writeln!(f, "{indent}this.{name},", name = constituent.name.as_ref())?;
                }
            }
            writeln!(f, "{indent});")?;
            writeln!(f)?;
            writeln!(f, "{indent}Map<String, dynamic> toJson() => {{")?;
            {
                let indent = Indent(2);
                for constituent in self.constituents.iter() {
                    writeln!(
                        f,
                        "{indent}...{name}.toJson(),",
                        name = constituent.name.as_ref()
                    )?;
                }
            }
            writeln!(f, "{indent}}};")?;
            if !self.union_parents.is_empty() {
                writeln!(f)?;
                for parent in self.union_parents.iter() {
                    match parent {
                        UnionParent::Union {
                            parent,
                            variant_name,
                        } => {
                            writeln!(
                                f,
                                "{indent}{parent_name} to{parent_name}() => {parent_name}.internal({variant_name}: this);",
                                parent_name = parent.name.as_ref(),
                                variant_name = variant_name.as_ref(),
                            )?;
                        }
                        UnionParent::DiscriminatedUnion {
                            parent,
                            variant_name,
                            discriminator_value,
                        } => {
                            writeln!(
                                f,
                                "{indent}{parent_name} to{parent_name}() => {parent_name}.internal('{discriminator_value}', {variant_name}: this);",
                                parent_name = parent.name.as_ref(),
                                variant_name = variant_name.as_ref(),
                                discriminator_value = discriminator_value,
                            )?;
                        }
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
    fn test_intersection() {
        let intersection = Intersection {
            name: Identifier::try_from("PaymentRequest").unwrap(),
            description: None,
            constituents: vec![
                IntersectionConstituent {
                    name: Identifier::try_from("paymentRequestBase").unwrap(),
                    type_name: TypeReference {
                        name: Identifier::try_from("PaymentRequestBase").unwrap(),
                        path: "".into(),
                    },
                },
                IntersectionConstituent {
                    name: Identifier::try_from("paymentRequestUnion").unwrap(),
                    type_name: TypeReference {
                        name: Identifier::try_from("PaymentRequestUnion").unwrap(),
                        path: "".into(),
                    },
                },
            ],
            union_parents: vec![],
        };
        assert_eq!(
            intersection.to_string(),
            r"class PaymentRequest {
    final PaymentRequestBase paymentRequestBase;
    final PaymentRequestUnion paymentRequestUnion;

    PaymentRequest(
        this.paymentRequestBase,
        this.paymentRequestUnion,
    );

    Map<String, dynamic> toJson() => {
        ...paymentRequestBase.toJson(),
        ...paymentRequestUnion.toJson(),
    };
}
"
        );
    }
}
