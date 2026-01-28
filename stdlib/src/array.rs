//! Array functions for Neurlang stdlib
//!
//! These functions operate on arrays of u64 values in memory.

/// Sum all elements in an array.
///
/// # Safety
/// Assumes ptr points to array of at least `len` u64 elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 1
///
/// # Prompts
/// - sum all elements in {arr} with length {len}
/// - calculate the total of {arr} array containing {len} elements
/// - add up all {len} values in array {arr}
/// - compute array sum for {arr} of size {len}
/// - get the sum of {len} integers stored at {arr}
/// - accumulate all elements from {arr} with {len} items
/// - find total value of array {arr} having {len} elements
/// - reduce {arr} by addition over {len} elements
/// - sum {len} u64 values starting at {arr}
/// - calculate cumulative sum of {arr} array with {len} entries
/// - aggregate {len} numbers in {arr} into single sum
/// - return sum of all {len} elements in {arr}
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements"
/// - len=r1 "Number of elements in the array"
///
/// # Test Cases
/// - sum([1, 2, 3, 4, 5], 5) = 15
/// - sum([], 0) = 0
#[inline(never)]
pub unsafe fn sum(ptr: *const u64, len: u64) -> u64 {
    let mut total: u64 = 0;
    let mut i: u64 = 0;

    while i < len {
        total = total + *ptr.add(i as usize);
        i = i + 1;
    }

    total
}

/// Find the minimum value in an array.
///
/// # Safety
/// Assumes ptr points to array of at least `len` u64 elements.
/// Returns u64::MAX if array is empty.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 1
///
/// # Prompts
/// - find minimum value in {arr} with {len} elements
/// - get the smallest element from array {arr} of size {len}
/// - return min of {len} values in {arr}
/// - locate the lowest value in {arr} containing {len} items
/// - find smallest number in array {arr} with length {len}
/// - compute minimum across {len} elements at {arr}
/// - get min value from {len} integers stored in {arr}
/// - find the least element in {arr} of {len} entries
/// - search for minimum in array {arr} having {len} elements
/// - return lowest u64 from {arr} with {len} values
/// - determine smallest of {len} numbers in {arr}
/// - extract minimum element from {arr} array of size {len}
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements"
/// - len=r1 "Number of elements in the array"
#[inline(never)]
pub unsafe fn array_min(ptr: *const u64, len: u64) -> u64 {
    if len == 0 {
        return u64::MAX;
    }

    let mut min_val = *ptr;
    let mut i: u64 = 1;

    while i < len {
        let val = *ptr.add(i as usize);
        if val < min_val {
            min_val = val;
        }
        i = i + 1;
    }

    min_val
}

/// Find the maximum value in an array.
///
/// # Safety
/// Assumes ptr points to array of at least `len` u64 elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 1
///
/// # Prompts
/// - find maximum value in {arr} with {len} elements
/// - get the largest element from array {arr} of size {len}
/// - return max of {len} values in {arr}
/// - locate the highest value in {arr} containing {len} items
/// - find biggest number in array {arr} with length {len}
/// - compute maximum across {len} elements at {arr}
/// - get max value from {len} integers stored in {arr}
/// - find the greatest element in {arr} of {len} entries
/// - search for maximum in array {arr} having {len} elements
/// - return highest u64 from {arr} with {len} values
/// - determine largest of {len} numbers in {arr}
/// - extract maximum element from {arr} array of size {len}
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements"
/// - len=r1 "Number of elements in the array"
#[inline(never)]
pub unsafe fn array_max(ptr: *const u64, len: u64) -> u64 {
    if len == 0 {
        return 0;
    }

    let mut max_val = *ptr;
    let mut i: u64 = 1;

    while i < len {
        let val = *ptr.add(i as usize);
        if val > max_val {
            max_val = val;
        }
        i = i + 1;
    }

    max_val
}

/// Linear search for a value in an array.
///
/// # Safety
/// Assumes ptr points to array of at least `len` u64 elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 1
///
/// # Prompts
/// - linear search for {target} in {arr} with {len} elements
/// - find {target} sequentially in array {arr} of size {len}
/// - search {arr} linearly for value {target} across {len} items
/// - locate {target} in unsorted array {arr} with {len} elements
/// - scan {arr} of length {len} looking for {target}
/// - sequential search for {target} in {len} element array {arr}
/// - find index of {target} in {arr} by linear scan over {len} values
/// - iterate through {arr} of {len} items to find {target}
/// - perform linear search for {target} in {arr} containing {len} entries
/// - search array {arr} element by element for {target} up to {len}
/// - find first occurrence of {target} in {arr} with {len} elements
/// - look for {target} in {arr} array of length {len} sequentially
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements"
/// - len=r1 "Number of elements in the array"
/// - target=r2 "Value to search for"
///
/// Returns: index of value, or u64::MAX if not found
#[inline(never)]
pub unsafe fn linear_search(ptr: *const u64, len: u64, target: u64) -> u64 {
    let mut i: u64 = 0;

    while i < len {
        if *ptr.add(i as usize) == target {
            return i;
        }
        i = i + 1;
    }

    u64::MAX // Not found
}

/// Binary search in a sorted array.
///
/// # Safety
/// Assumes ptr points to sorted array of at least `len` u64 elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 2
///
/// # Prompts
/// - binary search for {target} in sorted {arr} with {len} elements
/// - find {target} using binary search in {arr} of size {len}
/// - search sorted array {arr} for {target} with {len} items
/// - locate {target} in sorted {arr} using divide and conquer over {len} elements
/// - perform binary search on {arr} of length {len} for value {target}
/// - efficient search for {target} in sorted {arr} containing {len} entries
/// - bisect {arr} array of {len} elements to find {target}
/// - binary search {len} sorted values in {arr} for {target}
/// - find index of {target} in sorted array {arr} with {len} items
/// - search {arr} with binary algorithm for {target} across {len} elements
/// - logarithmic search for {target} in sorted {arr} of {len} values
/// - divide and conquer search for {target} in {arr} with {len} entries
///
/// # Parameters
/// - arr=r0 "Pointer to sorted array of u64 elements"
/// - len=r1 "Number of elements in the array"
/// - target=r2 "Value to search for"
///
/// Returns: index of value, or u64::MAX if not found
#[inline(never)]
pub unsafe fn binary_search(ptr: *const u64, len: u64, target: u64) -> u64 {
    if len == 0 {
        return u64::MAX;
    }

    let mut left: u64 = 0;
    let mut right: u64 = len - 1;

    while left <= right {
        let mid = left + (right - left) / 2;
        let val = *ptr.add(mid as usize);

        if val == target {
            return mid;
        } else if val < target {
            left = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            right = mid - 1;
        }
    }

    u64::MAX // Not found
}

/// Reverse an array in place.
///
/// # Safety
/// Assumes ptr points to array of at least `len` u64 elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 1
///
/// # Prompts
/// - reverse array {arr} in place with {len} elements
/// - flip the order of {len} elements in {arr}
/// - reverse {arr} of size {len} in place
/// - invert element order in array {arr} containing {len} items
/// - swap elements to reverse {arr} with {len} values
/// - reverse {len} element array {arr} without extra memory
/// - flip {arr} array of length {len} end to end
/// - in-place reversal of {arr} with {len} entries
/// - mirror array {arr} containing {len} elements
/// - reverse order of {len} u64 values in {arr}
/// - turn {arr} array backwards across {len} items
/// - swap first and last elements progressively in {arr} of {len}
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements (mutable)"
/// - len=r1 "Number of elements in the array"
#[inline(never)]
pub unsafe fn reverse(ptr: *mut u64, len: u64) {
    if len < 2 {
        return;
    }

    let mut left: u64 = 0;
    let mut right: u64 = len - 1;

    while left < right {
        let temp = *ptr.add(left as usize);
        *ptr.add(left as usize) = *ptr.add(right as usize);
        *ptr.add(right as usize) = temp;

        left = left + 1;
        right = right - 1;
    }
}

/// Count elements equal to a value.
///
/// # Safety
/// Assumes ptr points to array of at least `len` u64 elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 1
///
/// # Prompts
/// - count occurrences of {target} in {arr} with {len} elements
/// - how many times does {target} appear in array {arr} of size {len}
/// - count {target} in {arr} containing {len} items
/// - find frequency of {target} in {arr} with {len} values
/// - tally occurrences of {target} in {len} element array {arr}
/// - count how many elements equal {target} in {arr} of length {len}
/// - number of {target} values in {arr} array with {len} entries
/// - count matches for {target} in {arr} across {len} elements
/// - get occurrence count of {target} in {arr} of {len} items
/// - count elements matching {target} in {arr} with {len} values
/// - frequency count of {target} in array {arr} having {len} elements
/// - count all {target} in {len} element array {arr}
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements"
/// - len=r1 "Number of elements in the array"
/// - target=r2 "Value to count"
#[inline(never)]
pub unsafe fn count(ptr: *const u64, len: u64, target: u64) -> u64 {
    let mut cnt: u64 = 0;
    let mut i: u64 = 0;

    while i < len {
        if *ptr.add(i as usize) == target {
            cnt = cnt + 1;
        }
        i = i + 1;
    }

    cnt
}

/// Fill array with a value.
///
/// # Safety
/// Assumes ptr points to array of at least `len` u64 elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 1
///
/// # Prompts
/// - fill array {arr} with {value} for {len} elements
/// - set all {len} elements in {arr} to {value}
/// - initialize {arr} of size {len} with value {value}
/// - fill {len} slots in {arr} with constant {value}
/// - write {value} to all positions in {arr} with {len} items
/// - memset {arr} to {value} for {len} u64 values
/// - populate {arr} array of length {len} with {value}
/// - fill {arr} buffer of {len} entries with {value}
/// - set {len} element array {arr} to uniform value {value}
/// - initialize all {len} elements of {arr} to {value}
/// - fill entire {arr} of size {len} with {value}
/// - assign {value} to each of {len} positions in {arr}
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements (mutable)"
/// - len=r1 "Number of elements to fill"
/// - value=r2 "Value to fill with"
#[inline(never)]
pub unsafe fn fill(ptr: *mut u64, len: u64, value: u64) {
    let mut i: u64 = 0;

    while i < len {
        *ptr.add(i as usize) = value;
        i = i + 1;
    }
}

/// Copy array from src to dst.
///
/// # Safety
/// Assumes both arrays have at least `len` elements and don't overlap.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 1
///
/// # Prompts
/// - copy {len} elements from {src} to {dst}
/// - duplicate array {src} into {dst} for {len} items
/// - memcpy {len} u64 values from {src} to {dst}
/// - copy array {src} to destination {dst} with {len} elements
/// - transfer {len} elements from {src} array to {dst}
/// - clone {src} into {dst} for {len} values
/// - copy {len} entries from source {src} to dest {dst}
/// - replicate {src} array to {dst} with {len} items
/// - copy {len} element buffer from {src} to {dst}
/// - array copy from {src} to {dst} of size {len}
/// - move {len} values from {src} into {dst} array
/// - duplicate {len} u64 elements from {src} to {dst}
///
/// # Parameters
/// - dst=r0 "Pointer to destination array (mutable)"
/// - src=r1 "Pointer to source array"
/// - len=r2 "Number of elements to copy"
#[inline(never)]
pub unsafe fn copy(dst: *mut u64, src: *const u64, len: u64) {
    let mut i: u64 = 0;

    while i < len {
        *dst.add(i as usize) = *src.add(i as usize);
        i = i + 1;
    }
}

/// Check if two arrays are equal.
///
/// # Safety
/// Assumes both arrays have at least `len` elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 1
///
/// # Prompts
/// - check if arrays {a} and {b} are equal with {len} elements
/// - compare {a} and {b} arrays of size {len} for equality
/// - test if {len} elements in {a} match {b}
/// - are arrays {a} and {b} identical for {len} items
/// - verify equality of {a} and {b} over {len} values
/// - compare {len} element arrays {a} and {b}
/// - check {a} equals {b} for {len} entries
/// - test array equality between {a} and {b} of length {len}
/// - do {len} elements of {a} equal those in {b}
/// - memcmp {a} and {b} for {len} u64 values
/// - compare {a} to {b} element by element for {len} items
/// - return true if {a} and {b} match across {len} elements
///
/// # Parameters
/// - a=r0 "Pointer to first array"
/// - b=r1 "Pointer to second array"
/// - len=r2 "Number of elements to compare"
#[inline(never)]
pub unsafe fn array_eq(a: *const u64, b: *const u64, len: u64) -> u64 {
    let mut i: u64 = 0;

    while i < len {
        if *a.add(i as usize) != *b.add(i as usize) {
            return 0;
        }
        i = i + 1;
    }

    1
}

/// Check if array is sorted in ascending order.
///
/// # Safety
/// Assumes ptr points to array of at least `len` elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 1
///
/// # Prompts
/// - check if {arr} is sorted ascending with {len} elements
/// - verify {arr} of size {len} is in sorted order
/// - is array {arr} sorted with {len} items
/// - test if {len} elements in {arr} are in ascending order
/// - check sorted property of {arr} with {len} values
/// - determine if {arr} of length {len} is sorted
/// - verify ascending order in {arr} for {len} entries
/// - is {arr} array of {len} elements already sorted
/// - check if {len} values in {arr} are non-decreasing
/// - test sorted ascending for {arr} with {len} items
/// - validate {arr} is sorted over {len} elements
/// - return true if {arr} of {len} is in sorted order
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements"
/// - len=r1 "Number of elements in the array"
#[inline(never)]
pub unsafe fn is_sorted(ptr: *const u64, len: u64) -> u64 {
    if len < 2 {
        return 1;
    }

    let mut i: u64 = 1;
    while i < len {
        if *ptr.add(i as usize) < *ptr.add((i - 1) as usize) {
            return 0;
        }
        i = i + 1;
    }

    1
}

/// Bubble sort (simple in-place sort).
///
/// # Safety
/// Assumes ptr points to array of at least `len` elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 2
///
/// # Prompts
/// - bubble sort {arr} with {len} elements
/// - sort array {arr} of size {len} using bubble sort
/// - perform bubble sort on {arr} containing {len} items
/// - in-place bubble sort {len} elements in {arr}
/// - sort {arr} of length {len} with bubble algorithm
/// - apply bubble sort to {arr} with {len} values
/// - bubble sort {len} element array {arr} ascending
/// - sort {arr} array of {len} entries using bubble method
/// - simple sort {arr} with {len} items via bubbling
/// - use bubble sort on {arr} containing {len} u64 values
/// - sort {len} numbers in {arr} using bubble sort
/// - ascending bubble sort on {arr} with {len} elements
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements (mutable)"
/// - len=r1 "Number of elements to sort"
#[inline(never)]
pub unsafe fn bubble_sort(ptr: *mut u64, len: u64) {
    if len < 2 {
        return;
    }

    let mut i: u64 = 0;
    while i < len - 1 {
        let mut j: u64 = 0;
        while j < len - 1 - i {
            let a = *ptr.add(j as usize);
            let b = *ptr.add((j + 1) as usize);
            if a > b {
                *ptr.add(j as usize) = b;
                *ptr.add((j + 1) as usize) = a;
            }
            j = j + 1;
        }
        i = i + 1;
    }
}

/// Partition for quicksort (Lomuto scheme).
///
/// # Safety
/// Assumes ptr points to array, low and high are valid indices.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 3
///
/// # Prompts
/// - partition {arr} from {low} to {high} for quicksort
/// - lomuto partition on {arr} between indices {low} and {high}
/// - partition array {arr} from {low} to {high} around pivot
/// - quicksort partition {arr} with range {low} to {high}
/// - perform partition step on {arr} from {low} to {high}
/// - divide {arr} for quicksort between {low} and {high}
/// - partition {arr} array using lomuto scheme from {low} to {high}
/// - rearrange {arr} around pivot from index {low} to {high}
/// - partition elements in {arr} between {low} and {high}
/// - quicksort helper partition {arr} range {low} to {high}
/// - lomuto partition step on {arr} from {low} through {high}
/// - split {arr} around pivot element from {low} to {high}
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements (mutable)"
/// - low=r1 "Starting index of partition range"
/// - high=r2 "Ending index of partition range (pivot location)"
#[inline(never)]
pub unsafe fn partition(ptr: *mut u64, low: u64, high: u64) -> u64 {
    let pivot = *ptr.add(high as usize);
    let mut i = low;
    let mut j = low;

    while j < high {
        if *ptr.add(j as usize) < pivot {
            // Swap
            let temp = *ptr.add(i as usize);
            *ptr.add(i as usize) = *ptr.add(j as usize);
            *ptr.add(j as usize) = temp;
            i = i + 1;
        }
        j = j + 1;
    }

    // Swap pivot into place
    let temp = *ptr.add(i as usize);
    *ptr.add(i as usize) = *ptr.add(high as usize);
    *ptr.add(high as usize) = temp;

    i
}

/// Find index of first element greater than target.
///
/// # Safety
/// Assumes ptr points to sorted array of at least `len` elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 2
///
/// # Prompts
/// - find upper bound for {target} in sorted {arr} with {len} elements
/// - get first index greater than {target} in {arr} of size {len}
/// - upper bound of {target} in sorted array {arr} containing {len} items
/// - find insertion point after {target} in sorted {arr} with {len} values
/// - locate first element exceeding {target} in {arr} of length {len}
/// - binary search upper bound for {target} in {arr} with {len} entries
/// - index of first value above {target} in sorted {arr} of {len} items
/// - find position after all {target} values in sorted {arr} with {len} elements
/// - upper bound search for {target} in {arr} array of {len} values
/// - get insertion index for {target} at end of equals in {arr} with {len}
/// - find rightmost position for {target} in sorted {arr} of {len} elements
/// - locate upper bound of {target} in {len} element sorted array {arr}
///
/// # Parameters
/// - arr=r0 "Pointer to sorted array of u64 elements"
/// - len=r1 "Number of elements in the array"
/// - target=r2 "Value to find upper bound for"
#[inline(never)]
pub unsafe fn upper_bound(ptr: *const u64, len: u64, target: u64) -> u64 {
    let mut left: u64 = 0;
    let mut right: u64 = len;

    while left < right {
        let mid = left + (right - left) / 2;
        if *ptr.add(mid as usize) <= target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    left
}

/// Find index of first element not less than target.
///
/// # Safety
/// Assumes ptr points to sorted array of at least `len` elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 2
///
/// # Prompts
/// - find lower bound for {target} in sorted {arr} with {len} elements
/// - get first index not less than {target} in {arr} of size {len}
/// - lower bound of {target} in sorted array {arr} containing {len} items
/// - find insertion point for {target} in sorted {arr} with {len} values
/// - locate first element >= {target} in {arr} of length {len}
/// - binary search lower bound for {target} in {arr} with {len} entries
/// - index of first value >= {target} in sorted {arr} of {len} items
/// - find leftmost position for {target} in sorted {arr} with {len} elements
/// - lower bound search for {target} in {arr} array of {len} values
/// - get insertion index for {target} in sorted {arr} with {len}
/// - find first position >= {target} in sorted {arr} of {len} elements
/// - locate lower bound of {target} in {len} element sorted array {arr}
///
/// # Parameters
/// - arr=r0 "Pointer to sorted array of u64 elements"
/// - len=r1 "Number of elements in the array"
/// - target=r2 "Value to find lower bound for"
#[inline(never)]
pub unsafe fn lower_bound(ptr: *const u64, len: u64, target: u64) -> u64 {
    let mut left: u64 = 0;
    let mut right: u64 = len;

    while left < right {
        let mid = left + (right - left) / 2;
        if *ptr.add(mid as usize) < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    left
}

/// Insertion sort (stable, efficient for small arrays).
///
/// # Safety
/// Assumes ptr points to array of at least `len` elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 2
///
/// # Prompts
/// - insertion sort {arr} with {len} elements
/// - sort array {arr} of size {len} using insertion sort
/// - perform insertion sort on {arr} containing {len} items
/// - stable sort {len} elements in {arr} using insertion method
/// - sort {arr} of length {len} with insertion algorithm
/// - apply insertion sort to {arr} with {len} values
/// - insertion sort {len} element array {arr} ascending
/// - sort {arr} array of {len} entries using insertion method
/// - sort small array {arr} with {len} items via insertion
/// - use insertion sort on {arr} containing {len} u64 values
/// - sort {len} numbers in {arr} using insertion sort
/// - stable ascending sort on {arr} with {len} elements
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements (mutable)"
/// - len=r1 "Number of elements to sort"
#[inline(never)]
pub unsafe fn insertion_sort(ptr: *mut u64, len: u64) {
    if len < 2 {
        return;
    }

    let mut i: u64 = 1;
    while i < len {
        let key = *ptr.add(i as usize);
        let mut j = i;

        while j > 0 && *ptr.add((j - 1) as usize) > key {
            *ptr.add(j as usize) = *ptr.add((j - 1) as usize);
            j = j - 1;
        }

        *ptr.add(j as usize) = key;
        i = i + 1;
    }
}

/// Quicksort implementation using heapsort (in-place, O(n log n) guaranteed).
///
/// # Safety
/// Assumes ptr points to array of at least `len` elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 3
///
/// # Prompts
/// - quicksort array {arr} with {len} elements
/// - sort {arr} of size {len} using quicksort
/// - perform quicksort on {arr} containing {len} items
/// - in-place sort {len} elements in {arr} with quicksort
/// - sort {arr} of length {len} efficiently
/// - apply quicksort to {arr} with {len} values
/// - sort {len} element array {arr} using heap-based quicksort
/// - fast sort {arr} array of {len} entries
/// - efficient sort for {arr} with {len} items
/// - use quicksort on {arr} containing {len} u64 values
/// - sort {len} numbers in {arr} with O(n log n) guarantee
/// - heapsort {arr} with {len} elements for guaranteed performance
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements (mutable)"
/// - len=r1 "Number of elements to sort"
///
/// Note: Uses heapsort which is in-place and doesn't require recursion stack.
/// All helper logic is inlined to avoid cross-function calls.
#[inline(never)]
pub unsafe fn quicksort(ptr: *mut u64, len: u64) {
    if len < 2 {
        return;
    }

    // Heapsort: Build max-heap
    // Start from last non-leaf node
    let mut start = (len - 2) / 2;
    loop {
        // Inline sift_down(ptr, start, len - 1)
        let heap_end = len - 1;
        let mut root = start;
        loop {
            let left_child = 2 * root + 1;
            if left_child > heap_end {
                break;
            }
            let right_child = left_child + 1;
            let mut swap = root;
            if *ptr.add(swap as usize) < *ptr.add(left_child as usize) {
                swap = left_child;
            }
            if right_child <= heap_end && *ptr.add(swap as usize) < *ptr.add(right_child as usize) {
                swap = right_child;
            }
            if swap == root {
                break;
            }
            let tmp = *ptr.add(root as usize);
            *ptr.add(root as usize) = *ptr.add(swap as usize);
            *ptr.add(swap as usize) = tmp;
            root = swap;
        }
        // End inline sift_down

        if start == 0 {
            break;
        }
        start = start - 1;
    }

    // Extract elements from heap one by one
    let mut end = len - 1;
    while end > 0 {
        // Swap max (root) with last element
        let tmp = *ptr.add(0);
        *ptr.add(0) = *ptr.add(end as usize);
        *ptr.add(end as usize) = tmp;

        end = end - 1;

        // Inline sift_down(ptr, 0, end)
        let mut root: u64 = 0;
        loop {
            let left_child = 2 * root + 1;
            if left_child > end {
                break;
            }
            let right_child = left_child + 1;
            let mut swap = root;
            if *ptr.add(swap as usize) < *ptr.add(left_child as usize) {
                swap = left_child;
            }
            if right_child <= end && *ptr.add(swap as usize) < *ptr.add(right_child as usize) {
                swap = right_child;
            }
            if swap == root {
                break;
            }
            let tmp2 = *ptr.add(root as usize);
            *ptr.add(root as usize) = *ptr.add(swap as usize);
            *ptr.add(swap as usize) = tmp2;
            root = swap;
        }
        // End inline sift_down
    }
}

/// Merge two sorted halves of an array.
///
/// # Safety
/// Assumes ptr points to array where [0..mid) and [mid..len) are sorted.
/// Requires temp buffer of at least `len` elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 3
///
/// # Prompts
/// - merge sorted halves of {arr} at {mid} with total length {len} using {temp}
/// - combine two sorted subarrays in {arr} split at {mid} for {len} elements into {temp}
/// - merge {arr} halves [0..{mid}) and [{mid}..{len}) using buffer {temp}
/// - merge sort helper combining {arr} at midpoint {mid} with {len} total using {temp}
/// - merge two sorted sections of {arr} at index {mid} with size {len} via {temp}
/// - combine sorted partitions in {arr} divided at {mid} for {len} items using {temp}
/// - merge sorted halves of array {arr} with split at {mid} length {len} buffer {temp}
/// - join two sorted runs in {arr} at {mid} totaling {len} elements using {temp}
/// - merge operation on {arr} with pivot {mid} and length {len} temp buffer {temp}
/// - combine {arr} sorted halves at {mid} with {len} elements into {temp}
/// - merge sorted left and right of {arr} at {mid} for {len} using {temp}
/// - two-way merge of {arr} split at {mid} with {len} total via {temp} buffer
///
/// # Parameters
/// - arr=r0 "Pointer to array with two sorted halves (mutable)"
/// - temp=r1 "Pointer to temporary buffer of at least len elements"
/// - mid=r2 "Index where second sorted half begins"
/// - len=r3 "Total number of elements in the array"
#[inline(never)]
pub unsafe fn merge(ptr: *mut u64, temp: *mut u64, mid: u64, len: u64) {
    // Copy to temp
    let mut i: u64 = 0;
    while i < len {
        *temp.add(i as usize) = *ptr.add(i as usize);
        i = i + 1;
    }

    // Merge back
    let mut left: u64 = 0;
    let mut right = mid;
    let mut dest: u64 = 0;

    while left < mid && right < len {
        if *temp.add(left as usize) <= *temp.add(right as usize) {
            *ptr.add(dest as usize) = *temp.add(left as usize);
            left = left + 1;
        } else {
            *ptr.add(dest as usize) = *temp.add(right as usize);
            right = right + 1;
        }
        dest = dest + 1;
    }

    // Copy remaining elements
    while left < mid {
        *ptr.add(dest as usize) = *temp.add(left as usize);
        left = left + 1;
        dest = dest + 1;
    }

    while right < len {
        *ptr.add(dest as usize) = *temp.add(right as usize);
        right = right + 1;
        dest = dest + 1;
    }
}

/// Remove duplicates from a sorted array.
///
/// # Safety
/// Assumes ptr points to sorted array of at least `len` elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 2
///
/// # Prompts
/// - remove duplicates from sorted {arr} with {len} elements
/// - deduplicate sorted array {arr} of size {len}
/// - unique elements in sorted {arr} containing {len} items
/// - eliminate duplicates from {arr} of length {len}
/// - remove duplicate values from sorted {arr} with {len} entries
/// - in-place dedup of sorted {arr} with {len} elements
/// - keep only unique values in sorted {arr} of {len} items
/// - remove repeated elements from sorted {arr} with {len} values
/// - dedupe {len} element sorted array {arr}
/// - filter duplicates from sorted {arr} containing {len} entries
/// - compress {arr} by removing duplicates over {len} elements
/// - return unique count after deduping sorted {arr} of {len}
///
/// # Parameters
/// - arr=r0 "Pointer to sorted array of u64 elements (mutable)"
/// - len=r1 "Number of elements in the array"
///
/// Returns: new length after removing duplicates
#[inline(never)]
pub unsafe fn unique(ptr: *mut u64, len: u64) -> u64 {
    if len < 2 {
        return len;
    }

    let mut write_idx: u64 = 1;
    let mut read_idx: u64 = 1;

    while read_idx < len {
        if *ptr.add(read_idx as usize) != *ptr.add((write_idx - 1) as usize) {
            *ptr.add(write_idx as usize) = *ptr.add(read_idx as usize);
            write_idx = write_idx + 1;
        }
        read_idx = read_idx + 1;
    }

    write_idx
}

/// Rotate array left by k positions.
///
/// # Safety
/// Assumes ptr points to array of at least `len` elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 2
///
/// # Prompts
/// - rotate array {arr} left by {k} positions with {len} elements
/// - left rotate {arr} of size {len} by {k} places
/// - shift {arr} left cyclically by {k} over {len} items
/// - circular left shift {arr} by {k} positions for {len} values
/// - rotate {len} element array {arr} leftward by {k}
/// - left rotation of {arr} with {len} entries by {k} positions
/// - cycle {arr} left by {k} across {len} elements
/// - move first {k} elements of {arr} to end for {len} total
/// - rotate {arr} array of {len} items left by {k}
/// - left circular rotate {arr} with {len} values by {k}
/// - shift {arr} leftward {k} times over {len} elements
/// - perform left rotation on {arr} of {len} by {k} positions
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements (mutable)"
/// - len=r1 "Number of elements in the array"
/// - k=r2 "Number of positions to rotate left"
#[inline(never)]
pub unsafe fn rotate_left(ptr: *mut u64, len: u64, k: u64) {
    if len == 0 || k == 0 {
        return;
    }

    let rot_amount = k % len;
    if rot_amount == 0 {
        return;
    }

    // Reverse first k elements
    reverse(ptr, rot_amount);
    // Reverse remaining elements
    reverse(ptr.add(rot_amount as usize), len - rot_amount);
    // Reverse entire array
    reverse(ptr, len);
}

/// Rotate array right by k positions.
///
/// # Safety
/// Assumes ptr points to array of at least `len` elements.
///
/// # Neurlang Export
/// - Category: array
/// - Difficulty: 2
///
/// # Prompts
/// - rotate array {arr} right by {k} positions with {len} elements
/// - right rotate {arr} of size {len} by {k} places
/// - shift {arr} right cyclically by {k} over {len} items
/// - circular right shift {arr} by {k} positions for {len} values
/// - rotate {len} element array {arr} rightward by {k}
/// - right rotation of {arr} with {len} entries by {k} positions
/// - cycle {arr} right by {k} across {len} elements
/// - move last {k} elements of {arr} to front for {len} total
/// - rotate {arr} array of {len} items right by {k}
/// - right circular rotate {arr} with {len} values by {k}
/// - shift {arr} rightward {k} times over {len} elements
/// - perform right rotation on {arr} of {len} by {k} positions
///
/// # Parameters
/// - arr=r0 "Pointer to array of u64 elements (mutable)"
/// - len=r1 "Number of elements in the array"
/// - k=r2 "Number of positions to rotate right"
#[inline(never)]
pub unsafe fn rotate_right(ptr: *mut u64, len: u64, k: u64) {
    if len == 0 || k == 0 {
        return;
    }

    let rot_amount = k % len;
    if rot_amount == 0 {
        return;
    }

    rotate_left(ptr, len, len - rot_amount);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum() {
        unsafe {
            let arr = [1u64, 2, 3, 4, 5];
            assert_eq!(sum(arr.as_ptr(), 5), 15);
            assert_eq!(sum(arr.as_ptr(), 0), 0);
        }
    }

    #[test]
    fn test_min_max() {
        unsafe {
            let arr = [5u64, 2, 8, 1, 9];
            assert_eq!(array_min(arr.as_ptr(), 5), 1);
            assert_eq!(array_max(arr.as_ptr(), 5), 9);
        }
    }

    #[test]
    fn test_linear_search() {
        unsafe {
            let arr = [10u64, 20, 30, 40, 50];
            assert_eq!(linear_search(arr.as_ptr(), 5, 30), 2);
            assert_eq!(linear_search(arr.as_ptr(), 5, 100), u64::MAX);
        }
    }

    #[test]
    fn test_binary_search() {
        unsafe {
            let arr = [10u64, 20, 30, 40, 50];
            assert_eq!(binary_search(arr.as_ptr(), 5, 30), 2);
            assert_eq!(binary_search(arr.as_ptr(), 5, 100), u64::MAX);
        }
    }

    #[test]
    fn test_reverse() {
        unsafe {
            let mut arr = [1u64, 2, 3, 4, 5];
            reverse(arr.as_mut_ptr(), 5);
            assert_eq!(arr, [5, 4, 3, 2, 1]);
        }
    }

    #[test]
    fn test_bubble_sort() {
        unsafe {
            let mut arr = [5u64, 2, 8, 1, 9];
            bubble_sort(arr.as_mut_ptr(), 5);
            assert_eq!(arr, [1, 2, 5, 8, 9]);
        }
    }

    #[test]
    fn test_insertion_sort() {
        unsafe {
            let mut arr = [5u64, 2, 8, 1, 9, 3];
            insertion_sort(arr.as_mut_ptr(), 6);
            assert_eq!(arr, [1, 2, 3, 5, 8, 9]);
        }
    }

    #[test]
    fn test_quicksort() {
        unsafe {
            let mut arr = [5u64, 2, 8, 1, 9, 3, 7, 4, 6, 0];
            quicksort(arr.as_mut_ptr(), 10);
            assert_eq!(arr, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

            // Test with larger array
            let mut large: [u64; 100] = core::array::from_fn(|i| (100 - i) as u64);
            quicksort(large.as_mut_ptr(), 100);
            for i in 0..100 {
                assert_eq!(large[i], (i + 1) as u64);
            }
        }
    }

    #[test]
    fn test_unique() {
        unsafe {
            let mut arr = [1u64, 1, 2, 2, 2, 3, 4, 4, 5];
            let new_len = unique(arr.as_mut_ptr(), 9);
            assert_eq!(new_len, 5);
            assert_eq!(&arr[..5], &[1, 2, 3, 4, 5]);
        }
    }

    #[test]
    fn test_rotate() {
        unsafe {
            let mut arr = [1u64, 2, 3, 4, 5];
            rotate_left(arr.as_mut_ptr(), 5, 2);
            assert_eq!(arr, [3, 4, 5, 1, 2]);

            let mut arr2 = [1u64, 2, 3, 4, 5];
            rotate_right(arr2.as_mut_ptr(), 5, 2);
            assert_eq!(arr2, [4, 5, 1, 2, 3]);
        }
    }
}
