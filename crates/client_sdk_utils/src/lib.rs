use markdown::{ParseOptions, mdast::Node, to_mdast};
use mdast_util_to_markdown::{ to_markdown_with_options, Options};

pub trait ToMdastExt {
    fn to_mdast(&self) -> Result<Node, markdown::message::Message>;
}

impl ToMdastExt for str {
    fn to_mdast(&self) -> Result<Node, markdown::message::Message> {
        to_mdast(self, &ParseOptions::mdx())
    }
}

impl ToMdastExt for String {
    fn to_mdast(&self) -> Result<Node, markdown::message::Message> {
        to_mdast(self, &ParseOptions::mdx())
    }
}

pub trait MdastNodeExt {
    fn to_markdown_string(&self) -> Result<String, markdown::message::Message>;
    fn remove_jsx_elements(&mut self) -> &mut Self;
}

impl MdastNodeExt for Node {
    fn to_markdown_string(&self) -> Result<String, markdown::message::Message> {
        to_markdown_with_options(self, &Options {
          bullet: '-',
          ..Default::default()
        })
    }

    fn remove_jsx_elements(&mut self) -> &mut Self {
        if let Some(children) = self.children_mut() {
            children.retain_mut(|child| match child {
                Node::MdxJsxFlowElement(_) | Node::MdxJsxTextElement(_) => false,
                _ => {
                    child.remove_jsx_elements();
                    true
                }
            });
        }

        self
    }
}
