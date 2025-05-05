use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Clone, Eq)]
pub struct Node<T> {
  pub(crate) value: T,
  pub(crate) width: usize,
}

impl<T: Debug> Debug for Node<T> {
  fn fmt(&self, f: &mut Formatter) -> Result {
    self.value.fmt(f)
  }
}

impl<T: Display> Display for Node<T> {
  fn fmt(&self, f: &mut Formatter) -> Result {
    self.value.fmt(f)
  }
}

impl<T, U: PartialEq<T>> PartialEq<Node<T>> for Node<U> {
  fn eq(&self, n: &Node<T>) -> bool {
    self.value == n.value && self.width == n.width
  }
}
