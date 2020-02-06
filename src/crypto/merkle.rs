use super::hash::{Hashable, H256};
use ring::{digest};

/// A Merkle tree.
//Use Option<Box<>> for Tree implementation
//Tree implementation reference: https://gist.github.com/aidanhs/5ac9088ca0f6bdd4a370
#[derive(Debug, Default)]
pub struct TreeNode {
    val: H256,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

#[derive(Debug, Default)]
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
        let mut length = data.len();
        let mut height = getHeight(length);

        if length == 0 { return MerkleTree{ root:None ,length:length , height:height}; }
        
        println!("height:{}, length:{}", height, length );

        let mut leafNodes = Vec::with_capacity(length);
        //insert leaf nodes fisrt
        for elem in data{
            leafNodes.push(TreeNode{val:elem.hash(),left:None,right:None});
        }
        if length % 2 == 1 {
            leafNodes.push(TreeNode{val:leafNodes[length-1].val,left:None,right:None});
        }
        //build tree
        let mut childrenNodes = leafNodes;
        let mut curLength = childrenNodes.len();
        while curLength > 1 {
            
            let mut TreeNodes:Vec<TreeNode> = Vec::new();
            while childrenNodes.len() > 0 {
                if childrenNodes.len() == 1 {
                    let tempNode = childrenNodes.remove(0);
                    TreeNodes.push(tempNode);
                }
                let leftNode = childrenNodes.remove(0);
                let leftNodeVal = (leftNode.val).as_ref();
                let rightNode = childrenNodes.remove(0);
                let rightNodeVal = (rightNode.val).as_ref();

                let parentNodeVal = <H256>::from(digest::digest(&digest::SHA256, &([&leftNodeVal[..], &rightNodeVal[..]].concat())));
                let parentNode = TreeNode{val: parentNodeVal ,left:Some(Box::new(leftNode)),right:Some(Box::new(rightNode))};
                
                TreeNodes.push(parentNode);
                println!("{:?}",childrenNodes.len() );

            }
            childrenNodes = TreeNodes;
            curLength = childrenNodes.len();
            println!("curLength:{}", curLength );
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
        let mut proofVector:Vec<H256> = Vec::new();
        let mut realIndex = (index + 1) as i32;
        let mut realHeight = self.height + 1;
        let mut center = (2_i32.pow(realHeight as u32))/2;
        for i in 1..realHeight {
            //if curNode.is_none() { break; }
            let leftNode = curNode.left.as_ref();
            let rightNode = curNode.right.as_ref();
            if leftNode.is_none() || rightNode.is_none() { break; }
            if realIndex > center {
                proofVector.push(leftNode.unwrap().val); // need left sibling
                curNode = rightNode.unwrap(); //move to the right
                center = center + center/2;
            }
            else{
                proofVector.push(rightNode.unwrap().val); // need right sibling
                curNode = leftNode.unwrap(); //move to the left
                center = center - center/2;
            }
        }
        return proofVector;

    }
}

pub fn getHeight(mut length:usize) -> usize {
        let mut height = 0;
        while length > 1 {
            height += 1;
            if length % 2 == 0 { //even number of nodes for this level
                length /= 2;
            }else { //old number of nodes for this level
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
    let mut proofVector = Vec::from(proof.clone());
    let mut realIndex = index + 1;

    while proofVector.len() > 0 {
        if realIndex % 2 == 0 {
            let left_temp = proofVector.remove(proofVector.len()-1);
            let right = mydatum.as_ref();
            let left = left_temp.as_ref();
            let parentVal = <H256>::from(digest::digest(&digest::SHA256, &([&left[..], &right[..]].concat())));
            realIndex = realIndex/2;
            mydatum = parentVal;
        }
        else {
            let left = mydatum.as_ref();
            let right_temp = proofVector.remove(proofVector.len()-1);
            let right = right_temp.as_ref();
            let parentVal = <H256>::from(digest::digest(&digest::SHA256, &([&left[..], &right[..]].concat())));
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
}
