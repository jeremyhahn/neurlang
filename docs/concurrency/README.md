# Concurrency Documentation

Lightweight tasks, channels, and atomics for safe parallelism.

## Concurrency Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Concurrency Architecture                         │
└─────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────────┐
  │                        Main Thread                               │
  │  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐         │
  │  │ SPAWN   │──▶│  Task   │──▶│  JOIN   │──▶│ Result  │         │
  │  └─────────┘   └────┬────┘   └─────────┘   └─────────┘         │
  │                     │                                            │
  └─────────────────────│────────────────────────────────────────────┘
                        │
            ┌───────────┴───────────┐
            ▼                       ▼
  ┌─────────────────┐     ┌─────────────────┐
  │    Task 1       │     │    Task 2       │
  │  Independent    │     │  Independent    │
  │  Execution      │     │  Execution      │
  └────────┬────────┘     └────────┬────────┘
           │                       │
           └───────────┬───────────┘
                       ▼
             ┌─────────────────┐
             │    Channel      │
             │  Communication  │
             └─────────────────┘
```

## Tasks

### Lightweight Threads

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Task Lifecycle                                │
└─────────────────────────────────────────────────────────────────────┘

  SPAWN                 RUNNING                 COMPLETED
  ┌─────┐   create    ┌─────────┐   finish    ┌──────────┐
  │     │────────────▶│         │────────────▶│          │
  └─────┘             └────┬────┘             └────┬─────┘
                           │                       │
                       YIELD ↓↑                    │
                           │                       │
                      ┌────┴────┐                  │
                      │ WAITING │                  │
                      └─────────┘                  │
                                                   │
  JOIN ◀───────────────────────────────────────────┘
```

### API

```rust
pub struct Task {
    pub id: TaskId,
    pub state: TaskState,
    pub stack: Vec<u64>,
    pub registers: [u64; 32],
}

pub enum TaskState {
    Ready,
    Running,
    Waiting(WaitReason),
    Completed(u64),
}

// Opcodes
SPAWN rd, addr     // Create task, store ID in rd
JOIN rd, rs        // Wait for task rs, store result in rd
YIELD              // Cooperative yield
```

### Usage Pattern

```asm
; Spawn a worker task
    mov r1, worker_fn    ; Function address
    spawn r0, r1         ; r0 = task ID

; Do other work...
    mov r2, 100
    add r3, r2, r2

; Wait for result
    join r4, r0          ; r4 = task result
    halt

worker_fn:
    mov r0, 42           ; Return value
    ret
```

## Channels

### Bounded MPMC Channels

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Channel Architecture                             │
└─────────────────────────────────────────────────────────────────────┘

  Sender 1 ──┐                              ┌── Receiver 1
             │     ┌─────────────────┐      │
  Sender 2 ──┼────▶│  Ring Buffer    │──────┼── Receiver 2
             │     │  [  ][  ][  ]   │      │
  Sender 3 ──┘     └─────────────────┘      └── Receiver 3

  Properties:
  • Multiple producers (atomic push)
  • Multiple consumers (atomic pop)
  • Bounded capacity (backpressure)
  • Blocking send/recv
```

### Channel Operations

| Mode | Operation | Description |
|------|-----------|-------------|
| 0 | CREATE | Create channel with capacity |
| 1 | SEND | Send value (blocks if full) |
| 2 | RECV | Receive value (blocks if empty) |
| 3 | CLOSE | Close channel |

### API

```rust
pub struct Channel {
    pub id: ChannelId,
    pub buffer: ArrayQueue<u64>,
    pub closed: AtomicBool,
    pub waiters: Mutex<Vec<TaskId>>,
}

// Opcodes
CHAN.CREATE rd, rs   // Create channel, capacity in rs
CHAN.SEND rd, rs     // Send rs to channel rd
CHAN.RECV rd, rs     // Receive from channel rs into rd
CHAN.CLOSE rs        // Close channel rs
```

### Usage Pattern

```asm
; Producer-consumer pattern
    mov r1, 10           ; Capacity = 10
    chan.create r0, r1   ; r0 = channel ID

; Spawn producer
    mov r2, producer
    spawn r3, r2

; Spawn consumer
    mov r2, consumer
    spawn r4, r2

; Wait for both
    join r5, r3
    join r6, r4
    halt

producer:
    mov r1, 0
loop:
    chan.send r0, r1     ; Send counter
    addi r1, r1, 1
    blt r1, 100, loop
    chan.close r0
    ret

consumer:
    mov r2, 0            ; Sum
recv_loop:
    chan.recv r1, r0     ; Receive value
    beq r1, -1, done     ; -1 = closed
    add r2, r2, r1
    b recv_loop
done:
    mov r0, r2           ; Return sum
    ret
```

## Atomics

### Lock-Free Operations

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Atomic Operations                                │
└─────────────────────────────────────────────────────────────────────┘

  ATOMIC.CAS (Compare-And-Swap):
  ┌─────────────────────────────────────────────────────────────────┐
  │  if *ptr == expected:                                            │
  │      *ptr = new_value                                            │
  │      return true                                                 │
  │  else:                                                           │
  │      return false                                                │
  └─────────────────────────────────────────────────────────────────┘

  ATOMIC.XCHG (Exchange):
  ┌─────────────────────────────────────────────────────────────────┐
  │  old = *ptr                                                      │
  │  *ptr = new_value                                                │
  │  return old                                                      │
  └─────────────────────────────────────────────────────────────────┘
```

### Atomic Modes

| Mode | Operation | Description |
|------|-----------|-------------|
| 0 | CAS | Compare-and-swap |
| 1 | XCHG | Atomic exchange |
| 2 | ADD | Atomic add |
| 3 | AND | Atomic and |
| 4 | OR | Atomic or |
| 5 | XOR | Atomic xor |
| 6 | MIN | Atomic min |
| 7 | MAX | Atomic max |

### Memory Fences

```rust
pub enum FenceMode {
    Acquire = 0,   // Loads before fence visible before loads after
    Release = 1,   // Stores before fence visible before stores after
    SeqCst = 2,    // Full sequential consistency
}
```

### Lock-Free Counter Example

```asm
; Atomic increment
atomic_inc:
    load.d r1, [r0]      ; Load current value
retry:
    mov r2, r1
    addi r3, r1, 1       ; New value
    atomic.cas r1, [r0], r2, r3
    beq r1, 0, retry     ; CAS failed, retry
    ret
```

### Spinlock Example

```asm
; Acquire lock
acquire:
    mov r1, 1
spin:
    atomic.xchg r2, [r0], r1
    bne r2, 0, spin      ; Already locked, spin
    fence.acquire        ; Memory barrier
    ret

; Release lock
release:
    fence.release        ; Memory barrier
    mov r1, 0
    store.d r1, [r0]
    ret
```

## Runtime Implementation

```rust
pub struct ConcurrencyRuntime {
    tasks: DashMap<TaskId, Arc<Mutex<Task>>>,
    channels: DashMap<ChannelId, Arc<Channel>>,
    next_task_id: AtomicU64,
    next_channel_id: AtomicU64,
    scheduler: Scheduler,
}

impl ConcurrencyRuntime {
    pub fn spawn(&self, entry: u64, args: &[u64]) -> TaskId;
    pub fn join(&self, task_id: TaskId) -> u64;
    pub fn yield_current(&self);

    pub fn create_channel(&self, capacity: usize) -> ChannelId;
    pub fn send(&self, chan: ChannelId, value: u64) -> Result<()>;
    pub fn recv(&self, chan: ChannelId) -> Result<u64>;
    pub fn close(&self, chan: ChannelId);
}
```

## FFI Functions

```rust
// Task operations
#[no_mangle]
pub extern "C" fn neurlang_spawn(entry: u64, arg: u64) -> u64;  // Returns task ID

#[no_mangle]
pub extern "C" fn neurlang_join(task_id: u64) -> u64;  // Returns result

#[no_mangle]
pub extern "C" fn neurlang_yield();

// Channel operations
#[no_mangle]
pub extern "C" fn neurlang_chan_create(capacity: u64) -> u64;

#[no_mangle]
pub extern "C" fn neurlang_chan_send(chan_id: u64, value: u64) -> u64;

#[no_mangle]
pub extern "C" fn neurlang_chan_recv(chan_id: u64) -> u64;

#[no_mangle]
pub extern "C" fn neurlang_chan_close(chan_id: u64);

// Memory ordering
#[no_mangle]
pub extern "C" fn neurlang_fence(mode: u64);
```

## Safety Guarantees

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Concurrency Safety                               │
└─────────────────────────────────────────────────────────────────────┘

  1. No Shared Mutable State (by default)
     ┌─────────────────────────────────────────────────────────────┐
     │ Tasks have isolated register files                          │
     │ Communication only through channels                         │
     └─────────────────────────────────────────────────────────────┘

  2. Capability Bounds Apply
     ┌─────────────────────────────────────────────────────────────┐
     │ Shared memory requires explicit capability passing          │
     │ Atomics still respect bounds checking                       │
     └─────────────────────────────────────────────────────────────┘

  3. No Deadlock Detection (yet)
     ┌─────────────────────────────────────────────────────────────┐
     │ User responsible for avoiding cycles                        │
     │ Future: timeout support, deadlock detection                 │
     └─────────────────────────────────────────────────────────────┘
```
