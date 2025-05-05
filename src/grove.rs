use crate::internal::get_tree;
use crate::internal::get_tree_mut;
use crate::node::Node;
use crate::traversal::TraversalOrder;
use crate::tree::Tree;

/// An unsized type referencing a collection of consecutive [`Tree`]s inside a
/// [`GroveBuf`][crate::GroveBuf].
#[repr(transparent)]
pub struct Grove<T> {
  pub(crate) nodes: [Node<T>],
}

impl<T> Grove<T> {
  /// Returns `true` if and only if the [`Grove`] contains no trees.
  pub fn is_empty(&self) -> bool {
    self.nodes.is_empty()
  }

  /// Returns the number of nodes in the [`Grove`].
  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  /// Returns an iterator traversing through references to nodes in the
  /// [`Grove`] according to the prescribed `TraversalOrder`.
  ///
  /// # Example
  /// ```
  /// # use grove::*;
  /// let g: GroveBuf<i32> = grove_buf![[1, 4, 9] => 16, 25];
  /// let g_ref = g.as_ref();
  /// let v: Vec<_> = g_ref.nodes(Preorder).collect();
  /// assert_eq!(v, vec![&1, &4, &9, &16, &25]);
  /// let v: Vec<_> = g_ref.nodes(ReversePostorder).collect();
  /// assert_eq!(v, vec![&25, &16, &9, &4, &1]);
  /// ```
  pub fn nodes<Order: TraversalOrder>(
    &self,
    order: Order,
  ) -> impl std::iter::Iterator<Item = &T> {
    order.node_iter(&self.nodes)
  }

  /// Returns an iterator traversing through mutable references to nodes in the
  /// [`Grove`] according to the prescribed `TraversalOrder`. Analogous to
  /// [`Grove::nodes`], with mutable references.
  pub fn nodes_mut<Order: TraversalOrder>(
    &mut self,
    order: Order,
  ) -> impl std::iter::Iterator<Item = &'_ mut T> {
    order.node_iter_mut(&mut self.nodes)
  }

  /// Returns an iterator traversing through references to trees in the `Grove`
  /// according to the prescribed `TraversalOrder`.
  pub fn trees<Order: TraversalOrder>(
    &self,
    order: Order,
  ) -> impl std::iter::Iterator<Item = &Tree<T>> {
    order.tree_iter(&self.nodes)
  }

  /// Returns an iterator traversing through mutable references to trees in the
  /// [`Grove`] according to the prescribed `TraversalOrder`.
  pub fn trees_mut<Order: TraversalOrder>(
    &mut self,
    order: Order,
  ) -> impl std::iter::Iterator<Item = &mut Tree<T>> {
    order.tree_iter_mut(&mut self.nodes)
  }
}

impl<T> std::ops::Index<usize> for Grove<T> {
  type Output = Tree<T>;

  /// Returns a reference to the [`Tree`] whose root has the given index.
  fn index(&self, index: usize) -> &Self::Output {
    get_tree(&self.nodes[index])
  }
}

impl<T> std::ops::IndexMut<usize> for Grove<T> {
  /// Returns a reference to the [`Tree`] whose root has the given index.
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    get_tree_mut(&mut self.nodes[index])
  }
}

#[cfg(test)]
mod tests {
  use crate::grove_buf;
  use crate::grove_buf::GroveBuf;
  use crate::traversal::Preorder;
  use crate::traversal::ReversePostorder;

  #[test]
  fn empty() {
    let g = GroveBuf::<i32>::new();
    assert!(g.as_ref().is_empty());
    assert_eq!(g.as_ref().len(), 0);
  }

  #[test]
  fn len() {
    let g = grove_buf![[1, 2] => 3, 4, [5, 6] => 7];
    assert!(!g.as_ref().is_empty());
    assert_eq!(g.as_ref().len(), 7);
  }

  #[test]
  fn preorder_nodes() {
    let g = grove_buf![[1, 2] => 3, 4, [5, 6] => 7];
    let v: Vec<_> = g.as_ref().nodes(Preorder).cloned().collect();
    assert_eq!(v, vec![1, 2, 3, 4, 5, 6, 7]);
  }

  #[test]
  fn reverse_postorder_nodes() {
    let g = grove_buf![[1, 2] => 3, 4, [5, 6] => 7];
    let v: Vec<_> = g.as_ref().nodes(ReversePostorder).cloned().collect();
    assert_eq!(v, vec![7, 6, 5, 4, 3, 2, 1]);
  }
}
