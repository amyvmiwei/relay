/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::util::PointerAddress;
use graphql_ir::{Program, Selection, Transformed, Transformer};
use std::collections::HashMap;

type Seen = HashMap<PointerAddress, Transformed<Selection>>;

///
/// Sorts selections in the fragments and queries (and their selections)
///
pub fn sort_selections<'s>(program: &'s Program<'s>) -> Program<'s> {
    let mut transform = SortSelectionsTransform::new();
    transform
        .transform_program(program)
        .unwrap_or_else(|| program.clone())
}

#[derive(Default)]
struct SortSelectionsTransform {
    seen: Seen,
}

impl SortSelectionsTransform {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Transformer for SortSelectionsTransform {
    const NAME: &'static str = "SortSelectionsTransform";
    const VISIT_ARGUMENTS: bool = false;
    const VISIT_DIRECTIVES: bool = false;

    fn transform_selections(&mut self, selections: &[Selection]) -> Option<Vec<Selection>> {
        let mut next_selections = self
            .transform_list(selections, Self::transform_selection)
            .unwrap_or_else(|| selections.to_vec());
        next_selections.sort_unstable();
        Some(next_selections)
    }

    fn transform_selection(&mut self, selection: &Selection) -> Transformed<Selection> {
        match selection {
            Selection::InlineFragment(selection) => {
                let key = PointerAddress::new(selection);
                if let Some(prev) = self.seen.get(&key) {
                    return prev.clone();
                }
                let transformed = self
                    .transform_inline_fragment(selection)
                    .map(Selection::InlineFragment);
                self.seen.insert(key, transformed.clone());
                transformed
            }
            Selection::LinkedField(selection) => {
                let key = PointerAddress::new(selection);
                if let Some(prev) = self.seen.get(&key) {
                    return prev.clone();
                }
                let transformed = self
                    .transform_linked_field(selection)
                    .map(Selection::LinkedField);
                self.seen.insert(key, transformed.clone());
                transformed
            }
            _ => Transformed::Keep,
        }
    }
}
