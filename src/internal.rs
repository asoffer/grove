#![allow(private_bounds)]

use crate::grove::Grove;
use crate::node::Node;
use crate::tree::Tree;

pub(crate) trait Internal {}

pub struct Zero;

pub struct Succ<N: crate::internal::Internal> {
  pub(crate) stashed: N,
  pub(crate) position: usize,
}

impl Internal for Zero {}
impl<N: Internal> Internal for Succ<N> {}

pub(crate) unsafe fn as_tree_unchecked<T>(nodes: &[Node<T>]) -> &Tree<T> {
  &*(nodes as *const [Node<T>] as *const Tree<T>)
}

pub(crate) fn get_tree<T>(node: &Node<T>) -> &Tree<T> {
  unsafe {
    let slice = std::slice::from_raw_parts(
      (node as *const Node<T>).offset(-(node.width as isize) + 1),
      node.width,
    );
    as_tree_unchecked(slice)
  }
}

pub(crate) unsafe fn as_tree_mut_unchecked<T>(
  slice: &mut [Node<T>],
) -> &mut Tree<T> {
  &mut *(slice as *mut [Node<T>] as *mut Tree<T>)
}

pub(crate) fn get_tree_mut<T>(node: &mut Node<T>) -> &mut Tree<T> {
  unsafe {
    as_tree_mut_unchecked(std::slice::from_raw_parts_mut(
      (node as *mut Node<T>).offset(-(node.width as isize) + 1),
      node.width,
    ))
  }
}

pub(crate) unsafe fn as_grove_unchecked<T>(slice: &[Node<T>]) -> &Grove<T> {
  &*(slice as *const [Node<T>] as *const Grove<T>)
}

pub(crate) unsafe fn as_grove_mut_unchecked<T>(
  slice: &mut [Node<T>],
) -> &mut Grove<T> {
  &mut *(slice as *mut [Node<T>] as *mut Grove<T>)
}
