use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;
use std::slice::IterMut;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct TreeToken(usize);

impl TreeToken {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}

impl From<TreeToken> for usize {
    fn from(value: TreeToken) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub struct TreeNode<T> {
    value: T,
    parent: Option<TreeToken>,
}

impl<T> TreeNode<T> {
    pub fn new(value: T, parent: Option<TreeToken>) -> Self {
        Self { value, parent }
    }
    pub fn parent(&self) -> Option<TreeToken> {
        self.parent
    }
    pub fn set_parent(&mut self, tt: TreeToken) {
        self.parent = Some(tt);
    }
}

impl<T> Deref for TreeNode<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T> DerefMut for TreeNode<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug)]
pub struct Tree<T> {
    pool: Vec<TreeNode<T>>,
}
impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self {
            pool: Vec::new()
        }
    }
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, parent: Option<TreeToken>, id: TreeToken, value: T) -> TreeToken {
        let len = self.pool.len();
        assert_eq!(len, id.0);
        self.pool.push(TreeNode { value, parent });
        TreeToken(len)
    }

    pub fn push(&mut self, parent: Option<TreeToken>, value: T) -> TreeToken {
        let id = self.pool.len();
        self.pool.push(TreeNode { value, parent });
        TreeToken(id)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, TreeNode<T>> {
        self.pool.iter_mut()
    }
}

impl<T> FromIterator<TreeNode<T>> for Tree<T> {
    fn from_iter<I: IntoIterator<Item = TreeNode<T>>>(iter: I) -> Self {
        Self {
            pool: iter.into_iter().collect::<Vec<_>>(),
        }
    }
}
// impl<T> From<Vec<TreeNode<T>>> for Tree<T> {
//     fn from(pool: Vec<TreeNode<T>>) -> Self {
//         Self { pool }
//     }
// }

impl<T> Index<TreeToken> for Tree<T> {
    type Output = TreeNode<T>;

    fn index(&self, index: TreeToken) -> &Self::Output {
        &self.pool[index.0]
    }
}

impl<T> IndexMut<TreeToken> for Tree<T> {
    fn index_mut(&mut self, index: TreeToken) -> &mut Self::Output {
        &mut self.pool[index.0]
    }
}
