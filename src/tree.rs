use std::collections::{HashMap, HashSet};

use serde_derive::{Serialize, Deserialize};

use crate::{
    id::RbxId,
    instance::RbxInstance,
};

/// Represents an instance that is rooted in a tree.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RootedRbxInstance {
    #[serde(flatten)]
    instance: RbxInstance,

    /// The unique ID of the instance
    id: RbxId,

    /// All of the children of this instance. Order is relevant to preserve!
    children: Vec<RbxId>,

    /// The parent of the instance, if there is one.
    parent: Option<RbxId>,
}

impl RootedRbxInstance {
    fn new(instance: RbxInstance, parent: Option<RbxId>) -> RootedRbxInstance {
        RootedRbxInstance {
            instance,
            id: RbxId::new(),
            parent,
            children: Vec::new(),
        }
    }

    /// Returns the unique ID associated with the rooted instance.
    pub fn get_id(&self) -> RbxId {
        self.id
    }

    /// Returns the ID of the parent of this instance, if it has a parent.
    pub fn get_parent_id(&self) -> Option<RbxId> {
        self.parent
    }

    /// Returns a list of the IDs of the children of this instance.
    pub fn get_children_ids(&self) -> &[RbxId] {
        &self.children
    }
}

impl std::ops::Deref for RootedRbxInstance {
    type Target = RbxInstance;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

/// Represents a tree containing rooted instances.
///
/// Rooted instances are described by
/// [RootedRbxInstance](struct.RootedRbxInstance.html) and have an ID, children,
/// and a parent.
#[derive(Debug, Serialize, Deserialize)]
pub struct RbxTree {
    instances: HashMap<RbxId, RootedRbxInstance>,
    root_ids: HashSet<RbxId>,
}

impl RbxTree {
    pub fn new() -> RbxTree {
        RbxTree {
            instances: HashMap::new(),
            root_ids: HashSet::new(),
        }
    }

    pub fn get_root_ids(&self) -> &HashSet<RbxId> {
        &self.root_ids
    }

    pub fn get_instance(&self, id: RbxId) -> Option<&RootedRbxInstance> {
        self.instances.get(&id)
    }

    pub fn get_instance_mut(&mut self, id: RbxId) -> Option<&mut RootedRbxInstance> {
        self.instances.get_mut(&id)
    }

    pub fn transplant(&mut self, source_tree: &mut RbxTree, source_id: RbxId, new_parent_id: Option<RbxId>) {
        let mut to_visit = vec![(source_id, new_parent_id)];

        loop {
            let (id, parent_id) = match to_visit.pop() {
                Some(id) => id,
                None => break,
            };

            let mut instance = source_tree.instances.remove(&id).unwrap();
            instance.parent = parent_id;
            instance.children.clear();

            for child in &instance.children {
                to_visit.push((*child, Some(id)));
            }

            self.insert_instance_internal(instance);
        }
    }

    fn insert_instance_internal(&mut self, instance: RootedRbxInstance) {
        match instance.parent {
            Some(parent_id) => {
                let parent = self.instances.get_mut(&parent_id)
                    .expect("Cannot insert_instance_internal into an instance not in this tree");
                parent.children.push(instance.get_id());
            },
            None => {
                self.root_ids.insert(instance.get_id());
            },
        }

        self.instances.insert(instance.get_id(), instance);
    }

    pub fn insert_instance(&mut self, instance: RbxInstance, parent_id: Option<RbxId>) -> RbxId {
        let tree_instance = RootedRbxInstance::new(instance, parent_id);
        let id = tree_instance.get_id();

        self.insert_instance_internal(tree_instance);

        id
    }

    /// Given an ID, remove the instance from the tree with that ID, along with
    /// all of its descendants.
    pub fn remove_instance(&mut self, root_id: RbxId) -> Option<RbxTree> {
        let mut ids_to_visit = vec![root_id];
        let mut new_tree_instances = HashMap::new();

        let parent_id = match self.instances.get(&root_id) {
            Some(instance) => instance.parent,
            None => return None,
        };

        match parent_id {
            Some(parent_id) => {
                let mut parent = self.get_instance_mut(parent_id).unwrap();
                let index = parent.children.iter().position(|&id| id == root_id).unwrap();

                parent.children.remove(index);
            },
            None => {
                self.root_ids.remove(&root_id);
            },
        }

        loop {
            let id = match ids_to_visit.pop() {
                Some(id) => id,
                None => break,
            };

            match self.instances.get(&id) {
                Some(instance) => ids_to_visit.extend_from_slice(&instance.children),
                None => continue,
            }

            let instance = self.instances.remove(&id).unwrap();
            new_tree_instances.insert(id, instance);
        }

        let mut root_ids = HashSet::new();
        root_ids.insert(root_id);

        Some(RbxTree {
            instances: new_tree_instances,
            root_ids,
        })
    }

    /// Returns an iterator over all of the descendants of the given instance by
    /// ID.
    pub fn descendants(&self, id: RbxId) -> Descendants {
        Descendants {
            tree: self,
            ids_to_visit: vec![id],
        }
    }
}

impl Clone for RbxTree {
    fn clone(&self) -> RbxTree {
        unimplemented!()
    }
}

pub struct Descendants<'a> {
    tree: &'a RbxTree,
    ids_to_visit: Vec<RbxId>,
}

impl<'a> Iterator for Descendants<'a> {
    type Item = &'a RootedRbxInstance;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = match self.ids_to_visit.pop() {
                Some(id) => id,
                None => break,
            };

            match self.tree.get_instance(id) {
                Some(instance) => {
                    for child_id in &instance.children {
                        self.ids_to_visit.push(*child_id);
                    }

                    return Some(instance);
                },
                None => continue,
            }
        }

        None
    }
}