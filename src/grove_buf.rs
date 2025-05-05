use crate::grove::Grove;
use crate::internal;
use crate::internal::as_grove_mut_unchecked;
use crate::internal::as_grove_unchecked;
use crate::node::Node;
use crate::traversal::TraversalOrder;
use crate::tree::Tree;

/// A sequence of trees structured so that nodes can be efficiently visited in
/// pre-order or reverse post-order. For any node, its children can also be
/// efficiently visited. All nodes are stored within a single allocation.
/// The structure is append-only, so once a subtree has been formed, one can
/// no longer modify it. In particular, this means that all children must be
/// appended before a (sub)tree's root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroveBuf<T> {
  pub(crate) nodes: Vec<Node<T>>,
}

/// Constructs a [`GroveBuf`] containing no trees.
impl<T> Default for GroveBuf<T> {
  fn default() -> GroveBuf<T> {
    GroveBuf { nodes: vec![] }
  }
}

/// A utility type for appending to a [`GroveBuf`] safely. This type is only
/// constructible by invoking [`builder`][GroveBuf::builder] on a [`GroveBuf`],
/// or by calling [`open`][GroveBufBuilder::open] on an existing
/// [`GroveBufBuilder`]. Each call to [`open`][GroveBufBuilder::open] indicates
/// that further calls to [`push`][GroveBufBuilder::push] will be at one extra
/// layer of depth in the tree construction. Each call to
/// [`close`][GroveBufBuilder::close] accepts the value for the parent of the
/// nodes at the prior depth. The type of the builder tracks the number of calls
/// to [`open`][GroveBufBuilder::open] and ensures at compile-time that
///
/// * [`close`][GroveBufBuilder::close] is not called more times than the
///   current working depth of the [`GroveBuf`] (the number of calls to
///   [`open`][GroveBufBuilder::open] minus the number of calls to
///   [`close`][GroveBufBuilder::close]).
/// * [`build`][GroveBufBuilder::build] is not invocable if there are more calls
///   to `open` than calls to [`close`][GroveBufBuilder::close].
///
/// It is possible that more calls to [`open`][GroveBufBuilder::open] are present
/// than calls to [`close`][GroveBufBuilder::close], in which case the behavior
/// is as if those calls to [`open`][GroveBufBuilder::open] that would correspond
/// to the missing calls to [`close`][GroveBufBuilder::close] were never made
/// and an implicit [`build`][GroveBufBuilder::build] is present at the end.
///
/// With this last consideration it is, a valid these rules ensure at
/// compile-time that a valid [`GroveBuf`] is always properly constructed.
pub struct GroveBufBuilder<'a, T, N: internal::Internal>(
  &'a mut GroveBuf<T>,
  N,
);

impl<'a, T, N: internal::Internal> GroveBufBuilder<'a, T, N> {
  /// Add a new leaf in the [`GroveBuf`] at the appropriate depth.
  pub fn push(self, value: T) -> GroveBufBuilder<'a, T, N> {
    self.0.push(value);
    self
  }

  /// Indicate that a new layer of tree depth is being started. There must be a
  /// corresponding call to [`close`][GroveBufBuilder::close] to match this call
  /// to [`open`][GroveBufBuilder::open]. Failure to do so will result in a
  /// compilation failure.
  pub fn open(self) -> GroveBufBuilder<'a, T, internal::Succ<N>> {
    let len = self.0.len();
    GroveBufBuilder(self.0, internal::Succ { stashed: self.1, position: len })
  }
}

impl<'a, T, N: internal::Internal> GroveBufBuilder<'a, T, internal::Succ<N>> {
  /// Consumes `self` and returns a [`GroveBufBuilder`] of depth one smaller. than
  /// that of `self`. Adds a new tree to the referenced [`GroveBuf`] whose children
  /// consist of those nodes and subtrees constructed since the corresponding
  /// call to [`open`][GroveBufBuilder::open].
  pub fn close(self, value: T) -> GroveBufBuilder<'a, T, N> {
    unsafe {
      self.0.push_unchecked(value, self.1.position);
    }
    GroveBufBuilder(self.0, self.1.stashed)
  }
}

impl<'a, T> GroveBufBuilder<'a, T, internal::Zero> {
  /// Consumes `self`, returning a mutable reference to the underlying
  /// [`GroveBuf`].
  pub fn build(self) -> &'a mut GroveBuf<T> {
    self.0
  }
}

impl<T> GroveBuf<T> {
  /// Constructs a [`GroveBuf`] containing no trees.
  pub fn new() -> GroveBuf<T> {
    Default::default()
  }

  /// Returns `true` if and only if the [`GroveBuf`] contains no trees.
  pub fn is_empty(&self) -> bool {
    self.nodes.is_empty()
  }

  /// Returns the number of nodes in the [`GroveBuf`].
  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  /// Returns a `&Grove<T>` referring to `&self`.
  pub fn as_ref(&self) -> &Grove<T> {
    unsafe { as_grove_unchecked(&self.nodes) }
  }

  /// Returns a `&mut Grove<T>` referring to `&mut self`.
  pub fn as_mut(&mut self) -> &mut Grove<T> {
    unsafe { as_grove_mut_unchecked(&mut self.nodes) }
  }

  /// Returns an iterator over references to the nodes in the grove according
  /// to the specified treversal `order`.
  pub fn nodes<Order: TraversalOrder>(
    &self,
    order: Order,
  ) -> impl std::iter::Iterator<Item = &T> {
    self.as_ref().nodes(order)
  }

  /// Returns an iterator over mutable references to the nodes in the grove
  /// according to the specified treversal `order`.
  pub fn nodes_mut<Order: TraversalOrder>(
    &mut self,
    order: Order,
  ) -> impl std::iter::Iterator<Item = &mut T> {
    self.as_mut().nodes_mut(order)
  }

  /// Returns an iterator over references to the trees in the grove according
  /// to the specified treversal `order`. This includes all subtrees, not just
  /// the top-level trees. For example, when iterating through
  ///
  /// ```
  /// # use grove::grove_buf;
  /// grove_buf![[[1, 2] => 3, [4, 5] => 6] => 7, 8];
  /// ```
  ///
  /// One would obtain the subtrees equivalent to
  /// ```
  /// # use grove::grove_buf;
  /// grove_buf![1];
  /// grove_buf![2];
  /// grove_buf![[1, 2] => 3];
  /// grove_buf![4];
  /// grove_buf![5];
  /// grove_buf![[4, 5] => 6];
  /// grove_buf![[[1, 2] => 3, [4, 5] => 6] => 7];
  /// grove_buf![8];
  /// ```
  ///
  pub fn trees<Order: TraversalOrder>(
    &self,
    order: Order,
  ) -> impl std::iter::Iterator<Item = &Tree<T>> {
    self.as_ref().trees(order)
  }

  /// Returns an iterator over mutable references to the trees in the grove
  /// according to the specified treversal `order`.
  pub fn trees_mut<Order: TraversalOrder>(
    &mut self,
    order: Order,
  ) -> impl std::iter::Iterator<Item = &mut Tree<T>> {
    self.as_mut().trees_mut(order)
  }

  /// Appends a leaf with value `value` to the grove.
  pub fn push(&mut self, value: T) {
    self.nodes.push(Node { value, width: 1 });
  }

  /// Appends a node with value `value` that contains `children` child nodes.
  pub fn push_root(&mut self, value: T, children: usize) {
    let mut element = self.nodes.len();
    for _ in 0..children {
      element -= self[element - 1].len();
    }
    unsafe {
      self.push_unchecked(value, element);
    }
  }

  /// Constructs a new `GroveBufBuilder` from which one can safely push nodes
  /// into the [`GroveBuf`]
  pub fn builder(&mut self) -> GroveBufBuilder<'_, T, internal::Zero> {
    GroveBufBuilder(self, internal::Zero)
  }

  /// Appends a node with value `value` that contains all elements at index
  /// `position` and larger in its subtree. It is the callers responsibility
  /// to ensure that no elements are with index smaller than `position` are
  /// already contained in a subtree whose root is greater than or equal to
  /// `position`.
  pub unsafe fn push_unchecked(&mut self, value: T, position: usize) {
    self.nodes.push(Node {
      value,
      width: self.nodes.len() - position + 1,
    });
  }
}

impl<'a, T> From<&'a GroveBuf<T>> for &'a Grove<T> {
  fn from(g: &'a GroveBuf<T>) -> &'a Grove<T> {
    g.as_ref()
  }
}

impl<'a, T> From<&'a mut GroveBuf<T>> for &'a mut Grove<T> {
  fn from(g: &'a mut GroveBuf<T>) -> &'a mut Grove<T> {
    g.as_mut()
  }
}

impl<T> std::ops::Index<usize> for GroveBuf<T> {
  type Output = Tree<T>;

  /// Returns a reference to the tree whose root has the given index.
  fn index(&self, index: usize) -> &Self::Output {
    &self.as_ref()[index]
  }
}

impl<T> std::ops::IndexMut<usize> for GroveBuf<T> {
  /// Returns a reference to the tree whose root has the given index.
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut self.as_mut()[index]
  }
}

#[macro_export]
#[cfg(not(doc))]
macro_rules! grove_buf_impl {
  ($id:ident; [$($children:tt)*] => $root:expr, $($rest:tt)*) => {{
    let builder = $crate::grove_buf_impl![$id; [$($children)*] => $root];
    $crate::grove_buf_impl![builder; $($rest)*]
  }};
  ($id:ident; [$($children:tt)*] => $root:expr) => {{
    let builder = $id.open();
    let builder = $crate::grove_buf_impl![builder; $($children)*];
    builder.close($root)
  }};
  ($id:ident; $e:expr, $($rest:tt)*) => {{
    let builder = $id.push($e);
    $crate::grove_buf_impl![builder; $($rest)*]
  }};
  ($id:ident; $e:expr) => {
    $id.push($e)
  };
}

/// Constructs a `GroveBuf<T>`. Consists of a comma-separated sequence of either
/// * expressions of type `T`, or
/// * the syntactic form `[subtrees...] => root`
///
/// The former case represents a leaf of a tree in the [`GroveBuf`]. The latter
/// represents a tree whose root contains the value `root`, and whose children
/// are `subtrees`. Each such subtree must recursively have one of the syntactic
/// forms described above.
#[macro_export]
macro_rules! grove_buf {
  () => { GroveBuf::new() };
  ([$($children:tt)*] => $root:expr, $($rest:tt)*) => {{
    let mut g = $crate::GroveBuf::new();
    let builder = g.builder();
    let builder = $crate::grove_buf_impl![builder; [$($children)*] => $root];
    let builder = $crate::grove_buf_impl![builder; $($rest)*];
    std::mem::take(builder.build())
  }};
  ([$($children:tt)*] => $root:expr) => {{
    let mut g = $crate::GroveBuf::new();
    let builder = g.builder();
    let builder = $crate::grove_buf_impl![builder; [$($children)*] => $root];
    std::mem::take(builder.build())
  }};
  ($e:expr, $($rest:tt)*) => {{
    let mut g = $crate::GroveBuf::new();
    let builder = g.builder();
    let builder = $crate::grove_buf_impl![builder; $e];
    let builder = $crate::grove_buf_impl![builder; $($rest)*];
    std::mem::take(builder.build())
  }};
  ($e:expr) => {{
    let mut g = $crate::GroveBuf::new();
    let builder = g.builder();
    let builder = $crate::grove_buf_impl![builder; $e];
    std::mem::take(builder.build())
  }};
}

impl<T, U: PartialEq<T>> PartialEq<Tree<T>> for GroveBuf<U> {
  fn eq(&self, t: &Tree<T>) -> bool {
    self.nodes == t.nodes
  }
}

impl<T, U: PartialEq<T>> PartialEq<GroveBuf<T>> for Tree<U> {
  fn eq(&self, t: &GroveBuf<T>) -> bool {
    self.nodes == t.nodes
  }
}

impl<T, U: PartialEq<T>> PartialEq<Grove<T>> for GroveBuf<U> {
  fn eq(&self, g: &Grove<T>) -> bool {
    self.nodes == g.nodes
  }
}

impl<T, U: PartialEq<T>> PartialEq<GroveBuf<T>> for Grove<U> {
  fn eq(&self, g: &GroveBuf<T>) -> bool {
    self.nodes == g.nodes
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::traversal::Preorder;

  #[test]
  fn new() {
    let g = GroveBuf::<i32>::new();
    assert!(g.is_empty());
    assert_eq!(g.len(), 0);
    let nodes: Vec<_> = g.nodes(Preorder).collect();
    assert_eq!(nodes, Vec::<&i32>::new());

    let nodes: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(nodes, vec![]);
  }

  #[test]
  fn default() {
    let g: GroveBuf<i32> = Default::default();
    assert!(g.is_empty());
    assert_eq!(g.len(), 0);
    let nodes: Vec<_> = g.nodes(Preorder).collect();
    assert_eq!(nodes, Vec::<&i32>::new());

    let nodes: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(nodes, vec![]);
  }

  #[test]
  fn leaf() {
    let mut g: GroveBuf<i32> = Default::default();
    g.push(3);
    assert!(!g.is_empty());
    assert_eq!(g.len(), 1);
    let nodes: Vec<_> = g.nodes(Preorder).collect();
    assert_eq!(nodes, vec![&3]);

    let nodes: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(nodes, vec![1]);
  }

  #[test]
  fn leaves() {
    let mut g: GroveBuf<i32> = Default::default();
    g.push(3);
    g.push(4);
    assert!(!g.is_empty());
    assert_eq!(g.len(), 2);
    let nodes: Vec<_> = g.nodes(Preorder).collect();
    assert_eq!(nodes, vec![&3, &4]);

    let widths: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(widths, vec![1, 1]);
  }

  #[test]
  fn tree() {
    let mut g: GroveBuf<i32> = Default::default();
    g.push(3);
    g.push(4);
    g.push_root(5, 2);
    assert!(!g.is_empty());
    assert_eq!(g.len(), 3);
    let nodes: Vec<_> = g.nodes(Preorder).collect();
    assert_eq!(nodes, vec![&3, &4, &5]);

    let widths: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(widths, vec![1, 1, 3]);
  }

  #[test]
  fn multi_level_tree() {
    let mut g: GroveBuf<i32> = Default::default();
    g.push(3);
    g.push(4);
    g.push_root(5, 2);
    g.push(6);
    g.push(7);
    g.push_root(8, 2);
    g.push_root(9, 2);
    assert!(!g.is_empty());
    assert_eq!(g.len(), 7);
    let nodes: Vec<_> = g.nodes(Preorder).collect();
    assert_eq!(nodes, vec![&3, &4, &5, &6, &7, &8, &9]);

    let widths: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(widths, vec![1, 1, 3, 1, 1, 3, 7]);
  }

  #[test]
  fn empty_macro() {
    let g: GroveBuf<i32> = grove_buf![];
    assert!(g.is_empty());
  }

  #[test]
  fn one_leaf_macro() {
    let g: GroveBuf<i32> = grove_buf![1];
    assert_eq!(g.len(), 1);
    let nodes: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(nodes, vec![1]);
  }

  #[test]
  fn many_leaves_macro() {
    let g: GroveBuf<i32> = grove_buf![1, 2, 3];
    assert_eq!(g.len(), 3);
    let nodes: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(nodes, vec![1, 1, 1]);
  }

  #[test]
  fn many_trees_macro() {
    let g: GroveBuf<i32> = grove_buf![1, [2] => 3, [4] => 5, 6];
    assert_eq!(g.len(), 6);
    let nodes: Vec<_> = g.nodes(Preorder).collect();
    assert_eq!(nodes, vec![&1, &2, &3, &4, &5, &6]);

    let widths: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(widths, vec![1, 1, 2, 1, 2, 1]);
  }

  #[test]
  fn path_macro() {
    let g: GroveBuf<i32> = grove_buf![[[3] => 2] => 1];
    assert_eq!(g.len(), 3);
    let nodes: Vec<_> = g.nodes(Preorder).collect();
    assert_eq!(nodes, vec![&3, &2, &1]);

    let widths: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(widths, vec![1, 2, 3]);
  }

  fn complex_example() -> GroveBuf<i32> {
    grove_buf![
      [
        [
          [1, 2] => 3,
          [4, 5] => 6
        ] => 7,
        8,
        [
          [9, 10] => 11,
          [12, 13] => 14
        ] => 15
      ] => 16
    ]
  }

  #[test]
  fn full_tree_macro() {
    let g = complex_example();
    assert_eq!(g.len(), 16);
    let nodes: Vec<_> = g.nodes(Preorder).collect();
    assert_eq!(
      nodes,
      vec![
        &1, &2, &3, &4, &5, &6, &7, &8, &9, &10, &11, &12, &13, &14, &15, &16
      ]
    );

    let widths: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(widths, vec![1, 1, 3, 1, 1, 3, 7, 1, 1, 1, 3, 1, 1, 3, 7, 16]);
  }

  #[test]
  fn builder() {
    let mut g: GroveBuf<i32> = Default::default();
    g.builder()
      .open()
      .open()
      .open()
      .push(1)
      .push(2)
      .close(3)
      .open()
      .push(4)
      .push(5)
      .close(6)
      .close(7)
      .push(8)
      .open()
      .open()
      .push(9)
      .push(10)
      .close(11)
      .open()
      .push(12)
      .push(13)
      .close(14)
      .close(15)
      .close(16)
      .build();

    let nodes: Vec<_> = g.nodes(Preorder).collect();
    assert_eq!(
      nodes,
      vec![
        &1, &2, &3, &4, &5, &6, &7, &8, &9, &10, &11, &12, &13, &14, &15, &16
      ]
    );

    let widths: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(widths, vec![1, 1, 3, 1, 1, 3, 7, 1, 1, 1, 3, 1, 1, 3, 7, 16]);
  }

  #[test]
  fn nodes_mut() {
    let mut g = complex_example();
    for node in g.nodes_mut(Preorder) {
      *node += 1;
    }
    let nodes: Vec<_> = g.nodes(Preorder).collect();
    assert_eq!(
      nodes,
      vec![
        &2, &3, &4, &5, &6, &7, &8, &9, &10, &11, &12, &13, &14, &15, &16, &17
      ]
    );

    let widths: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(widths, vec![1, 1, 3, 1, 1, 3, 7, 1, 1, 1, 3, 1, 1, 3, 7, 16]);
  }

  #[test]
  fn trees() {
    let g = complex_example();
    assert_eq!(g.len(), 16);
    let len_and_roots: Vec<_> =
      g.trees(Preorder).map(|t| (t.len(), t.root())).collect();
    assert_eq!(
      len_and_roots,
      vec![
        (1, &1),
        (1, &2),
        (3, &3),
        (1, &4),
        (1, &5),
        (3, &6),
        (7, &7),
        (1, &8),
        (1, &9),
        (1, &10),
        (3, &11),
        (1, &12),
        (1, &13),
        (3, &14),
        (7, &15),
        (16, &16)
      ]
    );
  }

  #[test]
  fn trees_mut() {
    let mut g = complex_example();
    assert_eq!(g.len(), 16);
    for tree in g.trees_mut(Preorder) {
      *tree.root_mut() += 1;
    }

    let nodes: Vec<_> = g.nodes(Preorder).collect();
    assert_eq!(
      nodes,
      vec![
        &2, &3, &4, &5, &6, &7, &8, &9, &10, &11, &12, &13, &14, &15, &16, &17
      ]
    );

    let widths: Vec<_> = g.trees(Preorder).map(Tree::len).collect();
    assert_eq!(widths, vec![1, 1, 3, 1, 1, 3, 7, 1, 1, 1, 3, 1, 1, 3, 7, 16]);
  }

  #[test]
  fn index() {
    let g = complex_example();
    assert_eq!(g[0], grove_buf![1 as i32]);
    assert_eq!(g[1], grove_buf![2 as i32]);
    assert_eq!(g[2], grove_buf![[1 as i32, 2] => 3]);
  }
}
