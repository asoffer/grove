use crate::internal::get_tree;
use crate::internal::get_tree_mut;
use crate::node::Node;
use crate::tree::Tree;

pub(crate) trait TraversalOrder: Sized {
  fn iter<'a, T: 'a>(
    self,
    nodes: &'a [Node<T>],
  ) -> impl Iterator<Item = &'a Node<T>>;

  fn iter_mut<'a, T: 'a>(
    self,
    nodes: &'a mut [Node<T>],
  ) -> impl Iterator<Item = &'a mut Node<T>>;

  fn node_iter<'a, T: 'a>(
    self,
    nodes: &'a [Node<T>],
  ) -> impl std::iter::Iterator<Item = &'a T> {
    self.iter(nodes).map(|node| &node.value)
  }
  fn tree_iter<'a, T: 'a>(
    self,
    nodes: &'a [Node<T>],
  ) -> impl std::iter::Iterator<Item = &'a Tree<T>> {
    self.iter(nodes).map(get_tree)
  }

  fn node_iter_mut<'a, T: 'a>(
    self,
    nodes: &'a mut [Node<T>],
  ) -> impl std::iter::Iterator<Item = &'a mut T> {
    self.iter_mut(nodes).map(|node| &mut node.value)
  }

  fn tree_iter_mut<'a, T: 'a>(
    self,
    nodes: &'a mut [Node<T>],
  ) -> impl std::iter::Iterator<Item = &'a mut Tree<T>> {
    self.iter_mut(nodes).map(get_tree_mut)
  }
}

/// A `TraversalOrder` iterating through nodes in pre-order. That is,
/// * Each node's children are visited in left-to-right order.
/// * Each node's children are visited before the node itself.
pub struct Preorder;
impl TraversalOrder for Preorder {
  fn iter<'a, T: 'a>(
    self,
    nodes: &'a [Node<T>],
  ) -> impl Iterator<Item = &'a Node<T>> {
    nodes.iter()
  }

  fn iter_mut<'a, T: 'a>(
    self,
    nodes: &'a mut [Node<T>],
  ) -> impl Iterator<Item = &'a mut Node<T>> {
    nodes.iter_mut()
  }
}

/// A `TraversalOrder` iterating through nodes in reverse post-order. That is,
/// * Each node's children are visited in right-to-left order.
/// * Each node is visited before its children.
pub struct ReversePostorder;
impl TraversalOrder for ReversePostorder {
  fn iter<'a, T: 'a>(
    self,
    nodes: &'a [Node<T>],
  ) -> impl Iterator<Item = &'a Node<T>> {
    nodes.iter().rev()
  }

  fn iter_mut<'a, T: 'a>(
    self,
    nodes: &'a mut [Node<T>],
  ) -> impl Iterator<Item = &'a mut Node<T>> {
    nodes.iter_mut().rev()
  }
}
