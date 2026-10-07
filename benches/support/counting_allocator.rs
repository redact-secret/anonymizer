//! Narrow benchmark-only unsafe proxy; no runtime library is affected.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
struct Counting;
static TRACK: AtomicBool = AtomicBool::new(false);
static CALLS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
// SAFETY: Every pointer, layout, and requested size is forwarded unchanged to
// System. No pointer is dereferenced, retained, or fabricated. Relaxed atomics
// only observe successful allocations; instrumentation is single-threaded and
// recording performs no allocation/reentry. Deallocation always uses System.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: caller supplies valid layout, forwarded to the system allocator.
        let result = unsafe { System.alloc(layout) };
        if !result.is_null() && TRACK.load(Ordering::Relaxed) {
            record(layout.size());
        }
        result
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: same unchanged valid layout; zeroing remains System's ownership.
        let result = unsafe { System.alloc_zeroed(layout) };
        if !result.is_null() && TRACK.load(Ordering::Relaxed) {
            record(layout.size());
        }
        result
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: original allocation/layout/new size are forwarded unchanged.
        let result = unsafe { System.realloc(ptr, layout, size) };
        if !result.is_null() && TRACK.load(Ordering::Relaxed) {
            record(size);
        }
        result
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: caller's pointer/layout are unchanged; no ownership retained.
        unsafe { System.dealloc(ptr, layout) }
    }
}
fn record(bytes: usize) {
    CALLS.fetch_add(1, Ordering::Relaxed);
    BYTES.fetch_add(bytes, Ordering::Relaxed);
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;
pub fn measured<T>(f: impl FnOnce() -> T) -> (T, usize, usize) {
    CALLS.store(0, Ordering::Relaxed);
    BYTES.store(0, Ordering::Relaxed);
    TRACK.store(true, Ordering::Relaxed);
    let value = f();
    TRACK.store(false, Ordering::Relaxed);
    (
        value,
        CALLS.load(Ordering::Relaxed),
        BYTES.load(Ordering::Relaxed),
    )
}
pub fn allocator_smoke() {
    let (mut buffer, calls, bytes) = measured(|| Vec::<u8>::with_capacity(64));
    assert!(calls == 1 && bytes == 64, "allocator alloc smoke failed");
    // SAFETY: this exact valid layout is allocated through the same global proxy
    // and freed once using its unchanged pointer/layout; no memory is dereferenced.
    let layout = Layout::from_size_align(32, 8).unwrap();
    let (pointer, zeroed_calls, zeroed_bytes) =
        measured(|| unsafe { ALLOCATOR.alloc_zeroed(layout) });
    assert!(
        !pointer.is_null() && zeroed_calls == 1 && zeroed_bytes == 32,
        "allocator zeroed smoke failed"
    );
    // SAFETY: pointer came from alloc_zeroed(layout) above and is freed once.
    unsafe { ALLOCATOR.dealloc(pointer, layout) };
    buffer.resize(64, 0);
    let (_, calls, bytes) = measured(|| buffer.reserve_exact(64));
    assert!(calls == 1 && bytes == 128, "allocator realloc smoke failed");
    let (_, calls, _) = measured(|| drop(buffer));
    assert!(calls == 0, "dealloc must not count as allocation");
}
