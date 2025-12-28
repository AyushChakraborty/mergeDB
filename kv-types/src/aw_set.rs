use super::Merge;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Debug)]
pub struct AWSet<T>
where
    T: Eq + Hash + Clone,
{
    //tags will look like {"hiking": {version_nos...}, "rafting": {version_nos...}}

    //version_nos are used to uniquely identify which element with the same name is present
    //in the map

    //where the order need not be maintained as thats not the primary task at hand.
    //The primary task here is to ensure that additions and removals of tags are eventually
    //consistent across both the nodes
    pub current_tags: HashSet<T>, //this is what reflects the actual set of tags
    add_tags: HashMap<T, (u32, Vec<u32>)>, //to not make finding the length O(n) time
    remove_tags: HashMap<T, (u32, Vec<u32>)>,
}

impl<T> AWSet<T>
where
    T: Eq + Hash + Clone,
{
    //PRINCIPLE FOLLOWED: never remove from add_tags or the remove_tags map, they are meant
    //to be totally self contained, and have the history of all the operations

    //user can only deal with the current_tags set, the add_tags and remove_tags are
    //private to the definition of the type

    fn new(current_tags: HashSet<T>) -> Self {
        AWSet {
            current_tags: current_tags,
            add_tags: HashMap::new(),
            remove_tags: HashMap::new(),
        }
    }

    fn add_tag(&mut self, tag: T) {
        //since its a set eventually, if the tag is not already in remove_tags map,
        //and its not already in add_tags map, only then add it, in all other cases

        let mut version_num = 0;
        if self.add_tags.contains_key(&tag) {
            version_num = self.add_tags.get(&tag).expect("could not extract value").0;
            //this way, the version number for the same tag
            //remains consistent across nodes, so this helps during merge()
        }

        //if tag does not already exist in both of the maps OR
        if !self.add_tags.contains_key(&tag) || !self.remove_tags.contains_key(&tag) {
            self.add_tags.insert(tag, (1, vec![version_num]));
        }
        //if lens of both the vectors for that tag are the same, it means that that tag no longer exists in the set
        //(equal number of additions and removals), so can add
        else if self.add_tags.get(&tag).expect("could not extract value").0
            == self
                .remove_tags
                .get(&tag)
                .expect("could not extract value")
                .0
        {
            let val = self
                .add_tags
                .get_mut(&tag)
                .expect("could not extract value");
            val.0 += 1; //incrementing the len by 1
            val.1.push(version_num);
        }
    }

    fn remove_tag(&mut self, tag: T) {
        if !self.add_tags.contains_key(&tag) {
            println!("tag has not been inserted already!");
            return;
        }

        let version_index_to_be_deleted: usize =
            (self.add_tags.get(&tag).expect("could not extract value").0 - 1) as usize;
        let version_to_be_deleted = self.add_tags.get(&tag).expect("could not extract value").1
            [version_index_to_be_deleted];

        if !self.remove_tags.contains_key(&tag) {
            let val = self
                .remove_tags
                .get_mut(&tag)
                .expect("could not extract value");
            val.0 = 1;
            val.1 = vec![version_to_be_deleted];
        } else {
            let val = self
                .remove_tags
                .get_mut(&tag)
                .expect("could not extract value");
            val.0 += 1;
            val.1.push(version_to_be_deleted);
        }
    }

    //this method emulates the commit action within a node
    //SHLD BE CALLED AFTER IT HAS MERGED ITSELF WITH ALL OTHER NODES
    //POSSIBLE DISCREPANCY, SINCE WHEN IT WILL BE MERGED IS "EVENTUAL"
    //DONT CALL THIS METHOD FOR NOW
    fn local_merge(&mut self) {
        //method called whenever a local change is made, so that it is visible to the
        //that user instantly

        //updaing current_tags based on tags added
        for tag in self.add_tags.keys() {
            if !self.current_tags.contains(tag) {
                self.current_tags.insert(tag.clone());
            }
        }

        //updating current_tags based on tags removed
        for tag in self.remove_tags.keys() {
            if self.current_tags.contains(tag) {
                self.current_tags.remove(tag);
            }
        }
    }

    //TO BE CALLED ONLY AFTER MERGE AND LOCAL_MERGE ON THE NODES, as there is no need
    //to maintain the add and remove tag maps
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

        //node1 merge, so for some tag t1, go through all of its version numbers in remove_tags map, and if a version
        //number is in the other node's add_tags map for that tag t1 and not in the other node's remove_tags map,
        //then retain it, hence the name AddWins Set

        //point here is to set self.current_tags

        //call local_merge method first to get the initial current_tags
        self.local_merge();

        for tag in self.remove_tags.keys() {
            let self_removed_values = &self.remove_tags.get(tag).expect("could not get value").1;
            //let self_add_values = &self.add_tags.get(tag).expect("could not get value").1;
            let other_remove_values = &other.remove_tags.get(tag).expect("could not get value").1;
            let other_add_values = &other.add_tags.get(tag).expect("could not get value").1;
            for v_no in self_removed_values {
                if other_add_values.contains(&v_no) && !other_remove_values.contains(&v_no) {
                    self.current_tags.insert(tag.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_tag() {
        let mut replica_a: AWSet<String> = AWSet::new(HashSet::new());
        let tag = String::from("apple");
        replica_a.add_tag(tag.clone());
        assert_eq!(replica_a.add_tags.contains_key(&tag), true);
    }
}
