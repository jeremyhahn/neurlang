//! Collection data structures for Neurlang stdlib
//!
//! These functions implement stack, queue, and other data structures
//! using memory buffers.

// ============================================================================
// Stack Operations
// ============================================================================
// Stack layout in memory:
// [capacity: u64][size: u64][data: u64 * capacity]

/// Initialize a stack at the given memory location.
///
/// # Safety
/// Assumes ptr has at least (2 + capacity) * 8 bytes available.
///
/// # Neurlang Export
/// - Category: collections/stack
/// - Difficulty: 1
///
/// # Prompts
/// - initialize a stack at {ptr} with capacity {capacity}
/// - create a new stack with {capacity} slots at memory address {ptr}
/// - set up empty stack structure at {ptr} holding up to {capacity} elements
/// - allocate stack of size {capacity} at location {ptr}
/// - init stack buffer at {ptr} with max size {capacity}
/// - prepare stack data structure at {ptr} for {capacity} items
/// - create LIFO stack at {ptr} with {capacity} capacity
/// - initialize empty stack at memory {ptr} with room for {capacity} values
/// - set up stack at address {ptr} supporting {capacity} entries
/// - construct stack at {ptr} with maximum capacity {capacity}
/// - make a new stack at {ptr} that can hold {capacity} elements
/// - initialize LIFO data structure at {ptr} with size {capacity}
/// - create stack buffer at {ptr} with {capacity} element capacity
///
/// # Parameters
/// - ptr=r0 "Memory address where stack will be initialized"
/// - capacity=r1 "Maximum number of elements the stack can hold"
#[inline(never)]
pub unsafe fn stack_init(ptr: *mut u64, capacity: u64) {
    *ptr = capacity;         // Store capacity
    *ptr.add(1) = 0;         // Initialize size to 0
}

/// Push a value onto the stack.
///
/// # Safety
/// Assumes ptr points to a valid initialized stack.
///
/// # Neurlang Export
/// - Category: collections/stack
/// - Difficulty: 1
///
/// Returns: 1 on success, 0 if stack is full
///
/// # Prompts
/// - push {value} onto stack at {ptr}
/// - add {value} to top of stack {ptr}
/// - put {value} on stack at address {ptr}
/// - push value {value} to stack {ptr}
/// - add element {value} to stack at {ptr}
/// - insert {value} at top of stack {ptr}
/// - stack push {value} to {ptr}
/// - append {value} to stack at memory {ptr}
/// - place {value} on top of stack {ptr}
/// - push item {value} onto LIFO stack at {ptr}
/// - add {value} to the stack located at {ptr}
/// - store {value} on stack {ptr}
/// - push {value} to stack structure at {ptr}
///
/// # Parameters
/// - ptr=r0 "Memory address of the stack"
/// - value=r1 "Value to push onto the stack"
#[inline(never)]
pub unsafe fn stack_push(ptr: *mut u64, value: u64) -> u64 {
    let capacity = *ptr;
    let size = *ptr.add(1);

    if size >= capacity {
        return 0; // Stack full
    }

    // Store value at data[size]
    *ptr.add(2 + size as usize) = value;
    // Increment size
    *ptr.add(1) = size + 1;

    1
}

/// Pop a value from the stack.
///
/// # Safety
/// Assumes ptr points to a valid initialized stack.
///
/// # Neurlang Export
/// - Category: collections/stack
/// - Difficulty: 1
///
/// Returns: the popped value, or 0 if stack is empty
///
/// # Prompts
/// - pop value from stack at {ptr}
/// - remove top element from stack {ptr}
/// - pop from stack at address {ptr}
/// - get and remove top of stack {ptr}
/// - stack pop from {ptr}
/// - remove top item from LIFO stack {ptr}
/// - pop element off stack at {ptr}
/// - take value from top of stack {ptr}
/// - extract top element from stack at {ptr}
/// - pop the stack located at {ptr}
/// - remove and return top of stack {ptr}
/// - get value from top of stack {ptr} and remove it
/// - pop item from stack structure at {ptr}
///
/// # Parameters
/// - ptr=r0 "Memory address of the stack"
#[inline(never)]
pub unsafe fn stack_pop(ptr: *mut u64) -> u64 {
    let size = *ptr.add(1);

    if size == 0 {
        return 0; // Stack empty
    }

    let new_size = size - 1;
    let value = *ptr.add(2 + new_size as usize);
    *ptr.add(1) = new_size;

    value
}

/// Peek at the top of the stack without popping.
///
/// # Safety
/// Assumes ptr points to a valid initialized stack.
///
/// # Neurlang Export
/// - Category: collections/stack
/// - Difficulty: 1
///
/// # Prompts
/// - peek at top of stack {ptr}
/// - get top value from stack {ptr} without removing
/// - look at top element of stack at {ptr}
/// - read top of stack {ptr}
/// - peek stack at address {ptr}
/// - view top item on stack {ptr}
/// - get stack top from {ptr} without popping
/// - examine top of stack at {ptr}
/// - check what is on top of stack {ptr}
/// - peek at stack {ptr} top element
/// - read top value from LIFO stack {ptr}
/// - inspect top of stack at memory {ptr}
/// - see top element of stack {ptr} without removal
///
/// # Parameters
/// - ptr=r0 "Memory address of the stack"
#[inline(never)]
pub unsafe fn stack_peek(ptr: *const u64) -> u64 {
    let size = *ptr.add(1);

    if size == 0 {
        return 0; // Stack empty
    }

    *ptr.add(2 + (size - 1) as usize)
}

/// Get the current size of the stack.
///
/// # Neurlang Export
/// - Category: collections/stack
/// - Difficulty: 1
///
/// # Prompts
/// - get size of stack at {ptr}
/// - how many elements in stack {ptr}
/// - return stack size for {ptr}
/// - count elements in stack at {ptr}
/// - get number of items on stack {ptr}
/// - stack length at address {ptr}
/// - find stack size at {ptr}
/// - get element count for stack {ptr}
/// - how full is stack at {ptr}
/// - return number of elements in stack {ptr}
/// - query stack size at memory {ptr}
/// - get current size of LIFO stack {ptr}
/// - count items on stack at {ptr}
///
/// # Parameters
/// - ptr=r0 "Memory address of the stack"
#[inline(never)]
pub unsafe fn stack_size(ptr: *const u64) -> u64 {
    *ptr.add(1)
}

/// Check if stack is empty.
///
/// # Neurlang Export
/// - Category: collections/stack
/// - Difficulty: 1
///
/// # Prompts
/// - check if stack at {ptr} is empty
/// - is stack {ptr} empty
/// - test if stack at {ptr} has no elements
/// - determine if stack {ptr} is empty
/// - check stack {ptr} for emptiness
/// - is LIFO stack at {ptr} empty
/// - verify stack at address {ptr} is empty
/// - see if stack {ptr} contains no items
/// - check for empty stack at {ptr}
/// - test stack {ptr} emptiness
/// - is stack at memory {ptr} empty
/// - check if stack {ptr} has zero elements
/// - query if stack at {ptr} is empty
///
/// # Parameters
/// - ptr=r0 "Memory address of the stack"
#[inline(never)]
pub unsafe fn stack_is_empty(ptr: *const u64) -> u64 {
    if *ptr.add(1) == 0 { 1 } else { 0 }
}

// ============================================================================
// Ring Buffer / Queue Operations
// ============================================================================
// Queue layout in memory:
// [capacity: u64][head: u64][tail: u64][count: u64][data: u64 * capacity]

/// Initialize a queue at the given memory location.
///
/// # Safety
/// Assumes ptr has at least (4 + capacity) * 8 bytes available.
///
/// # Neurlang Export
/// - Category: collections/queue
/// - Difficulty: 1
///
/// # Prompts
/// - initialize a queue at {ptr} with capacity {capacity}
/// - create a new queue with {capacity} slots at memory address {ptr}
/// - set up empty queue structure at {ptr} holding up to {capacity} elements
/// - allocate queue of size {capacity} at location {ptr}
/// - init queue buffer at {ptr} with max size {capacity}
/// - prepare queue data structure at {ptr} for {capacity} items
/// - create FIFO queue at {ptr} with {capacity} capacity
/// - initialize empty queue at memory {ptr} with room for {capacity} values
/// - set up ring buffer queue at address {ptr} supporting {capacity} entries
/// - construct queue at {ptr} with maximum capacity {capacity}
/// - make a new queue at {ptr} that can hold {capacity} elements
/// - initialize FIFO data structure at {ptr} with size {capacity}
/// - create circular queue buffer at {ptr} with {capacity} element capacity
///
/// # Parameters
/// - ptr=r0 "Memory address where queue will be initialized"
/// - capacity=r1 "Maximum number of elements the queue can hold"
#[inline(never)]
pub unsafe fn queue_init(ptr: *mut u64, capacity: u64) {
    *ptr = capacity;         // capacity
    *ptr.add(1) = 0;         // head
    *ptr.add(2) = 0;         // tail
    *ptr.add(3) = 0;         // count
}

/// Enqueue a value.
///
/// # Safety
/// Assumes ptr points to a valid initialized queue.
///
/// # Neurlang Export
/// - Category: collections/queue
/// - Difficulty: 2
///
/// Returns: 1 on success, 0 if queue is full
///
/// # Prompts
/// - enqueue {value} to queue at {ptr}
/// - add {value} to back of queue {ptr}
/// - put {value} in queue at address {ptr}
/// - enqueue value {value} to queue {ptr}
/// - add element {value} to queue at {ptr}
/// - insert {value} at end of queue {ptr}
/// - queue enqueue {value} to {ptr}
/// - append {value} to queue at memory {ptr}
/// - place {value} at back of queue {ptr}
/// - add item {value} to FIFO queue at {ptr}
/// - add {value} to the queue located at {ptr}
/// - store {value} in queue {ptr}
/// - enqueue {value} to queue structure at {ptr}
/// - push {value} to end of queue {ptr}
///
/// # Parameters
/// - ptr=r0 "Memory address of the queue"
/// - value=r1 "Value to enqueue"
#[inline(never)]
pub unsafe fn queue_enqueue(ptr: *mut u64, value: u64) -> u64 {
    let capacity = *ptr;
    let tail = *ptr.add(2);
    let count = *ptr.add(3);

    if count >= capacity {
        return 0; // Queue full
    }

    // Store value at data[tail]
    *ptr.add(4 + tail as usize) = value;

    // Update tail (wrap around)
    let new_tail = if tail + 1 >= capacity { 0 } else { tail + 1 };
    *ptr.add(2) = new_tail;
    *ptr.add(3) = count + 1;

    1
}

/// Dequeue a value.
///
/// # Safety
/// Assumes ptr points to a valid initialized queue.
///
/// # Neurlang Export
/// - Category: collections/queue
/// - Difficulty: 2
///
/// Returns: the dequeued value, or 0 if queue is empty
///
/// # Prompts
/// - dequeue value from queue at {ptr}
/// - remove front element from queue {ptr}
/// - dequeue from queue at address {ptr}
/// - get and remove front of queue {ptr}
/// - queue dequeue from {ptr}
/// - remove front item from FIFO queue {ptr}
/// - dequeue element from queue at {ptr}
/// - take value from front of queue {ptr}
/// - extract front element from queue at {ptr}
/// - pop the queue located at {ptr}
/// - remove and return front of queue {ptr}
/// - get value from front of queue {ptr} and remove it
/// - dequeue item from queue structure at {ptr}
///
/// # Parameters
/// - ptr=r0 "Memory address of the queue"
#[inline(never)]
pub unsafe fn queue_dequeue(ptr: *mut u64) -> u64 {
    let capacity = *ptr;
    let head = *ptr.add(1);
    let count = *ptr.add(3);

    if count == 0 {
        return 0; // Queue empty
    }

    // Get value at data[head]
    let value = *ptr.add(4 + head as usize);

    // Update head (wrap around)
    let new_head = if head + 1 >= capacity { 0 } else { head + 1 };
    *ptr.add(1) = new_head;
    *ptr.add(3) = count - 1;

    value
}

/// Peek at the front of the queue without dequeuing.
///
/// # Neurlang Export
/// - Category: collections/queue
/// - Difficulty: 1
///
/// # Prompts
/// - peek at front of queue {ptr}
/// - get front value from queue {ptr} without removing
/// - look at front element of queue at {ptr}
/// - read front of queue {ptr}
/// - peek queue at address {ptr}
/// - view front item in queue {ptr}
/// - get queue front from {ptr} without dequeuing
/// - examine front of queue at {ptr}
/// - check what is at front of queue {ptr}
/// - peek at queue {ptr} front element
/// - read front value from FIFO queue {ptr}
/// - inspect front of queue at memory {ptr}
/// - see front element of queue {ptr} without removal
///
/// # Parameters
/// - ptr=r0 "Memory address of the queue"
#[inline(never)]
pub unsafe fn queue_peek(ptr: *const u64) -> u64 {
    let head = *ptr.add(1);
    let count = *ptr.add(3);

    if count == 0 {
        return 0;
    }

    *ptr.add(4 + head as usize)
}

/// Get the current size of the queue.
///
/// # Neurlang Export
/// - Category: collections/queue
/// - Difficulty: 1
///
/// # Prompts
/// - get size of queue at {ptr}
/// - how many elements in queue {ptr}
/// - return queue size for {ptr}
/// - count elements in queue at {ptr}
/// - get number of items in queue {ptr}
/// - queue length at address {ptr}
/// - find queue size at {ptr}
/// - get element count for queue {ptr}
/// - how full is queue at {ptr}
/// - return number of elements in queue {ptr}
/// - query queue size at memory {ptr}
/// - get current size of FIFO queue {ptr}
/// - count items in queue at {ptr}
///
/// # Parameters
/// - ptr=r0 "Memory address of the queue"
#[inline(never)]
pub unsafe fn queue_size(ptr: *const u64) -> u64 {
    *ptr.add(3)
}

/// Check if queue is empty.
///
/// # Neurlang Export
/// - Category: collections/queue
/// - Difficulty: 1
///
/// # Prompts
/// - check if queue at {ptr} is empty
/// - is queue {ptr} empty
/// - test if queue at {ptr} has no elements
/// - determine if queue {ptr} is empty
/// - check queue {ptr} for emptiness
/// - is FIFO queue at {ptr} empty
/// - verify queue at address {ptr} is empty
/// - see if queue {ptr} contains no items
/// - check for empty queue at {ptr}
/// - test queue {ptr} emptiness
/// - is queue at memory {ptr} empty
/// - check if queue {ptr} has zero elements
/// - query if queue at {ptr} is empty
///
/// # Parameters
/// - ptr=r0 "Memory address of the queue"
#[inline(never)]
pub unsafe fn queue_is_empty(ptr: *const u64) -> u64 {
    if *ptr.add(3) == 0 { 1 } else { 0 }
}

// ============================================================================
// Simple Hash Table Operations (Open Addressing)
// ============================================================================
// Hash table layout:
// [capacity: u64][count: u64][keys: u64 * capacity][values: u64 * capacity]
// Uses 0 as empty key marker (so 0 cannot be a valid key)

// Hash function is inlined directly in each function to avoid cross-function calls

/// Initialize a hash table.
///
/// # Safety
/// Assumes ptr has at least (2 + 2*capacity) * 8 bytes available.
///
/// # Neurlang Export
/// - Category: collections/hashtable
/// - Difficulty: 2
///
/// # Prompts
/// - initialize a hash table at {ptr} with capacity {capacity}
/// - create a new hashtable with {capacity} buckets at memory address {ptr}
/// - set up empty hash table structure at {ptr} holding up to {capacity} entries
/// - allocate hash table of size {capacity} at location {ptr}
/// - init hashtable buffer at {ptr} with max size {capacity}
/// - prepare hash table data structure at {ptr} for {capacity} items
/// - create hash map at {ptr} with {capacity} capacity
/// - initialize empty hashtable at memory {ptr} with room for {capacity} key-value pairs
/// - set up hash table at address {ptr} supporting {capacity} entries
/// - construct hashtable at {ptr} with maximum capacity {capacity}
/// - make a new hash table at {ptr} that can hold {capacity} elements
/// - initialize dictionary at {ptr} with size {capacity}
/// - create key-value store at {ptr} with {capacity} bucket capacity
///
/// # Parameters
/// - ptr=r0 "Memory address where hash table will be initialized"
/// - capacity=r1 "Maximum number of key-value pairs the hash table can hold"
#[inline(never)]
pub unsafe fn hashtable_init(ptr: *mut u64, capacity: u64) {
    *ptr = capacity;
    *ptr.add(1) = 0; // count

    // Clear all keys to 0 (empty marker)
    let mut i: u64 = 0;
    while i < capacity {
        *ptr.add(2 + i as usize) = 0;
        i = i + 1;
    }
}

/// Insert or update a key-value pair.
///
/// # Safety
/// Assumes ptr points to a valid initialized hash table.
/// Key must not be 0 (reserved as empty marker).
///
/// # Neurlang Export
/// - Category: collections/hashtable
/// - Difficulty: 3
///
/// Returns: 1 on success, 0 if table is full
///
/// # Prompts
/// - put {key} with value {value} in hash table at {ptr}
/// - insert key {key} value {value} into hashtable {ptr}
/// - add entry {key}={value} to hash table at {ptr}
/// - store {value} under key {key} in hashtable at {ptr}
/// - set {key} to {value} in hash table {ptr}
/// - hash table put {key} {value} at {ptr}
/// - insert or update {key} with {value} in hashtable {ptr}
/// - add key-value pair {key}:{value} to hash table at {ptr}
/// - map {key} to {value} in hash table at address {ptr}
/// - associate {key} with {value} in hashtable {ptr}
/// - upsert {key}={value} into hash table at {ptr}
/// - write {value} for key {key} in hashtable {ptr}
/// - hashtable insert {key} {value} at memory {ptr}
/// - put entry key={key} value={value} in hash map {ptr}
///
/// # Parameters
/// - ptr=r0 "Memory address of the hash table"
/// - key=r1 "Key to insert or update (must not be 0)"
/// - value=r2 "Value to associate with the key"
#[inline(never)]
pub unsafe fn hashtable_put(ptr: *mut u64, key: u64, value: u64) -> u64 {
    if key == 0 {
        return 0; // Invalid key
    }

    let capacity = *ptr;
    let count = *ptr.add(1);

    // Table full (leave some slack for probing)
    if count >= capacity * 3 / 4 {
        return 0;
    }

    let keys_base = ptr.add(2);
    let values_base = ptr.add(2 + capacity as usize);

    let h = key.wrapping_mul(0x9e3779b97f4a7c15);
    let mut idx = h % capacity;
    let start_idx = idx;

    loop {
        let existing_key = *keys_base.add(idx as usize);

        if existing_key == 0 {
            // Empty slot - insert here
            *keys_base.add(idx as usize) = key;
            *values_base.add(idx as usize) = value;
            *ptr.add(1) = count + 1;
            return 1;
        }

        if existing_key == key {
            // Key exists - update value
            *values_base.add(idx as usize) = value;
            return 1;
        }

        // Linear probing
        idx = (idx + 1) % capacity;
        if idx == start_idx {
            return 0; // Table full (shouldn't happen with 75% load check)
        }
    }
}

/// Get a value by key.
///
/// # Safety
/// Assumes ptr points to a valid initialized hash table.
///
/// # Neurlang Export
/// - Category: collections/hashtable
/// - Difficulty: 2
///
/// Returns: value if found, 0 if not found
///
/// # Prompts
/// - get value for key {key} from hash table at {ptr}
/// - lookup {key} in hashtable {ptr}
/// - retrieve value for {key} from hash table at {ptr}
/// - find value of key {key} in hashtable at {ptr}
/// - hash table get {key} from {ptr}
/// - read value for key {key} in hash table {ptr}
/// - fetch entry for {key} from hashtable at {ptr}
/// - get {key} from hash table at address {ptr}
/// - query hash table {ptr} for key {key}
/// - obtain value mapped to {key} in hashtable {ptr}
/// - hashtable lookup {key} at memory {ptr}
/// - search for {key} in hash table {ptr}
/// - access value of {key} in hash map {ptr}
///
/// # Parameters
/// - ptr=r0 "Memory address of the hash table"
/// - key=r1 "Key to look up"
#[inline(never)]
pub unsafe fn hashtable_get(ptr: *const u64, key: u64) -> u64 {
    if key == 0 {
        return 0;
    }

    let capacity = *ptr;
    let keys_base = ptr.add(2);
    let values_base = ptr.add(2 + capacity as usize);

    let h = key.wrapping_mul(0x9e3779b97f4a7c15);
    let mut idx = h % capacity;
    let start_idx = idx;

    loop {
        let existing_key = *keys_base.add(idx as usize);

        if existing_key == 0 {
            return 0; // Not found
        }

        if existing_key == key {
            return *values_base.add(idx as usize);
        }

        idx = (idx + 1) % capacity;
        if idx == start_idx {
            return 0; // Full loop - not found
        }
    }
}

/// Check if key exists in hash table.
///
/// # Neurlang Export
/// - Category: collections/hashtable
/// - Difficulty: 2
///
/// # Prompts
/// - check if key {key} exists in hash table at {ptr}
/// - does hashtable {ptr} contain key {key}
/// - test if {key} is in hash table at {ptr}
/// - determine if key {key} exists in hashtable {ptr}
/// - check hash table {ptr} for key {key}
/// - is {key} present in hashtable at {ptr}
/// - verify key {key} exists in hash table at address {ptr}
/// - see if hash table {ptr} has key {key}
/// - check for key {key} in hashtable at {ptr}
/// - test hashtable {ptr} contains {key}
/// - is key {key} in hash table at memory {ptr}
/// - check if hash table {ptr} has entry for {key}
/// - query if {key} exists in hash map {ptr}
///
/// # Parameters
/// - ptr=r0 "Memory address of the hash table"
/// - key=r1 "Key to check for existence"
#[inline(never)]
pub unsafe fn hashtable_contains(ptr: *const u64, key: u64) -> u64 {
    if key == 0 {
        return 0;
    }

    let capacity = *ptr;
    let keys_base = ptr.add(2);

    let h = key.wrapping_mul(0x9e3779b97f4a7c15);
    let mut idx = h % capacity;
    let start_idx = idx;

    loop {
        let existing_key = *keys_base.add(idx as usize);

        if existing_key == 0 {
            return 0;
        }

        if existing_key == key {
            return 1;
        }

        idx = (idx + 1) % capacity;
        if idx == start_idx {
            return 0;
        }
    }
}

/// Get the count of entries in the hash table.
///
/// # Neurlang Export
/// - Category: collections/hashtable
/// - Difficulty: 1
///
/// # Prompts
/// - get count of entries in hash table at {ptr}
/// - how many key-value pairs in hashtable {ptr}
/// - return hash table size for {ptr}
/// - count entries in hash table at {ptr}
/// - get number of items in hashtable {ptr}
/// - hash table length at address {ptr}
/// - find hash table entry count at {ptr}
/// - get element count for hashtable {ptr}
/// - how many entries in hash table at {ptr}
/// - return number of keys in hash table {ptr}
/// - query hashtable size at memory {ptr}
/// - get current size of hash map {ptr}
/// - count items in hash table at {ptr}
///
/// # Parameters
/// - ptr=r0 "Memory address of the hash table"
#[inline(never)]
pub unsafe fn hashtable_count(ptr: *const u64) -> u64 {
    *ptr.add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack() {
        unsafe {
            let mut buffer = [0u64; 12]; // 2 header + 10 data
            let ptr = buffer.as_mut_ptr();

            stack_init(ptr, 10);
            assert_eq!(stack_is_empty(ptr), 1);

            stack_push(ptr, 42);
            stack_push(ptr, 100);
            assert_eq!(stack_size(ptr), 2);
            assert_eq!(stack_peek(ptr), 100);

            assert_eq!(stack_pop(ptr), 100);
            assert_eq!(stack_pop(ptr), 42);
            assert_eq!(stack_is_empty(ptr), 1);
        }
    }

    #[test]
    fn test_queue() {
        unsafe {
            let mut buffer = [0u64; 14]; // 4 header + 10 data
            let ptr = buffer.as_mut_ptr();

            queue_init(ptr, 10);
            assert_eq!(queue_is_empty(ptr), 1);

            queue_enqueue(ptr, 1);
            queue_enqueue(ptr, 2);
            queue_enqueue(ptr, 3);
            assert_eq!(queue_size(ptr), 3);

            assert_eq!(queue_dequeue(ptr), 1);
            assert_eq!(queue_dequeue(ptr), 2);
            assert_eq!(queue_dequeue(ptr), 3);
            assert_eq!(queue_is_empty(ptr), 1);
        }
    }

    #[test]
    fn test_hashtable() {
        unsafe {
            let mut buffer = [0u64; 22]; // 2 header + 10 keys + 10 values
            let ptr = buffer.as_mut_ptr();

            hashtable_init(ptr, 10);
            assert_eq!(hashtable_count(ptr), 0);

            hashtable_put(ptr, 1, 100);
            hashtable_put(ptr, 2, 200);
            hashtable_put(ptr, 3, 300);

            assert_eq!(hashtable_get(ptr, 1), 100);
            assert_eq!(hashtable_get(ptr, 2), 200);
            assert_eq!(hashtable_get(ptr, 3), 300);
            assert_eq!(hashtable_get(ptr, 4), 0);

            assert_eq!(hashtable_contains(ptr, 1), 1);
            assert_eq!(hashtable_contains(ptr, 4), 0);

            // Update existing key
            hashtable_put(ptr, 1, 999);
            assert_eq!(hashtable_get(ptr, 1), 999);
        }
    }
}
