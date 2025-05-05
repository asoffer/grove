# Grove

## Overview

Grove is a library for representing sequences of trees in a single flat
buffer, in a manner that makes common traversals efficiently. To achieve
this, certain uncommon mutations are not feasible.

The name "grove" is intended to be indicative of the structure: A linear
sequence of trees. All nodes in the sequence of trees are layed out in a
single buffer, along with data that indicates how many elements are in each
subtree. For example, to repreesnt the two trees

```text
     "primary color"          "direction"
     /      |      \            /      \
 "red"  "yellow"  "blue"    "left"   "right"
```

The data would be stored in the format:

```text
[
  ("red", 1),          <---+
  ("yellow", 1),           |
  ("blue", 1),             |
  ("primary color", 4), ---+
  ("left", 1),         <---+
  ("right", 1),            |        
  ("direction", 3),     ---+
]
```

Note that traversing these nodes in order visits each node in pre-order,
and traversing in reverse order visits each node in revers post-order.

Moreover, one can quickly visit each tree root by traversing in reverse
order and skipping the width of each tree. One can visit the children of any
node in a similar fashion. The library provides utilities for each such
iteration

## Usage

The easiest way to construct a [`GroveBuf`] is with via the [`grove_buf`]
macro:

```rust
# use grove::grove_buf;
let g = grove_buf![
  ["red", "yellow", "blue"] => "primary color",
  ["left", "right"] => "direction"
];
```

See the member functions on [`Grove`] and [`GroveBuf`] for facilities for
facilities for querying and traversing.


