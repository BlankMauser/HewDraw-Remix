use smash::app::BattleObject;
use smash::phx::{Vector2f, Vector3f, Vector4f};

use super::VAR_MODULE_OFFSET;

const VAR_COUNT: usize = 0x200;
const VAR_INDEX_MASK: i32 = 0xFFF;

macro_rules! get_var_module {
    ($object:ident) => {{
        unsafe {
            let vtable = *($object as *mut *mut *mut u64);
            &mut *super::get_entry::<VarModule>(vtable, VAR_MODULE_OFFSET).expect("Did not find VarModule!")
        }
    }};
}

macro_rules! has_var_module {
    ($object:ident) => {{
        unsafe {
            if $object.is_null() {
                false
            } else {
                let vtable = *($object as *mut *mut *mut u64);
                super::is_hdr_object(vtable as _) && !super::get_entry::<VarModule>(vtable, VAR_MODULE_OFFSET).is_none()
            }
        }
    }};
}

macro_rules! require_var_module {
    ($object:ident) => {{
        if !has_var_module!($object) {
            panic!("BattleObject does not contain reference to VarModule!");
        }
        get_var_module!($object)
    }};
}

#[repr(C)]
pub struct VarModule {
    int: [Vec<i32>; 2],
    int64: [Vec<u64>; 2],
    float: [Vec<f32>; 2],
    flag: [Vec<bool>; 2],
}

/// An additional module to be used with Smash's `BattleObject` class. This handles storing and retrieving primitive variables
/// that you want to associate with a specific object (such as associating a gimmick timer with mario or dk)
impl VarModule {
    /// Resets all integers that are within the instance array.
    pub const RESET_INSTANCE_INT: u8 = 0b00000001;
    /// Resets all 64-bit values that are within the instance array
    pub const RESET_INSTANCE_INT64: u8 = 0b00000010;
    /// Resets all floats that are within the instance array
    pub const RESET_INSTANCE_FLOAT: u8 = 0b00000100;
    /// Resets all flags that are within the instance array (default is `false`)
    pub const RESET_INSTANCE_FLAG: u8 = 0b00001000;

    /// Resets all integers that are within the status array
    pub const RESET_STATUS_INT: u8 = 0b00010000;
    /// Resets all 64-bit values that are within the status array
    pub const RESET_STATUS_INT64: u8 = 0b00100000;
    /// Resets all floats that are within the status array
    pub const RESET_STATUS_FLOAT: u8 = 0b01000000;
    /// Resets all flags that are within the status array
    pub const RESET_STATUS_FLAG: u8 = 0b10000000;

    /// Resets all integers
    pub const RESET_INT: u8 = Self::RESET_INSTANCE_INT | Self::RESET_STATUS_INT;
    /// Resets all 64-bit values
    pub const RESET_INT64: u8 = Self::RESET_INSTANCE_INT64 | Self::RESET_STATUS_INT64;
    /// Resets all floats
    pub const RESET_FLOAT: u8 = Self::RESET_INSTANCE_FLOAT | Self::RESET_STATUS_FLOAT;
    /// Resets all flags
    pub const RESET_FLAG: u8 = Self::RESET_INSTANCE_FLAG | Self::RESET_STATUS_FLAG;

    /// Resets all values in the instance array
    pub const RESET_INSTANCE: u8 = 0xF;
    /// Resets all values in the status array
    pub const RESET_STATUS: u8 = 0xF0;
    /// Resets all values
    pub const RESET_ALL: u8 = 0xFF;

    /// Constructs a new instance of `VarModule` that defaults all values to either `0` or `false`
    /// # Returns
    /// A blank `VarModule` instance
    pub(crate) fn new() -> Self {
        Self {
            int: [Vec::new(), Vec::new()],
            int64: [Vec::new(), Vec::new()],
            float: [Vec::new(), Vec::new()],
            flag: [Vec::new(), Vec::new()],
        }
    }

    fn decode_var(what: i32) -> (usize, usize) {
        let vec_index = ((what >> 0xC) & 0x1) as usize;
        let index = (what & VAR_INDEX_MASK) as usize;
        assert!(index < VAR_COUNT, "VarModule index out of range: {:#x}", index);
        (vec_index, index)
    }

    fn get_value<T: Copy + Default>(values: &[Vec<T>; 2], what: i32) -> T {
        let (vec_index, index) = Self::decode_var(what);
        values[vec_index].get(index).copied().unwrap_or_default()
    }

    fn set_value<T: Clone + Default + PartialEq>(values: &mut [Vec<T>; 2], what: i32, val: T) {
        let (vec_index, index) = Self::decode_var(what);
        if values[vec_index].len() <= index {
            if val == T::default() {
                return;
            }
            values[vec_index].resize(index + 1, T::default());
        }
        values[vec_index][index] = val;
    }

    fn value_mut<T: Clone + Default>(values: &mut [Vec<T>; 2], what: i32) -> &mut T {
        let (vec_index, index) = Self::decode_var(what);
        if values[vec_index].len() <= index {
            values[vec_index].resize(index + 1, T::default());
        }
        &mut values[vec_index][index]
    }

    fn get_float_at(&self, vec_index: usize, index: usize) -> f32 {
        self.float[vec_index].get(index).copied().unwrap_or_default()
    }

    /// Checks if the object has `VarModule`
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    #[export_name = "VarModule__has_var_module"]
    pub extern "Rust" fn has_var_module(object: *mut BattleObject) -> bool {
        has_var_module!(object)
    }

    /// Resets various `VarModule` arrays depending on the mask
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `mask` - A mask of the reset values to determine what to reset
    #[export_name = "VarModule__reset"]
    pub extern "Rust" fn reset(object: *mut BattleObject, mask: u8) {
        let module = require_var_module!(object);
        if mask & Self::RESET_INSTANCE_INT != 0 {
            module.int[0].fill(0);
        }
        if mask & Self::RESET_STATUS_INT != 0 {
            module.int[1].fill(0);
        }
        if mask & Self::RESET_INSTANCE_INT64 != 0 {
            module.int64[0].fill(0);
        }
        if mask & Self::RESET_STATUS_INT64 != 0 {
            module.int64[1].fill(0);
        }
        if mask & Self::RESET_INSTANCE_FLOAT != 0 {
            module.float[0].fill(0.0);
        }
        if mask & Self::RESET_STATUS_FLOAT != 0 {
            module.float[1].fill(0.0);
        }
        if mask & Self::RESET_INSTANCE_FLAG != 0 {
            module.flag[0].fill(false);
        }
        if mask & Self::RESET_STATUS_FLAG != 0 {
            module.flag[1].fill(false);
        }
    }

    /// Retrieves an integer
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to retrieve
    /// # Returns
    /// The variable requested
    #[export_name = "VarModule__get_int"]
    pub extern "Rust" fn get_int(object: *mut BattleObject, what: i32) -> i32 {
        let module = require_var_module!(object);
        Self::get_value(&module.int, what)
    }

    /// Retrieves a float
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to retrieve
    /// # Returns
    /// The variable requested
    #[export_name = "VarModule__get_float"]
    pub extern "Rust" fn get_float(object: *mut BattleObject, what: i32) -> f32 {
        let module = require_var_module!(object);
        Self::get_value(&module.float, what)
    }

    /// Retrieves a 64-bit value
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to retrieve
    /// # Returns
    /// The variable requested
    #[export_name = "VarModule__get_int64"]
    pub extern "Rust" fn get_int64(object: *mut BattleObject, what: i32) -> u64 {
        let module = require_var_module!(object);
        Self::get_value(&module.int64, what)
    }

    /// Retrieves a flag
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to retrieve
    /// # Returns
    /// The variable requested
    #[export_name = "VarModule__is_flag"]
    pub extern "Rust" fn is_flag(object: *mut BattleObject, what: i32) -> bool {
        let module = require_var_module!(object);
        Self::get_value(&module.flag, what)
    }

    /// Sets an integer value
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to set
    /// * `val` - The value to set the variable to
    #[export_name = "VarModule__set_int"]
    pub extern "Rust" fn set_int(object: *mut BattleObject, what: i32, val: i32) {
        let module = require_var_module!(object);
        Self::set_value(&mut module.int, what, val);
    }

    /// Sets a float value
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to set
    /// * `val` - The value to set the variable to
    #[export_name = "VarModule__set_float"]
    pub extern "Rust" fn set_float(object: *mut BattleObject, what: i32, val: f32) {
        let module = require_var_module!(object);
        Self::set_value(&mut module.float, what, val);
    }

    /// Sets a 64-bit value
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to set
    /// * `val` - The value to set the variable to
    #[export_name = "VarModule__set_int64"]
    pub extern "Rust" fn set_int64(object: *mut BattleObject, what: i32, val: u64) {
        let module = require_var_module!(object);
        Self::set_value(&mut module.int64, what, val);
    }

    /// Sets a flag
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to set
    /// * `val` - The value to set the variable to
    #[export_name = "VarModule__set_flag"]
    pub extern "Rust" fn set_flag(object: *mut BattleObject, what: i32, val: bool) {
        let module = require_var_module!(object);
        Self::set_value(&mut module.flag, what, val);
    }

    /// Sets a flag to false
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to set
    /// # Note
    /// This method is equivalent to `VarModule::set_flag(object, what, false)`
    #[export_name = "VarModule__off_flag"]
    pub extern "Rust" fn off_flag(object: *mut BattleObject, what: i32) {
        Self::set_flag(object, what, false);
    }

    /// Sets a flag to true
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to set
    /// # Note
    /// This method is equivalent to `VarModule::set_flag(object, what, true)`
    #[export_name = "VarModule__on_flag"]
    pub extern "Rust" fn on_flag(object: *mut BattleObject, what: i32) {
        Self::set_flag(object, what, true);
    }

    /// Countdowns an integer
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to count down
    /// * `min` - The minimum value that variable should be before we are done counting down
    /// # Returns
    /// * `true` - `what` was less than `min` before or after decrementing
    /// * `false` - `what` remains greater than or equal to `min` after decrementing
    #[export_name = "VarModule__countdown_int"]
    pub extern "Rust" fn countdown_int(object: *mut BattleObject, what: i32, min: i32) -> bool {
        if Self::get_int(object, what) > min {
            Self::dec_int(object, what);
            Self::get_int(object, what) == min
        } else {
            false
        }
    }

    /// Adds a value to an integer
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to add to
    /// * `amount` - The value to add to the variable
    #[export_name = "VarModule__add_int"]
    pub extern "Rust" fn add_int(object: *mut BattleObject, what: i32, amount: i32) {
        let module = require_var_module!(object);
        *Self::value_mut(&mut module.int, what) += amount;
    }

    /// Subtracts a value from an integer
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to subtract from
    /// * `amount` - The value to subtract from the variable
    #[export_name = "VarModule__sub_int"]
    pub extern "Rust" fn sub_int(object: *mut BattleObject, what: i32, amount: i32) {
        let module = require_var_module!(object);
        *Self::value_mut(&mut module.int, what) -= amount;
    }

    /// Increments an integer
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to increment
    /// # Note
    /// This is functionally equivalent to `VarModule::add_int(object, what, 1)`
    #[export_name = "VarModule__inc_int"]
    pub extern "Rust" fn inc_int(object: *mut BattleObject, what: i32) {
        Self::add_int(object, what, 1);
    }

    /// Decrements an integer
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to decrement
    /// # Note
    /// This is functionally equivalent to `VarModule::sub_int(object, what, 1)`
    #[export_name = "VarModule__dec_int"]
    pub extern "Rust" fn dec_int(object: *mut BattleObject, what: i32) {
        Self::sub_int(object, what, 1);
    }

    /// Adds a value to float
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to add on to
    /// * `amount` - The amount to add to the variable
    #[export_name = "VarModule__add_float"]
    pub extern "Rust" fn add_float(object: *mut BattleObject, what: i32, amount: f32) {
        let module = require_var_module!(object);
        *Self::value_mut(&mut module.float, what) += amount;
    }

    /// Subtracts a value from a float
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - The variable to subtract from
    /// * `amount` - The amount to subtract from the variable
    #[export_name = "VarModule__sub_float"]
    pub extern "Rust" fn sub_float(object: *mut BattleObject, what: i32, amount: f32) {
        let module = require_var_module!(object);
        *Self::value_mut(&mut module.float, what) -= amount;
    }

    /// Sets a 2-dimensional vector
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - Where to start setting the vector
    /// * `val` - The vector to set
    /// # Panics
    /// This function requires that the last 3 bytes of `what` are less than `0xFFF`
    #[export_name = "VarModule__set_vec2"]
    pub extern "Rust" fn set_vec2(object: *mut BattleObject, what: i32, val: Vector2f) {
        let module = require_var_module!(object);
        let (vec_index, index) = Self::decode_var(what);
        if index + 2 > VAR_COUNT {
            panic!("Cannot set Vec2 on index that will overflow!");
        }
        if module.float[vec_index].len() < index + 2 {
            module.float[vec_index].resize(index + 2, 0.0);
        }
        module.float[vec_index][index + 0] = val.x;
        module.float[vec_index][index + 1] = val.y;
    }

    /// Sets a 3-dimensional vector
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - Where to start setting the vector
    /// * `val` - The vector to set
    /// # Panics
    /// This function requires that the last 3 bytes of `what` are less than `0xFFE`
    #[export_name = "VarModule__set_vec3"]
    pub extern "Rust" fn set_vec3(object: *mut BattleObject, what: i32, val: Vector3f) {
        let module = require_var_module!(object);
        let (vec_index, index) = Self::decode_var(what);
        if index + 3 > VAR_COUNT {
            panic!("Cannot set Vec2 on index that will overflow!");
        }
        if module.float[vec_index].len() < index + 3 {
            module.float[vec_index].resize(index + 3, 0.0);
        }
        module.float[vec_index][index + 0] = val.x;
        module.float[vec_index][index + 1] = val.y;
        module.float[vec_index][index + 2] = val.z;
    }

    /// Sets a 4-dimensional vector
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - Where to start setting the vector
    /// * `val` - The vector to set
    /// # Panics
    /// This function requires that the last 3 bytes of `what` are less than `0xFFD`
    #[export_name = "VarModule__set_vec4"]
    pub extern "Rust" fn set_vec4(object: *mut BattleObject, what: i32, val: Vector4f) {
        let module = require_var_module!(object);
        let (vec_index, index) = Self::decode_var(what);
        if index + 4 > VAR_COUNT {
            panic!("Cannot set Vec2 on index that will overflow!");
        }
        if module.float[vec_index].len() < index + 4 {
            module.float[vec_index].resize(index + 4, 0.0);
        }
        module.float[vec_index][index + 0] = val.x;
        module.float[vec_index][index + 1] = val.y;
        module.float[vec_index][index + 2] = val.z;
        module.float[vec_index][index + 3] = val.w;
    }

    /// Gets a 2-dimensional vector
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - Where to start setting the vector
    /// # Returns
    /// The 2-dimensional vector starting at the value specified
    /// # Panics
    /// This function requires that the last 3 bytes of `what` are less than `0xFFF`
    #[export_name = "VarModule__get_vec2"]
    pub extern "Rust" fn get_vec2(object: *mut BattleObject, what: i32) -> Vector2f {
        let module = require_var_module!(object);
        let (vec_index, index) = Self::decode_var(what);
        if index + 2 > VAR_COUNT {
            panic!("Cannot get Vec2 with index that will overflow!");
        }
        Vector2f {
            x: module.get_float_at(vec_index, index + 0),
            y: module.get_float_at(vec_index, index + 1),
        }
    }

    /// Gets a 3-dimensional vector
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - Where to start setting the vector
    /// # Returns
    /// The 3-dimensional vector starting at the value specified
    /// # Panics
    /// This function requires that the last 3 bytes of `what` are less than `0xFFE`
    #[export_name = "VarModule__get_vec3"]
    pub extern "Rust" fn get_vec3(object: *mut BattleObject, what: i32) -> Vector3f {
        let module = require_var_module!(object);
        let (vec_index, index) = Self::decode_var(what);
        if index + 3 > VAR_COUNT {
            panic!("Cannot get Vec2 with index that will overflow!");
        }
        Vector3f {
            x: module.get_float_at(vec_index, index + 0),
            y: module.get_float_at(vec_index, index + 1),
            z: module.get_float_at(vec_index, index + 2),
        }
    }

    /// Gets a 4-dimensional vector
    /// # Arguments
    /// * `object` - The owning `BattleObject` instance
    /// * `what` - Where to start setting the vector
    /// # Returns
    /// The 4-dimensional vector starting at the value specified
    /// # Panics
    /// This function requires that the last 3 bytes of `what` are less than `0xFFD`
    #[export_name = "VarModule__get_vec4"]
    pub extern "Rust" fn get_vec4(object: *mut BattleObject, what: i32) -> Vector4f {
        let module = require_var_module!(object);
        let (vec_index, index) = Self::decode_var(what);
        if index + 4 > VAR_COUNT {
            panic!("Cannot get Vec2 with index that will overflow!");
        }
        Vector4f {
            x: module.get_float_at(vec_index, index + 0),
            y: module.get_float_at(vec_index, index + 1),
            z: module.get_float_at(vec_index, index + 2),
            w: module.get_float_at(vec_index, index + 3),
        }
    }
}
