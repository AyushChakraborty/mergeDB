use super::Merge;
use std::{collections::HashSet, hash::Hash};

pub struct AWSet<T>
where
    T: Eq + Hash + Clone,
{
    //tags will look like {"hiking", "rafting"}
    //where the order need not be maintained as thats not the primary task at hand.
    //The primary task here is to ensure that additions and removals of tags are eventually
    //consistent across both the nodes
    current_tags: HashSet<T>, //this is what reflects the actual set of tags
    add_tags: HashSet<T>,
    remove_tags: HashSet<T>,
}

impl<T> AWSet<T>
where
    T: Eq + Hash + Clone,
{
    fn local_merge(&mut self) {
        //method called whenever a local change is made, so that it is visible to the
        //that user instantly

        //updaing current_tags based on tags added
        for tag in &self.add_tags {
            if !self.current_tags.contains(tag) {
                self.current_tags.insert(tag.clone());
            }
        }

        //updating current_tags based on tags removed
        for tag in &self.remove_tags {
            if self.current_tags.contains(tag) {
                self.current_tags.remove(tag);
            }
        }
    }

    //TO BE CALLED ONLY AFTER MERGE AND LOCAL_MERGE ON THE NODES
    fn clear_sets(&mut self) {
        self.add_tags.clear();
        self.remove_tags.clear();
    }
}

impl<T> Merge for AWSet<T>
where
    T: Eq + Hash + Clone,
{
    fn merge(&mut self, other: &mut Self) {
        //update wrt each other

        //node1 merge

        //update node1s add_tag
        for tag in &other.add_tags {
            // TODO: REFINE AS TO WHEN TO ADD
            self.add_tags.insert(tag.clone());
        }

        //update node1s remove_tag
        for tag in &other.remove_tags {
            if !self.add_tags.contains(tag) {
                self.remove_tags.insert(tag.clone());
            }
        }

        //update node1s remove_tag again for cases when tag in remove_tag is present in node2s add_tag
        let mut to_remove: Vec<T> = Vec::new();

        for tag in &self.remove_tags {
            if !other.add_tags.contains(&tag) {
                to_remove.push(tag.clone());
            }
        }

        for tag in to_remove {
            self.remove_tags.remove(&tag);
        }

        //reason below code is commented is because this method only deals with merge of
        //other into self, and for the other way around, the method must be called as such. Again this is
        //a design choice

        // //node2 merge

        // //update node2s add_tag
        // for tag in &temp_add_tags {
        //     other.add_tags.insert(tag.clone());
        // }

        // //update node2s remove tag
        // for tag in &temp_remove_tags {
        //     if !other.add_tags.contains(tag) {
        //         other.remove_tags.insert(tag.clone());
        //     }
        // }

        // //update node2s remove_tag again for cases when tag in remove_tag is present in node1s add_tag
        // let mut to_remove_2: Vec<T> = Vec::new();

        // for tag in &other.remove_tags {
        //     if !temp_add_tags.contains(&tag) {
        //         to_remove_2.push(tag.clone());
        //     }
        // }

        // for tag in to_remove_2 {
        //     other.remove_tags.remove(&tag);
        // }
    }
}

//TODO: unit tests
