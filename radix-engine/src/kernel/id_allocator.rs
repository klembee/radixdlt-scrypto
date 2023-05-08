use crate::errors::{IdAllocationError, KernelError, RuntimeError};
use crate::types::*;

/// An ID allocator defines how identities are generated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdAllocator {
    /// A system transaction (eg genesis) may define static addresses available for use
    pre_allocated_ids: BTreeSet<NodeId>,
    /// Each frame may allocate addresses, but they can only be used by that frame, and
    /// have to be attached to a node before the frame completes
    frame_allocated_ids: Vec<BTreeSet<NodeId>>,
    transaction_hash: Hash,
    next_id: u32,
}

impl IdAllocator {
    pub fn new(transaction_hash: Hash, pre_allocated_ids: BTreeSet<NodeId>) -> Self {
        Self {
            pre_allocated_ids,
            frame_allocated_ids: vec![BTreeSet::new()],
            transaction_hash,
            next_id: 0u32,
        }
    }

    /// Called on a new frame.
    pub fn push(&mut self) {
        self.frame_allocated_ids.push(BTreeSet::new());
    }

    /// Called when the frame is over.
    /// Ensures all allocated ids have been used.
    pub fn pop(&mut self) -> Result<(), RuntimeError> {
        let ids = self.frame_allocated_ids.pop().expect("No frame found");
        if !ids.is_empty() {
            return Err(RuntimeError::KernelError(KernelError::IdAllocationError(
                IdAllocationError::AllocatedIDsNotEmpty(ids),
            )));
        }
        Ok(())
    }

    /// Called when the transaction is over.
    /// Ensures all transaction-scoped allocated ids have been used.
    pub fn on_teardown(&mut self) -> Result<(), RuntimeError> {
        if !self.pre_allocated_ids.is_empty() {
            return Err(RuntimeError::KernelError(KernelError::IdAllocationError(
                IdAllocationError::AllocatedIDsNotEmpty(self.pre_allocated_ids.clone()),
            )));
        }
        Ok(())
    }

    /// Called when a node is created - the node id is specified at creation time,
    /// this method ensures the id is already allocated.
    pub fn take_node_id(&mut self, node_id: NodeId) -> Result<(), RuntimeError> {
        let ids = self.frame_allocated_ids.last_mut().expect("No frame found");
        let frame_allocated = ids.remove(&node_id);
        let pre_allocated = self.pre_allocated_ids.remove(&node_id);
        if !frame_allocated && !pre_allocated {
            return Err(RuntimeError::KernelError(KernelError::IdAllocationError(
                IdAllocationError::NodeIdWasNotAllocated(node_id),
            )));
        }
        Ok(())
    }

    // Protected, only virtual manager should call this
    // TODO: Clean up interface
    pub fn allocate_virtual_node_id(&mut self, node_id: NodeId) {
        let ids = self
            .frame_allocated_ids
            .last_mut()
            .expect("No frame found.");
        ids.insert(node_id);
    }

    /// Called before a node is created to allocate an address.
    /// This should only be used when not using a pre-allocated address.
    pub fn allocate_node_id(&mut self, entity_type: EntityType) -> Result<NodeId, RuntimeError> {
        let node_id = self
            .next_node_id(entity_type)
            .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;

        let ids = self
            .frame_allocated_ids
            .last_mut()
            .expect("No frame found.");
        ids.insert(node_id);

        Ok(node_id)
    }

    fn next(&mut self) -> Result<u32, IdAllocationError> {
        if self.next_id == u32::MAX {
            Err(IdAllocationError::OutOfID)
        } else {
            let rtn = self.next_id;
            self.next_id += 1;
            Ok(rtn)
        }
    }

    fn next_node_id(&mut self, entity_type: EntityType) -> Result<NodeId, IdAllocationError> {
        // Compute `hash(transaction_hash, index)`
        let mut buf = [0u8; Hash::LENGTH + 4];
        buf[..Hash::LENGTH].copy_from_slice(self.transaction_hash.as_ref());
        buf[Hash::LENGTH..].copy_from_slice(&self.next()?.to_le_bytes());
        let hash = hash(buf);

        // Install the entity type
        let mut node_id: [u8; NodeId::LENGTH] = hash.lower_bytes();
        node_id[0] = entity_type as u8;

        Ok(NodeId(node_id))
    }
}
