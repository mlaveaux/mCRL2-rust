use crate::{LabelledTransitionSystem, Partition};

pub fn strong_bisim_partition(lts: &LabelledTransitionSystem) -> Partition {

    // Compute the initial partition, a single constellation consisting of
    // blocks with states that have different outgoing labels.
    let mut partition = Partition::new(lts.states.len());

    for (label_index, label) in lts.labels.iter().enumerate() {
        partition.split(0, |element| {
            lts.states[element].outgoing.iter().any(|(label, _)| {
                *label == label_index
            })
        })
    }

    
    let mut constellation = Partition::new(partition.number_of_blocks());


    let mut work = vec![0];

    // Take an arbitrary constellation that is not trivial   
    while let Some(constellation_index) = work.pop() {

        // Pick a block in constellation that is not the largest.
        let mut blocks = constellation.iter_block(constellation_index);
        let first_block = blocks.next().unwrap();

        let smallest_block = match blocks.next() {
            Some(block) => {
                if constellation.block(block).len() < constellation.block(first_block).len() {
                    block
                } else {
                    first_block
                }
            },
            None => {
                first_block
            }
        };



    }

    partition
}

#[cfg(test)]
mod tests {
    use log::trace;
    use test_log::test;
    
    use crate::random_lts;

    use super::*;

    #[test]
    fn test_random_bisim_partition() {
        let lts = random_lts(10, 3);

        trace!("{lts:?}");

        strong_bisim_partition(&lts);
    }
}