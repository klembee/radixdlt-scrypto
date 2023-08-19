// Re-export
pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

use sbor::rust::vec::Vec;

pub fn copy_buffer(buffer: Buffer) -> Vec<u8> {
    let len = buffer.len() as usize;
    let mut vec = Vec::<u8>::with_capacity(len);
    unsafe {
        buffer::consume_buffer(buffer.id(), vec.as_mut_ptr());
        vec.set_len(len);
    };
    vec
}

pub fn forget_vec(vec: Vec<u8>) -> Slice {
    let ptr = vec.as_ptr() as usize;
    let len = vec.len();
    assert!(ptr <= 0xffffffff && len <= 0xffffffff);

    // Note that the memory used by the Vec is forever leaked.
    // However, it's not an issue since the wasm instance will be destroyed after engine
    // consuming the data.
    sbor::rust::mem::forget(vec);

    Slice::new(ptr as u32, len as u32)
}

pub mod buffer {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        /// Consumes a buffer by copying the contents into the specified destination.
        pub fn consume_buffer(buffer_id: BufferId, destination_ptr: *mut u8);
    }
}

pub mod costing {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        pub fn execution_cost_unit_limit() -> u32;

        pub fn execution_cost_unit_price() -> Buffer;

        pub fn finalization_cost_unit_limit() -> u32;

        pub fn finalization_cost_unit_price() -> Buffer;

        pub fn usd_price() -> Buffer;

        pub fn tip_percentage() -> u32;

        pub fn fee_balance() -> Buffer;
    }
}

pub mod object {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        pub fn new_object(
            blueprint_ident_ptr: *const u8,
            blueprint_ident: usize,
            object_states_ptr: *const u8,
            object_states_len: usize,
        ) -> Buffer;

        pub fn drop_object(node_id_ptr: *const u8, node_id_len: usize);

        pub fn allocate_global_address(
            blueprint_id_ptr: *const u8,
            blueprint_id_len: usize,
        ) -> Buffer;

        pub fn globalize(
            modules_ptr: *const u8,
            modules_len: usize,
            address_ptr: *const u8,
            address_len: usize,
        ) -> Buffer;

        pub fn get_blueprint_id(component_id_ptr: *const u8, component_id_len: usize) -> Buffer;

        pub fn get_outer_object(component_id_ptr: *const u8, component_id_len: usize) -> Buffer;

        pub fn get_reservation_address(node_id_ptr: *const u8, node_id_len: usize) -> Buffer;
    }
}

pub mod kv_store {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        pub fn kv_store_new(schema_ptr: *const u8, schema_len: usize) -> Buffer;

        pub fn kv_store_open_entry(
            key_value_store_id_ptr: *const u8,
            key_value_store_id_len: usize,
            offset: *const u8,
            offset_len: usize,
            flags: u32,
        ) -> u32;

        pub fn kv_store_remove_entry(
            key_value_store_id_ptr: *const u8,
            key_value_store_id_len: usize,
            key: *const u8,
            key_len: usize,
        ) -> Buffer;
    }
}

pub mod kv_entry {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        pub fn kv_entry_get(key_value_entry_lock_handle: u32) -> Buffer;

        pub fn kv_entry_set(
            key_value_entry_lock_handle: u32,
            buffer_ptr: *const u8,
            buffer_len: usize,
        );

        pub fn kv_entry_remove(key_value_entry_lock_handle: u32) -> Buffer;

        pub fn kv_entry_close(key_value_entry_lock_handle: u32);
    }
}

pub mod invocation {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        pub fn call_method(
            receiver_ptr: *const u8,
            receive_len: usize,
            ident_ptr: *const u8,
            ident_len: usize,
            args_ptr: *const u8,
            args_len: usize,
        ) -> Buffer;

        pub fn call_module_method(
            _receiver_ptr: *const u8,
            _receive_len: usize,
            _module_id: u32,
            _ident_ptr: *const u8,
            _ident_len: usize,
            _args_ptr: *const u8,
            _args_len: usize,
        ) -> Buffer;

        pub fn call_direct_method(
            receiver_ptr: *const u8,
            receive_len: usize,
            ident_ptr: *const u8,
            ident_len: usize,
            args_ptr: *const u8,
            args_len: usize,
        ) -> Buffer;

        /// Invokes a function on a blueprint.
        pub fn call_function(
            package_address_ptr: *const u8,
            package_address_len: usize,
            blueprint_ident_ptr: *const u8,
            blueprint_ident_len: usize,
            function_ident_ptr: *const u8,
            function_ident_len: usize,
            args_ptr: *const u8,
            args_len: usize,
        ) -> Buffer;
    }
}

pub mod actor {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        pub fn actor_open_field(object_handle: u32, field: u32, flags: u32) -> u32;

        pub fn actor_call_module_method(
            module_id: u32,
            ident_ptr: *const u8,
            ident_len: usize,
            args_ptr: *const u8,
            args_len: usize,
        ) -> Buffer;

        pub fn get_node_id() -> Buffer;

        pub fn get_global_address() -> Buffer;

        pub fn get_blueprint() -> Buffer;

        pub fn get_auth_zone() -> Buffer;

        pub fn emit_event(
            event_name_ptr: *const u8,
            event_name_len: usize,
            event_data_ptr: *const u8,
            event_data_len: usize,
        );
    }
}

pub mod field_entry {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        // Reads a substate
        pub fn field_entry_read(handle: u32) -> Buffer;

        // Writes into a substate
        pub fn field_entry_write(handle: u32, data_ptr: *const u8, data_len: usize);

        // Releases a lock
        pub fn field_entry_close(handle: u32);
    }
}

pub mod system {
    pub use radix_engine_interface::types::{Buffer, BufferId, Slice};

    extern "C" {
        pub fn emit_log(
            level_ptr: *const u8,
            level_len: usize,
            message_ptr: *const u8,
            message_len: usize,
        );

        pub fn panic(message_ptr: *const u8, message_len: usize);

        pub fn get_transaction_hash() -> Buffer;

        pub fn generate_ruid() -> Buffer;
    }
}
