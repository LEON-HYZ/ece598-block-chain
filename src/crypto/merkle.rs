use super::hash::{Hashable, H256};
use ring::{digest};

/// A Merkle tree.
//Use Option<Box<>> for Tree implementation
//Tree implementation reference: https://gist.github.com/aidanhs/5ac9088ca0f6bdd4a370
#[derive(Debug, Default, Clone)]
pub struct TreeNode {
    val: H256,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

#[derive(Debug, Default, Clone)]
pub struct MerkleTree {
    root: Option<Box<TreeNode>>,
    length: usize,
    height: usize,
    //left: Option<Box<TreeNode>>,
    //right: Option<Box<TreeNode>>,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        //unimplemented!()
        let length = data.len();
        let height = getHeight(length);

        if length == 0 { return MerkleTree{ root:None ,length:length , height:height}; }
        
        //println!("height:{}, length:{}", height, length );

        let mut leafNodes = Vec::with_capacity(length);
        //insert leaf nodes fisrt
        for elem in data{
            leafNodes.push(TreeNode{val:elem.hash(),left:None,right:None});
            println!("children: {:?}", elem.hash());
        }
        if length % 2 == 1 {
            leafNodes.push(TreeNode{val:leafNodes[length-1].val,left:None,right:None});
        }
        //build tree
        let mut childrenNodes = leafNodes.clone();
        let mut curLength = childrenNodes.len();
        while curLength > 1 {
            
            let mut TreeNodes:Vec<TreeNode> = Vec::new();

            if curLength % 2 == 1 {
                leafNodes.push(TreeNode{val:leafNodes[curLength-1].val,left:leafNodes[curLength-1].left.clone(),right:leafNodes[curLength-1].right.clone()});
            }

            while childrenNodes.len() > 0 {
            
                let leftNode = childrenNodes.remove(0);
                let leftNodeVal = (leftNode.val).as_ref();
                let rightNode = childrenNodes.remove(0);
                let rightNodeVal = (rightNode.val).as_ref();

                let parentNodeVal = <H256>::from(digest::digest(&digest::SHA256, &([&leftNodeVal[..], &rightNodeVal[..]].concat())));
                println!("parent: {:?}, left: {:?}, right:{:?}", parentNodeVal,leftNode.val,rightNode.val);
                let parentNode = TreeNode{val: parentNodeVal ,left:Some(Box::new(leftNode)),right:Some(Box::new(rightNode))};
            
                TreeNodes.push(parentNode);
                
            }
            childrenNodes = TreeNodes;
            curLength = childrenNodes.len();
            //println!("curLength:{}", curLength );
        }
        let root = childrenNodes.remove(0);
        //let leftSub = **(&root.left).as_ref().unwrap();
        //let rightSub = &root.right;//Some(Box::new(leftSub))
        return MerkleTree{ root:Some(Box::new(root)) ,length:length ,height:height };
    }
    

    pub fn root(&self) -> H256 {
        //unimplemented!()
        return (self.root.as_ref().unwrap()).val;
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        //unimplemented!()
        let mut curNode = self.root.as_ref().unwrap();
        let mut proofVec:Vec<H256> = Vec::new();
        let realIndex = (index + 1) as i32;
        let realHeight = self.height;
        println!("height: {:?}", realHeight);
        let mut center = (2_i32.pow(realHeight as u32))/2;
        let mut oriCenter = center;
        for i in 0..realHeight {
            //if curNode.is_none() { break; }
            let leftNode = curNode.left.as_ref();
            let rightNode = curNode.right.as_ref();
            if leftNode.is_none() || rightNode.is_none() { break; }
            //println!("index: {:?}, center: {:?}", realIndex,center);
            if realIndex > center {
                proofVec.push(leftNode.unwrap().val); // need left sibling
                curNode = rightNode.unwrap(); //move to the right
                center = center + oriCenter/2
            }
            else{
                proofVec.push(rightNode.unwrap().val); // need right sibling
                curNode = leftNode.unwrap(); //move to the left
                center = center - oriCenter/2;
            }
            oriCenter = oriCenter/2;
        }
        println!("{:?}", proofVec);
        return proofVec;

    }
}

pub fn getHeight(mut length:usize) -> usize {
        let mut height = 0;
        while length > 1 {
            height += 1;
            if length % 2 == 0 { //even number of nodes for this level
                length /= 2;
            }else { //odd number of nodes for this level
                length = ( length + 1 ) / 2;
            }
        }
        return height;
    }

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    //unimplemented!()
    let mut mydatum = datum.clone();
    let mut proofVec = Vec::from(proof.clone());
    let mut realIndex = index + 1;

    while proofVec.len() > 0 {
        if realIndex % 2 == 0 {
            let left_temp = proofVec.remove(proofVec.len()-1);
            let right = mydatum.as_ref();
            let left = left_temp.as_ref();
            let parentVal = <H256>::from(digest::digest(&digest::SHA256, &([&left[..], &right[..]].concat())));
            println!("verified parent: {:?}", parentVal);
            realIndex = realIndex/2;
            mydatum = parentVal;
        }
        else {
            let left = mydatum.as_ref();
            let right_temp = proofVec.remove(proofVec.len()-1);
            let right = right_temp.as_ref();
            let parentVal = <H256>::from(digest::digest(&digest::SHA256, &([&left[..], &right[..]].concat())));
            println!("verified parent: {:?}", parentVal);
            realIndex = (realIndex+1)/2;
            mydatum = parentVal;
        }
        
    }    
    return mydatum == *root;
}



#[cfg(test)]
mod tests {
    use crate::crypto::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }

    macro_rules! gen_merkle_tree_assignment2 {
        () => {{
            vec![
                (hex!("0000000000000000000000000000000000000000000000000000000000000011")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000022")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000033")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000044")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000055")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000066")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000077")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000088")).into(),
            ]
        }};
    }

    macro_rules! gen_merkle_tree_assignment2_another {
        () => {{
            vec![
                (hex!("1000000000000000000000000000000000000000000000000000000000000088")).into(),
                (hex!("2000000000000000000000000000000000000000000000000000000000000077")).into(),
                (hex!("3000000000000000000000000000000000000000000000000000000000000066")).into(),
                (hex!("4000000000000000000000000000000000000000000000000000000000000055")).into(),
                (hex!("5000000000000000000000000000000000000000000000000000000000000044")).into(),
                (hex!("6000000000000000000000000000000000000000000000000000000000000033")).into(),
                (hex!("7000000000000000000000000000000000000000000000000000000000000022")).into(),
                (hex!("8000000000000000000000000000000000000000000000000000000000000011")).into(),
            ]
        }};
    }

    #[test]
    fn assignment2_merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_assignment2!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6e18c8441bc8b0d1f0d4dc442c0d82ff2b4f38e2d7ca487c92e6db435d820a10")).into()
        );
    }

    #[test]
    fn assignment2_merkle_verify() {
        let input_data: Vec<H256> = gen_merkle_tree_assignment2!();
        let merkle_tree = MerkleTree::new(&input_data);
        for i in 0.. input_data.len() {
            let proof = merkle_tree.proof(i);
            println!("test index: {:?}", i);
            assert!(verify(&merkle_tree.root(), &input_data[i].hash(), &proof, i, input_data.len()));
        }
        let input_data_2: Vec<H256> = gen_merkle_tree_assignment2_another!();
        let merkle_tree_2 = MerkleTree::new(&input_data_2);
        assert!(!verify(&merkle_tree.root(), &input_data[0].hash(), &merkle_tree_2.proof(0), 0, input_data.len()));
    }

    #[test]
    fn assignment2_merkle_proof() {
        use std::collections::HashSet;
        let input_data: Vec<H256> = gen_merkle_tree_assignment2!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(5);
        let proof: HashSet<H256> = proof.into_iter().collect();
        let p: H256 = (hex!("c8c37c89fcc6ee7f5e8237d2b7ed8c17640c154f8d7751c774719b2b82040c76")).into();
        assert!(proof.contains(&p));
        let p: H256 = (hex!("bada70a695501195fb5ad950a5a41c02c0f9c449a918937267710a0425151b77")).into();
        assert!(proof.contains(&p));
        let p: H256 = (hex!("1e28fb71415f259bd4b0b3b98d67a1240b4f3bed5923aa222c5fdbd97c8fb002")).into();
        assert!(proof.contains(&p));
    }
}
