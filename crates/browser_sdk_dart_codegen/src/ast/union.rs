use std::fmt;

use convert_case::{Case, Casing};

use crate::ast::Indent;

use super::{Comment, Identifier};

#[derive(Debug, Clone)]
pub enum UnionParent {
    Union {
        parent_name: Identifier,
        variant_name: Identifier,
    },
    DiscriminatedUnion {
        parent_name: Identifier,
        variant_name: Identifier,
        discriminator_value: String,
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
    pub type_name: Identifier,
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
                    variant_type = variant.type_name.as_ref(),
                    variant_name = variant.name.as_ref(),
                )?;
            }
            writeln!(f)?;
            writeln!(f, "{indent}{name}._internal({{", name = self.name.as_ref())?;
            {
                let indent = Indent(2);
                for variant in self.variants.iter() {
                    writeln!(
                        f,
                        "{indent}this.{variant_name} = null,",
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
                    "{indent}dynamic _toJson() => {variant_name}?._toJson();",
                    variant_name = first.name.as_ref()
                )?;
            } else {
                writeln!(
                    f,
                    "{indent}dynamic _toJson() => {variant_name}?._toJson()",
                    variant_name = first.name.as_ref()
                )?;
                {
                    let indent = Indent(2);
                    let len = rest.len();
                    for (i, variant) in rest.iter().enumerate() {
                        let terminator = if i + 1 == len { ";" } else { "" };
                        writeln!(
                            f,
                            "{indent}?? {variant_name}?._toJson(){terminator}",
                            variant_name = variant.name.as_ref()
                        )?;
                    }
                }
            }
        }
        writeln!(f, "}}")
    }
}

#[derive(Debug, Clone)]
pub struct DiscriminatedUnion {
    pub name: Identifier,
    pub description: Option<Comment>,
    pub discriminator: Identifier,
    pub variants: Vec<DiscriminatedUnionVariant>,
    pub allow_empty: bool,
}

#[derive(Debug, Clone)]
pub struct DiscriminatedUnionVariant {
    pub discriminator_value: String,
    pub name: Identifier,
    pub type_name: Identifier,
    pub description: Option<Comment>,
}

impl fmt::Display for DiscriminatedUnion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for comment in self.description.iter().flat_map(Comment::lines) {
            writeln!(f, "/// {comment}")?;
        }
        writeln!(f, "class {name} {{", name = self.name.as_ref())?;
        {
            let variant_names = self
                .variants
                .iter()
                .map(|variant| variant.discriminator_value.to_case(Case::Camel))
                .collect::<Vec<_>>();

            let indent = Indent(1);
            if self.allow_empty {
                writeln!(
                    f,
                    "{indent}final String? {discriminator};",
                    discriminator = self.discriminator.as_ref()
                )?;
            } else {
                writeln!(
                    f,
                    "{indent}final String {discriminator};",
                    discriminator = self.discriminator.as_ref()
                )?;
            }
            for (variant, variant_name) in self.variants.iter().zip(variant_names.iter()) {
                for comment in variant.description.iter().flat_map(Comment::lines) {
                    writeln!(f, "{indent}/// {comment}")?;
                }
                writeln!(
                    f,
                    "{indent}final {variant_type}? {variant_name};",
                    variant_type = variant.type_name.as_ref(),
                )?;
            }
            writeln!(f)?;
            writeln!(f, "{indent}{name}._internal(", name = self.name.as_ref())?;
            {
                let indent = Indent(2);
                writeln!(
                    f,
                    "{indent}this.{discriminator},",
                    discriminator = self.discriminator.as_ref()
                )?;
                writeln!(f, "{indent}{{")?;
                {
                    let indent = Indent(3);
                    for variant_name in variant_names.iter() {
                        writeln!(f, "{indent}this.{variant_name} = null,",)?;
                    }
                }
                writeln!(f, "{indent}}}")?;
            }
            writeln!(f, "{indent});")?;
            if self.allow_empty {
                writeln!(f)?;
                writeln!(
                    f,
                    "{indent}{name}.empty() : this._internal(null);",
                    name = self.name.as_ref()
                )?;
            }
            writeln!(f)?;
            writeln!(f, "{indent}Map<String, dynamic> _toJson() => {{")?;
            {
                let indent = Indent(2);
                writeln!(
                    f,
                    "{indent}'{discriminator}': {discriminator},",
                    discriminator = self.discriminator.as_ref()
                )?;
                for variant_name in variant_names.iter() {
                    writeln!(f, "{indent}...?{variant_name}?._toJson(),")?;
                }
            }
            writeln!(f, "{indent}}};")?;
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
                    type_name: Identifier::try_from("PaymentUIType").unwrap(),
                    description: None,
                },
                UnionVariant {
                    name: Identifier::try_from("issueBillingKeyUiType").unwrap(),
                    type_name: Identifier::try_from("IssueBillingKeyUIType").unwrap(),
                    description: None,
                },
            ],
        };
        assert_eq!(
            union.to_string(),
            r"class LoadableUIType {
    final PaymentUIType? paymentUiType;
    final IssueBillingKeyUIType? issueBillingKeyUiType;

    LoadableUIType._internal({
        this.paymentUiType = null,
        this.issueBillingKeyUiType = null,
    });

    dynamic _toJson() => paymentUiType?._toJson()
        ?? issueBillingKeyUiType?._toJson();
}
"
        );
    }

    #[test]
    fn discriminated_union_with_variant() {
        let union = DiscriminatedUnion {
            name: Identifier::try_from("PaymentRequestUnion").unwrap(),
            description: None,
            discriminator: Identifier::try_from("payMethod").unwrap(),
            variants: vec![
                DiscriminatedUnionVariant {
                    name: Identifier::try_from("card").unwrap(),
                    discriminator_value: "CARD".into(),
                    type_name: Identifier::try_from("PaymentRequestUnionCard").unwrap(),
                    description: None,
                },
                DiscriminatedUnionVariant {
                    name: Identifier::try_from("easyPay").unwrap(),
                    discriminator_value: "EASY_PAY".into(),
                    type_name: Identifier::try_from("PaymentRequestUnionEasyPay").unwrap(),
                    description: None,
                },
            ],
            allow_empty: true,
        };
        assert_eq!(
            union.to_string(),
            r"class PaymentRequestUnion {
    final String? payMethod;
    final PaymentRequestUnionCard? card;
    final PaymentRequestUnionEasyPay? easyPay;

    PaymentRequestUnion._internal(
        this.payMethod,
        {
            this.card = null,
            this.easyPay = null,
        }
    );

    PaymentRequestUnion.empty() : this._internal(null);

    Map<String, dynamic> _toJson() => {
        'payMethod': payMethod,
        ...?card?._toJson(),
        ...?easyPay?._toJson(),
    };
}
"
        );
    }
}
