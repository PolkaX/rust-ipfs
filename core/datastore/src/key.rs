// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Key(String);

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub const LEFT_SLASH: u8 = '/' as u8;
pub const LEFT_SLASH_STR: &'static str = "/";

/// ensure after clean, string is start with "/", and cleaned with the rule of "path_clean"
pub fn clean<S: AsRef<str>>(s: S) -> String {
    let b = s.as_ref().as_bytes();
    if b.len() == 0 {
        LEFT_SLASH_STR.to_string()
    } else if b[0] == LEFT_SLASH {
        path_clean::clean(s.as_ref())
    } else {
        let s = LEFT_SLASH_STR.to_string() + s.as_ref();
        path_clean::clean(&s)
    }
}

/*
A Key Namespace is like a path element.
A namespace can optionally include a type (delimited by ':')

    > NamespaceValue("Song:PhilosopherSong")
    PhilosopherSong
    > NamespaceType("Song:PhilosopherSong")
    Song
    > NamespaceType("Music:Song:PhilosopherSong")
    Music:Song
*/

/// `namespace_type()` is the first component of a namespace. `foo` in `foo:bar`
pub fn namespace_type(input: &str) -> &str {
    input
        .rfind(|f| f == ':')
        .map(|i| input.split_at(i).0)
        .unwrap_or(&"")
}

/// `namespace_value()` returns the last component of a namespace. `baz` in `f:b:baz`
pub fn namespace_value(input: &str) -> &str {
    input
        .rfind(|f| f == ':')
        .map(|i| input.split_at(i + 1).1)
        .unwrap_or(input)
}

impl Key {
    ///  constructs a key from string. it will clean the value.
    pub fn new<T: AsRef<str>>(s: T) -> Self {
        let k = clean(s);
        Key(k)
    }

    /// creates a new Key without safety checking the input. Use with care.
    pub fn from_raw<T: AsRef<str>>(s: T) -> Self {
        // it's safe to use &[u8] to check, due to we only check char "/"
        let c_str: &[u8] = s.as_ref().as_bytes();
        if c_str.len() == 0 {
            return Key(LEFT_SLASH_STR.to_string());
        }

        // s.len() must large than 1
        if c_str[0] != LEFT_SLASH || (c_str.len() > 1 && c_str[c_str.len() - 1] == LEFT_SLASH) {
            panic!(format!("invalid datastore key: {:}", s.as_ref()));
        }
        Key(s.as_ref().to_string())
    }

    pub fn key_with_namespace<I: AsRef<str>, V: AsRef<[I]>>(v: V) -> Self {
        let v: Vec<&str> = v.as_ref().iter().map(|i| i.as_ref()).collect();
        let s_ref_list: &[&str] = v.as_ref();
        let s = s_ref_list.join(LEFT_SLASH_STR);
        Key::new(s)
    }

    /// `random_key()` returns a randomly (uuid) generated key.
    ///   RandomKey()
    ///   NewKey("/f98719ea086343f7b71f32ea9d9d521d")
    pub fn random_key() -> Self {
        let uuid = uuid::Uuid::new_v4();
        Key::new(uuid.to_string().replace("-", ""))
    }

    pub fn clean(&mut self) {
        let new_k = clean(&self.0);
        self.0 = new_k;
    }

    /// `list()` returns the `list` representation of this Key.
    ///   NewKey("/Comedy/MontyPython/Actor:JohnCleese").List()
    ///   ["Comedy", "MontyPythong", "Actor:JohnCleese"]
    pub fn list(&self) -> Vec<&str> {
        // equal to strings.Split(k.string, "/")[1:], just ignore first item
        self.0.split(LEFT_SLASH_STR).skip(1).collect::<Vec<_>>()
    }

    pub fn split_prefix(&self) -> (Option<&str>, &str) {
        // key first char must be "/", skip check it
        let skip = 1;
        if let Some(i) = &self.0[skip..].find(LEFT_SLASH_STR) {
            let (a, b) = self.0.split_at(*i);
            (Some(a), b)
        } else {
            (None, &self)
        }
    }

    /// `reverse()` returns the reverse of this Key.
    ///   NewKey("/Comedy/MontyPython/Actor:JohnCleese").Reverse()
    ///   NewKey("/Actor:JohnCleese/MontyPython/Comedy")
    pub fn reverse(&self) -> Key {
        let mut l = self.list();
        l.reverse();
        Key::key_with_namespace(l)
    }

    /// `namespace()` returns the `namespaces` making up this Key.
    ///   NewKey("/Comedy/MontyPython/Actor:JohnCleese").Namespaces()
    ///   ["Comedy", "MontyPython", "Actor:JohnCleese"]
    pub fn namespace(&self) -> Vec<&str> {
        self.list()
    }

    pub fn base_namespace(&self) -> &str {
        self.list().last().unwrap_or(&"")
    }

    /// `type_()` returns the "type" of this key (value of last namespace).
    ///   NewKey("/Comedy/MontyPython/Actor:JohnCleese").Type()
    ///   "Actor"
    pub fn type_(&self) -> &str {
        namespace_type(self.base_namespace())
    }

    /// Name returns the "name" of this key (field of last namespace).
    ///   NewKey("/Comedy/MontyPython/Actor:JohnCleese").Name()
    ///   "JohnCleese"
    pub fn name(&self) -> &str {
        namespace_value(self.base_namespace())
    }

    /// `instance()` returns an "instance" of this type key (appends value to namespace).
    ///   NewKey("/Comedy/MontyPython/Actor").Instance("JohnClesse")
    ///   NewKey("/Comedy/MontyPython/Actor:JohnCleese")
    pub fn instance<T: AsRef<str>>(&self, s: T) -> Key {
        self.clone().into_instance(s)
    }

    pub fn into_instance<T: AsRef<str>>(self, s: T) -> Key {
        Key::new(self.0 + ":" + s.as_ref())
    }

    /// Path returns the "path" of this key (parent + type).
    ///   NewKey("/Comedy/MontyPython/Actor:JohnCleese").Path()
    ///   NewKey("/Comedy/MontyPython/Actor")
    pub fn path(&self) -> Key {
        self.clone().into_path()
    }

    pub fn into_path(self) -> Key {
        let k = self.parent();
        let s = k.0 + LEFT_SLASH_STR + namespace_type(self.base_namespace());
        Key::new(s)
    }

    /// Parent returns the `parent` Key of this Key.
    ///   NewKey("/Comedy/MontyPython/Actor:JohnCleese").Parent()
    ///   NewKey("/Comedy/MontyPython")
    pub fn parent(&self) -> Key {
        let l = self.list();
        if l.len() == 1 {
            return Key::from_raw(LEFT_SLASH_STR);
        }
        Key::new(l[..l.len() - 1].join(LEFT_SLASH_STR))
    }

    pub fn into_parent(self) -> Key {
        self.parent()
    }

    /// `child()` returns the `child` Key of this Key.
    ///   NewKey("/Comedy/MontyPython").Child(NewKey("Actor:JohnCleese"))
    ///   NewKey("/Comedy/MontyPython/Actor:JohnCleese")
    pub fn child<K: AsRef<Key> + Into<Key>>(&self, k2: K) -> Key {
        self.clone().into_child(k2)
    }

    pub fn into_child<K: AsRef<Key> + Into<Key>>(self, k2: K) -> Key {
        if self.as_str() == "/" {
            k2.into()
        } else if k2.as_ref().as_str() == "/" {
            self
        } else {
            Key::from_raw(self.0 + k2.as_ref().as_str())
        }
    }

    /// `child_string()` returns the `child` Key of this Key -- string helper.
    ///   NewKey("/Comedy/M;'
    /// ontyPython").ChildString("Actor:JohnCleese")
    ///   NewKey("/Comedy/MontyPython/Actor:JohnCleese")
    pub fn child_string<T: AsRef<str>>(&self, s: T) -> Key {
        self.clone().into_child_string(s)
    }

    pub fn into_child_string<T: AsRef<str>>(self, s: T) -> Key {
        let src = self.0;
        Key::new(src + LEFT_SLASH_STR + s.as_ref())
    }

    /// `is_ancestor_of()` returns whether this key contains another as a prefix.
    ///   NewKey("/Comedy").IsAncestorOf("/Comedy/MontyPython")
    ///   true
    pub fn is_ancestor_of<K: AsRef<str>>(&self, other: K) -> bool {
        if other.as_ref().starts_with(self.as_str()) {
            other.as_ref().as_bytes().len() > self.as_bytes().len()
        } else {
            false
        }
    }

    /// `is_descendant_of()` returns whether this key contains another as a prefix.
    ///   NewKey("/Comedy/MontyPython").IsDescendantOf("/Comedy")
    ///   true
    pub fn is_descendant_of<K: AsRef<str>>(&self, other: K) -> bool {
        if self.as_str().starts_with(other.as_ref()) {
            self.as_bytes().len() > other.as_ref().as_bytes().len()
        } else {
            false
        }
    }

    /// IsTopLevel returns whether this key has only one namespace.
    pub fn is_top_level(&self) -> bool {
        self.list().len() == 1
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        let list1 = self.list();
        let list2 = other.list();
        list1.cmp(&list2)
    }
}

impl Into<String> for Key {
    fn into(self) -> String {
        self.0
    }
}

impl Into<Vec<u8>> for Key {
    fn into(self) -> Vec<u8> {
        self.0.into_bytes()
    }
}

impl Into<Key> for &Key {
    fn into(self) -> Key {
        (*self).clone()
    }
}

impl From<String> for Key {
    fn from(s: String) -> Self {
        Key::new(s)
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        Key::new(s)
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<Key> for Key {
    fn as_ref(&self) -> &Key {
        &self
    }
}

impl AsMut<str> for Key {
    fn as_mut(&mut self) -> &mut str {
        self.0.as_mut_str()
    }
}

impl Borrow<str> for Key {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl Deref for Key {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Key {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
