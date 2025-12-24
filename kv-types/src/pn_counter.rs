use super::Merge;

#[derive(Clone)]
pub struct PNCounter {
    p: u64,
    n: u64,
}

impl Merge for PNCounter {
    //when merged, both the replicas get to a common state
    fn merge(&mut self, other: &mut Self) {
        self.n += other.n;
        self.p += other.p;
    }
}

impl PNCounter {
    fn new(p: u64, n: u64) -> Self {
        PNCounter {p: p, n: n}
    }
    
    fn increment(&mut self) {
        self.p += 1; 
    }
    
    fn decrement(&mut self) {
        self.n += 1
    }
    
    //for the user of the node to see the value of the counter
    fn value(&self) -> u64{
        self.p - self.n
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_local_increments_and_decremenets() {
        let mut counter = PNCounter::new(0, 0);
        counter.increment();
        counter.increment();
        counter.decrement();
        
        assert_eq!(counter.value(), 1);
    }
    
    #[test]
    fn merge_maintains_total() {
        let mut replica_a = PNCounter::new(0, 0);
        replica_a.increment();     //becomes 1 now
        
        let mut replica_b = PNCounter::new(1, 0);
        replica_b.increment();     //becomes 2 now
        
        //merge b's state to a
        replica_a.merge(&mut replica_b);
        
        assert_eq!(replica_a.value(), 3);      //as it should get b's value now

        let mut replica_c = PNCounter::new(0, 0);
        replica_c.increment();
        replica_c.increment();
        replica_c.decrement();

        let mut replica_d = PNCounter::new(0, 0);
        replica_d.increment();
        replica_d.increment();
        replica_d.increment();

        replica_c.merge(&mut replica_d);
        assert_eq!(replica_c.value(), 4);

    }
    
    #[test]
    fn test_merge_is_commutative() {
        let mut replica_a = PNCounter::new(0, 0);
        replica_a.increment();
    
        let mut replica_b = PNCounter::new(1, 0);
        replica_b.decrement();
    
        let mut a_then_b = replica_a.clone();
        a_then_b.merge(&mut replica_b);
        
        let mut b_then_a = replica_b.clone();
        b_then_a.merge(&mut replica_a);
    
        //the final state must be identical regardless of merge order
        assert_eq!(a_then_b.value(), b_then_a.value());
    }
}
