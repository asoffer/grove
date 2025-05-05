use crate::internal::as_tree_mut_unchecked;
use crate::internal::as_tree_unchecked;
use crate::internal::get_tree;
use crate::internal::get_tree_mut;
use crate::node::Node;

/// An unsized type referencing a a single tree inside a
/// [`GroveBuf`][crate::GroveBuf].
#[repr(transparent)]
#[derive(Debug, Eq)]
pub struct Tree<T> {
  pub(crate) nodes: [Node<T>],
}

impl<T, U: PartialEq<T>> PartialEq<Tree<T>> for Tree<U> {
  fn eq(&self, t: &Tree<T>) -> bool {
    self.nodes == t.nodes
  }
}

struct ChildIter<'a, T>(&'a [Node<T>]);

impl<'a, T> std::iter::Iterator for ChildIter<'a, T> {
  type Item = &'a Tree<T>;

  fn next(&mut self) -> Option<Self::Item> {
    let last = self.0.last()?;
    let (new_nodes, tree_nodes) = self.0.split_at(self.0.len() - last.width);
    self.0 = new_nodes;
    Some(unsafe { as_tree_unchecked(tree_nodes) })
  }
}

struct ChildIterMut<'a, T>(&'a mut [Node<T>]);

impl<'a, T> std::iter::Iterator for ChildIterMut<'a, T> {
  type Item = &'a mut Tree<T>;

  fn next(&mut self) -> Option<Self::Item> {
    let last = self.0.last_mut()?;
    let width = last.width;
    let (front, back) =
      std::mem::take(&mut self.0).split_at_mut(self.0.len() - width);
    self.0 = front;
    Some(unsafe { as_tree_mut_unchecked(back) })
  }
}

impl<T> Tree<T> {
  /// Returns a reference to the value held at the root of the tree.
  pub fn root(&self) -> &T {
    &self.nodes.last().unwrap().value
  }

  /// Returns a mutable reference to the value held at the root of the tree.
  pub fn root_mut(&mut self) -> &mut T {
    &mut self.nodes.last_mut().unwrap().value
  }

  /// Retruns the number of nodes contained in the tree (including the root).
  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  /// Returns an iterator over the references to the maximal proper subtrees in
  /// reverse order.
  ///
  /// # Example:
  /// ```
  /// # use grove::{grove_buf, GroveBuf, Tree};
  /// let g: GroveBuf<i32> = grove_buf![[[1, 2, 3] => 4, 5, [6] => 7, 8] => 9];
  /// let t: &Tree<i32> = &g[8];
  /// let v: Vec<_> = t.children_rev().collect();
  /// assert_eq!(v, vec![
  ///                 &grove_buf![8i32],
  ///                 &grove_buf![[6i32] => 7],
  ///                 &grove_buf![5i32],
  ///                 &grove_buf![[1i32, 2, 3] => 4],
  ///               ]
  /// );
  /// ```
  pub fn children_rev(&self) -> impl std::iter::Iterator<Item = &Tree<T>> {
    ChildIter(&self.nodes[..self.nodes.len() - 1])
  }

  /// Analogous to [`children_rev`][Tree::children_rev] but iterates through mutable references.
  pub fn children_rev_mut(
    &mut self,
  ) -> impl std::iter::Iterator<Item = &mut Tree<T>> {
    let len = self.nodes.len() - 1;
    ChildIterMut(&mut self.nodes[..len])
  }
}

impl<T> std::ops::Index<usize> for Tree<T> {
  type Output = Tree<T>;

  fn index(&self, index: usize) -> &Self::Output {
    get_tree(&self.nodes[index])
  }
}

impl<T> std::ops::IndexMut<usize> for Tree<T> {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    get_tree_mut(&mut self.nodes[index])
  }
}
