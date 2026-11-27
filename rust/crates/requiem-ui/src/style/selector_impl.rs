use selectors::attr;
use selectors::bloom;
use selectors::context;
use selectors::matching;
use selectors::parser;
use std::borrow::Borrow;
use std::fmt;

use cssparser::{CowRcStr, ParseError, SourceLocation, ToCss};
use precomputed_hash::PrecomputedHash;
use selectors::attr::CaseSensitivity;
use selectors::parser::SelectorParseErrorKind;
use selectors::{OpaqueElement, SelectorImpl};

use crate::widget::{WidgetId, WidgetTree};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct CssString(pub String);

impl<'a> From<&'a str> for CssString {
    fn from(value: &'a str) -> Self {
        CssString(value.to_owned())
    }
}

impl ToCss for CssString {
    fn to_css<W: fmt::Write>(&self, dest: &mut W) -> fmt::Result {
        dest.write_str(&self.0)
    }
}

impl PrecomputedHash for CssString {
    fn precomputed_hash(&self) -> u32 {
        let mut hash: u32 = 2166136261;
        for byte in self.0.as_bytes() {
            hash ^= *byte as u32;
            hash = hash.wrapping_mul(16777619);
        }
        hash
    }
}

impl Borrow<str> for CssString {
    fn borrow(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PseudoClass {
    Hover,
    Active,

    Focus,

    FocusWithin,
}

impl ToCss for PseudoClass {
    fn to_css<W: fmt::Write>(&self, dest: &mut W) -> fmt::Result {
        dest.write_str(match self {
            PseudoClass::Hover => ":hover",
            PseudoClass::Active => ":active",
            PseudoClass::Focus => ":focus",
            PseudoClass::FocusWithin => ":focus-within",
        })
    }
}

impl parser::NonTSPseudoClass for PseudoClass {
    type Impl = WidgetSelectorImpl;

    fn is_active_or_hover(&self) -> bool {
        matches!(self, PseudoClass::Hover | PseudoClass::Active)
    }

    fn is_user_action_state(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoPseudoElement {}

impl ToCss for NoPseudoElement {
    fn to_css<W: fmt::Write>(&self, _dest: &mut W) -> fmt::Result {
        match *self {}
    }
}

impl parser::PseudoElement for NoPseudoElement {
    type Impl = WidgetSelectorImpl;
}

#[derive(Clone, Debug)]
pub struct WidgetSelectorImpl;

impl SelectorImpl for WidgetSelectorImpl {
    type ExtraMatchingData<'a> = ();
    type AttrValue = CssString;
    type Identifier = CssString;
    type LocalName = CssString;
    type NamespaceUrl = CssString;
    type NamespacePrefix = CssString;
    type BorrowedNamespaceUrl = str;
    type BorrowedLocalName = str;
    type NonTSPseudoClass = PseudoClass;
    type PseudoElement = NoPseudoElement;
}

pub struct WidgetSelectorParser;

impl<'i> parser::Parser<'i> for WidgetSelectorParser {
    type Impl = WidgetSelectorImpl;
    type Error = SelectorParseErrorKind<'i>;

    fn parse_non_ts_pseudo_class(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<PseudoClass, ParseError<'i, Self::Error>> {
        match name.as_ref() {
            "hover" => Ok(PseudoClass::Hover),
            "active" => Ok(PseudoClass::Active),
            "focus" => Ok(PseudoClass::Focus),
            "focus-within" => Ok(PseudoClass::FocusWithin),
            _ => Err(location.new_custom_error(
                SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name),
            )),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ElementRef<'a> {
    pub tree: &'a WidgetTree,
    pub id: WidgetId,
}

impl<'a> fmt::Debug for ElementRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElementRef").field("id", &self.id).finish()
    }
}

impl<'a> selectors::Element for ElementRef<'a> {
    type Impl = WidgetSelectorImpl;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(&self.tree.nodes[self.id])
    }

    fn parent_element(&self) -> Option<Self> {
        self.tree.nodes[self.id].parent.map(|id| ElementRef {
            tree: self.tree,
            id,
        })
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        let node = &self.tree.nodes[self.id];
        let parent = node.parent?;
        let siblings = &self.tree.nodes[parent].children;
        let idx = siblings.iter().position(|&c| c == self.id)?;
        idx.checked_sub(1).map(|i| ElementRef {
            tree: self.tree,
            id: siblings[i],
        })
    }

    fn next_sibling_element(&self) -> Option<Self> {
        let node = &self.tree.nodes[self.id];
        let parent = node.parent?;
        let siblings = &self.tree.nodes[parent].children;
        let idx = siblings.iter().position(|&c| c == self.id)?;
        siblings.get(idx + 1).map(|&id| ElementRef {
            tree: self.tree,
            id,
        })
    }

    fn first_element_child(&self) -> Option<Self> {
        self.tree.nodes[self.id]
            .children
            .first()
            .map(|&id| ElementRef {
                tree: self.tree,
                id,
            })
    }

    fn is_html_element_in_html_document(&self) -> bool {
        false
    }

    fn has_local_name(&self, local_name: &str) -> bool {
        self.tree.nodes[self.id].tag == local_name
    }

    fn has_namespace(&self, ns: &str) -> bool {
        ns.is_empty()
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.tree.nodes[self.id].tag == self.tree.nodes[other.id].tag
    }

    fn attr_matches(
        &self,
        _ns: &attr::NamespaceConstraint<&CssString>,
        _local_name: &CssString,
        _operation: &attr::AttrSelectorOperation<&CssString>,
    ) -> bool {
        false
    }

    fn match_non_ts_pseudo_class(
        &self,
        pc: &PseudoClass,
        _context: &mut context::MatchingContext<Self::Impl>,
    ) -> bool {
        match pc {
            PseudoClass::Hover => self.tree.hovered_path.contains(&self.id),
            PseudoClass::Active => self.tree.active_path.contains(&self.id),
            PseudoClass::Focus => self.tree.focused == Some(self.id),
            PseudoClass::FocusWithin => {
                let mut cursor = self.tree.focused;
                while let Some(id) = cursor {
                    if id == self.id {
                        return true;
                    }
                    cursor = self.tree.nodes.get(id).and_then(|n| n.parent);
                }
                false
            }
        }
    }

    fn match_pseudo_element(
        &self,
        pe: &NoPseudoElement,
        _context: &mut context::MatchingContext<Self::Impl>,
    ) -> bool {
        match *pe {}
    }

    fn apply_selector_flags(&self, _flags: matching::ElementSelectorFlags) {}

    fn is_link(&self) -> bool {
        false
    }

    fn is_html_slot_element(&self) -> bool {
        false
    }

    fn has_id(&self, id: &CssString, case_sensitivity: CaseSensitivity) -> bool {
        match &self.tree.nodes[self.id].dom_id {
            Some(dom_id) => case_sensitivity.eq(dom_id.as_bytes(), id.0.as_bytes()),
            None => false,
        }
    }

    fn has_class(&self, name: &CssString, case_sensitivity: CaseSensitivity) -> bool {
        self.tree.nodes[self.id]
            .classes
            .iter()
            .any(|c| case_sensitivity.eq(c.as_bytes(), name.0.as_bytes()))
    }

    fn has_custom_state(&self, _name: &CssString) -> bool {
        false
    }

    fn imported_part(&self, _name: &CssString) -> Option<CssString> {
        None
    }

    fn is_part(&self, _name: &CssString) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        self.tree.nodes[self.id].children.is_empty()
    }

    fn is_root(&self) -> bool {
        self.tree.nodes[self.id].parent.is_none()
    }

    fn add_element_unique_hashes(&self, _filter: &mut bloom::BloomFilter) -> bool {
        false
    }
}
