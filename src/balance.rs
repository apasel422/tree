#[forbid(missing_docs)]

/// A binary search tree node.
pub trait Node {
    /// The node's balance metadata.
    type Balance: Balance;

    /// Returns a reference to the node's balance metadata.
    fn balance(&self) -> &Self::Balance;

    /// Returns a mutable reference to the node's balance metadata.
    fn balance_mut(&mut self) -> &mut Self::Balance;

    /// Returns a reference to the node's left child, if any.
    fn left(&self) -> Option<&Self>;

    /// Returns a mutable reference to the node's left child, if any.
    fn left_mut(&mut self) -> Option<&mut Self>;

    /// Returns a reference to the node's right child, if any.
    fn right(&self) -> Option<&Self>;

    /// Returns a mutable reference to the node's right child, if any.
    fn right_mut(&mut self) -> Option<&mut Self>;

    /// Rotates the node to the left, if possible.
    fn rotate_left(&mut self);

    /// Rotates the node to the right, if possible.
    fn rotate_right(&mut self);
}

/// Balance metadata for a single binary search tree node.
pub trait Balance: Default {
    /// Rebalances the given node after an insertion in one of its subtrees.
    fn rebalance_insert<N>(node: &mut N) where N: Node<Balance = Self>;

    /// Rebalances the given node after a removal in one of its subtrees.
    fn rebalance_remove<N>(node: &mut N) where N: Node<Balance = Self>;
}

/// Metadata for the AA balance scheme.
#[derive(Clone, Copy, Debug)]
pub struct Aa(usize);

impl Aa {
    #[cfg(test)]
    pub fn level(&self) -> usize { self.0 }

    // Remove left horizontal link by rotating right
    //
    // From https://github.com/Gankro/collect-rs/tree/map.rs
    fn skew<N>(node: &mut N) where N: Node<Balance = Self> {
        if node.left().map_or(false, |x| x.balance().0 == node.balance().0) {
            node.rotate_right();
        }
    }

    // Remove dual horizontal link by rotating left and increasing level of
    // the parent
    //
    // From https://github.com/Gankro/collect-rs/tree/map.rs
    fn split<N>(node: &mut N) where N: Node<Balance = Self> {
        if node.right().map_or(false,
          |x| x.right().map_or(false, |y| y.balance().0 == node.balance().0)) {
            node.rotate_left();
            node.balance_mut().0 += 1;
        }
    }
}

impl Default for Aa {
    fn default() -> Self { Aa(1) }
}

impl Balance for Aa {
    fn rebalance_insert<N>(node: &mut N) where N: Node<Balance = Self> {
        Self::skew(node);
        Self::split(node);
    }

    fn rebalance_remove<N>(node: &mut N) where N: Node<Balance = Self> {
        let left_level = node.left().map_or(0, |node| node.balance().0);
        let right_level = node.right().map_or(0, |node| node.balance().0);

        // re-balance, if necessary
        if left_level < node.balance().0 - 1 || right_level < node.balance().0 - 1 {
            node.balance_mut().0 -= 1;

            if right_level > node.balance().0 {
                let node_level = node.balance().0;
                if let Some(x) = node.right_mut() { x.balance_mut().0 = node_level; }
            }

            Self::skew(node);

            if let Some(right) = node.right_mut() {
                Self::skew(right);
                if let Some(x) = right.right_mut() { Self::skew(x); };
            }

            Self::split(node);
            if let Some(x) = node.right_mut() { Self::split(x); }
        }
    }
}
